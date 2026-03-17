//! Bridge module for Claude Code integration
//!
//! This module provides bidirectional communication between Claude Code
//! and Parry, allowing real-time validation of file writes before they
//! are committed to disk.

use crate::config::DaemonConfig;
use oparry_core::{Error, Issue, Result, ValidationResult};
use oparry_parser::parser_for_path;
use oparry_validators::Validators;
use oparry_wrapper::{
    ipc::IpcChannel,
    protocol::{ClaudeRequest, ClaudeResponse, IssueDetail, IssueSeverity},
};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Bridge configuration
#[derive(Debug, Clone)]
pub struct BridgeConfig {
    /// Enable strict mode (block on errors)
    pub strict_mode: bool,
    /// Enable auto-fix for fixable issues
    pub auto_fix: bool,
    /// Maximum file size to validate (bytes)
    pub max_file_size: usize,
    /// Timeout for validation (seconds)
    pub validation_timeout: u64,
}

impl Default for BridgeConfig {
    fn default() -> Self {
        Self {
            strict_mode: false,
            auto_fix: true,
            max_file_size: 1_000_000, // 1MB
            validation_timeout: 30,
        }
    }
}

/// Bridge state shared across connections
#[derive(Clone)]
pub struct BridgeState {
    /// Configuration
    config: BridgeConfig,
    /// Validators
    validators: Arc<RwLock<Validators>>,
    /// Active connections count
    connections: Arc<RwLock<usize>>,
}

impl BridgeState {
    /// Create new bridge state
    pub fn new(config: BridgeConfig) -> Self {
        Self {
            config,
            validators: Arc::new(RwLock::new(Validators::new())),
            connections: Arc::new(RwLock::new(0)),
        }
    }

    /// Create from daemon config
    pub fn from_daemon_config(daemon_config: &DaemonConfig) -> Self {
        let bridge_config = BridgeConfig {
            strict_mode: daemon_config.strict_mode,
            auto_fix: daemon_config.auto_fix,
            max_file_size: daemon_config.max_file_size.unwrap_or(1_000_000),
            validation_timeout: daemon_config.validation_timeout.unwrap_or(30),
        };
        Self::new(bridge_config)
    }

    /// Get active connection count
    pub async fn connection_count(&self) -> usize {
        *self.connections.read().await
    }

    /// Increment connection count
    pub async fn add_connection(&self) {
        let mut count = self.connections.write().await;
        *count += 1;
    }

    /// Decrement connection count
    pub async fn remove_connection(&self) {
        let mut count = self.connections.write().await;
        if *count > 0 {
            *count -= 1;
        }
    }

    /// Set validators
    pub async fn set_validators(&self, validators: Validators) {
        let mut guard = self.validators.write().await;
        *guard = validators;
    }
}

/// Claude Code bridge handler
pub struct ClaudeBridge {
    state: BridgeState,
}

impl ClaudeBridge {
    /// Create new bridge
    pub fn new(state: BridgeState) -> Self {
        Self { state }
    }

    /// Create with default config
    pub fn default_config() -> Self {
        Self::new(BridgeState::new(BridgeConfig::default()))
    }

    /// Create from daemon config
    pub fn from_daemon_config(daemon_config: &DaemonConfig) -> Self {
        Self::new(BridgeState::from_daemon_config(daemon_config))
    }

    /// Handle a Claude Code request
    pub async fn handle_request(&self, request: ClaudeRequest) -> ClaudeResponse {
        debug!("Handling request: {:?}", request.id());

        match request {
            ClaudeRequest::WriteFile(wr) => self.handle_write_file(wr).await,
            ClaudeRequest::EditFile(ed) => self.handle_edit_file(ed).await,
            ClaudeRequest::Ping => ClaudeResponse::Pong,
        }
    }

    /// Handle write file request
    async fn handle_write_file(
        &self,
        request: oparry_wrapper::protocol::WriteFileRequest,
    ) -> ClaudeResponse {
        let path = &request.path;

        // Check file size
        if request.content.len() > self.state.config.max_file_size {
            return ClaudeResponse::warning(
                &request.id,
                format!(
                    "File too large ({} bytes), skipping validation",
                    request.content.len()
                ),
                vec![],
            );
        }

        // Get validators
        let validators = self.state.validators.read().await;

        // Parse the file
        let parser = parser_for_path(path);
        let parse_result = parser.parse(&request.content);

        match parse_result {
            Ok(parsed) => {
                // Validate
                match validators.validate(&parsed, path) {
                    Ok(validation) => {
                        if validation.passed {
                            debug!("Validation passed for {:?}", path);
                            ClaudeResponse::approved(&request.id)
                        } else {
                            // Check if we should auto-fix
                            if self.state.config.auto_fix {
                                if let Some(fixed_content) =
                                    self.try_auto_fix(&request.content, &validation).await
                                {
                                    return ClaudeResponse::approved_with_fix(
                                        &request.id,
                                        fixed_content,
                                    );
                                }
                            }

                            // Check severity
                            let has_errors = validation
                                .issues
                                .iter()
                                .any(|i| matches!(i.level, oparry_core::IssueLevel::Error));

                            let issues: Vec<IssueDetail> =
                                validation.issues.into_iter().map(Into::into).collect();

                            if has_errors && self.state.config.strict_mode {
                                ClaudeResponse::rejected(
                                    &request.id,
                                    "Validation failed with errors in strict mode",
                                    issues,
                                )
                            } else if has_errors {
                                ClaudeResponse::rejected(
                                    &request.id,
                                    "Validation failed with errors",
                                    issues,
                                )
                            } else {
                                ClaudeResponse::warning(
                                    &request.id,
                                    format!("Validation passed with {} warning(s)", issues.len()),
                                    issues,
                                )
                            }
                        }
                    }
                    Err(e) => {
                        error!("Validation error: {}", e);
                        ClaudeResponse::protocol_error(format!("Validation error: {}", e))
                    }
                }
            }
            Err(e) => {
                // Parse error
                warn!("Parse error for {:?}: {}", path, e);
                ClaudeResponse::rejected(
                    &request.id,
                    format!("Failed to parse file: {}", e),
                    vec![IssueDetail {
                        code: "parse-error".to_string(),
                        level: IssueSeverity::Error,
                        message: format!("Parse error: {}", e),
                        line: None,
                        column: None,
                        suggestion: None,
                        context: None,
                    }],
                )
            }
        }
    }

    /// Handle edit file request
    async fn handle_edit_file(
        &self,
        request: oparry_wrapper::protocol::EditFileRequest,
    ) -> ClaudeResponse {
        let path = &request.path;

        // Read current file content
        let current_content = match tokio::fs::read_to_string(path).await {
            Ok(content) => content,
            Err(e) => {
                // File doesn't exist, treat as write
                debug!("File {:?} doesn't exist, treating as write", path);
                return self
                    .handle_write_file(oparry_wrapper::protocol::WriteFileRequest {
                        id: request.id,
                        path: path.clone(),
                        content: request.new_string.clone(),
                        encoding: None,
                        create_dirs: None,
                    })
                    .await;
            }
        };

        // Apply edit
        let new_content = if request.replace_all.unwrap_or(false) {
            current_content.replace(&request.old_string, &request.new_string)
        } else {
            // Replace first occurrence only
            if let Some(pos) = current_content.find(&request.old_string) {
                format!(
                    "{}{}{}",
                    &current_content[..pos],
                    &request.new_string,
                    &current_content[pos + request.old_string.len()..]
                )
            } else {
                // Old string not found, reject
                return ClaudeResponse::rejected(
                    &request.id,
                    "Old string not found in file",
                    vec![IssueDetail {
                        code: "edit-no-match".to_string(),
                        level: IssueSeverity::Error,
                        message: format!("String '{}' not found in file", request.old_string),
                        line: None,
                        column: None,
                        suggestion: Some("Read the file first to verify the content".to_string()),
                        context: None,
                    }],
                );
            }
        };

        // Validate the new content
        let write_request = oparry_wrapper::protocol::WriteFileRequest {
            id: request.id,
            path: path.clone(),
            content: new_content,
            encoding: None,
            create_dirs: None,
        };

        self.handle_write_file(write_request).await
    }

    /// Try to auto-fix issues in the content
    async fn try_auto_fix(&self, content: &str, validation: &ValidationResult) -> Option<String> {
        // Start with original content
        let mut fixed = content.to_string();

        // Apply fixes for common issues
        for issue in &validation.issues {
            match issue.code.as_str() {
                "tailwind-invalid-class" => {
                    // Remove invalid Tailwind classes
                    if let Some(ref context) = issue.context {
                        if let Some(ref suggestion) = issue.suggestion {
                            fixed = fixed.replace(context, suggestion);
                        }
                    }
                }
                "trailing-whitespace" => {
                    // Remove trailing whitespace
                    fixed = fixed
                        .lines()
                        .map(|line| line.trim_end().to_string())
                        .collect::<Vec<_>>()
                        .join("\n");
                }
                _ => {
                    // No auto-fix available for this issue
                    debug!("No auto-fix for issue: {}", issue.code);
                }
            }
        }

        // Check if we made any changes
        if fixed != content {
            Some(fixed)
        } else {
            None
        }
    }

    /// Run the IPC loop
    pub async fn run_ipc_loop(&self) -> Result<()> {
        info!("Starting Claude Code bridge IPC loop");

        let channel = IpcChannel::stdio();
        let bridge = self.clone();

        // Track connection
        self.state.add_connection().await;

        let handler = move |request: ClaudeRequest| -> Result<ClaudeResponse> {
            // This is a sync handler, but we need to call async methods
            // For now, we'll use a simple approach
            let rt = tokio::runtime::Handle::try_current()
                .map_err(|e| Error::Wrapper(format!("No runtime: {}", e)))?;

            Ok(rt.block_on(async { bridge.handle_request(request).await }))
        };

        let result = channel.run_loop(handler);

        // Remove connection tracking
        self.state.remove_connection().await;

        result
    }
}

impl Clone for ClaudeBridge {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
        }
    }
}

/// Builder for creating a configured bridge
pub struct ClaudeBridgeBuilder {
    config: BridgeConfig,
    validators: Option<Validators>,
}

impl ClaudeBridgeBuilder {
    /// Create new builder
    pub fn new() -> Self {
        Self {
            config: BridgeConfig::default(),
            validators: None,
        }
    }

    /// Set strict mode
    pub fn strict_mode(mut self, strict: bool) -> Self {
        self.config.strict_mode = strict;
        self
    }

    /// Set auto-fix
    pub fn auto_fix(mut self, auto_fix: bool) -> Self {
        self.config.auto_fix = auto_fix;
        self
    }

    /// Set max file size
    pub fn max_file_size(mut self, size: usize) -> Self {
        self.config.max_file_size = size;
        self
    }

    /// Set validators
    pub fn validators(mut self, validators: Validators) -> Self {
        self.validators = Some(validators);
        self
    }

    /// Build the bridge
    pub async fn build(self) -> ClaudeBridge {
        let state = BridgeState::new(self.config);

        if let Some(validators) = self.validators {
            state.set_validators(validators).await;
        }

        ClaudeBridge::new(state)
    }
}

impl Default for ClaudeBridgeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_config_default() {
        let config = BridgeConfig::default();
        assert_eq!(config.strict_mode, false);
        assert_eq!(config.auto_fix, true);
        assert_eq!(config.max_file_size, 1_000_000);
    }

    #[test]
    fn test_bridge_state() {
        let state = BridgeState::new(BridgeConfig::default());
        assert_eq!(state.config.strict_mode, false);
    }

    #[tokio::test]
    async fn test_connection_tracking() {
        let state = BridgeState::new(BridgeConfig::default());
        assert_eq!(state.connection_count().await, 0);

        state.add_connection().await;
        assert_eq!(state.connection_count().await, 1);

        state.remove_connection().await;
        assert_eq!(state.connection_count().await, 0);
    }

    #[tokio::test]
    async fn test_ping_request() {
        let bridge = ClaudeBridge::default_config();
        let response = bridge.handle_request(ClaudeRequest::Ping).await;
        assert!(matches!(response, ClaudeResponse::Pong));
    }
}
