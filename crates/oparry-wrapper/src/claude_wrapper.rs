//! Claude Code wrapper implementation
//!
//! This module provides the main wrapper that intercepts Claude Code
//! file operations and validates them before writing to disk.

use crate::ipc::IpcChannel;
use crate::protocol::{ClaudeRequest, ClaudeResponse, IssueDetail};
use crate::{WrapConfig, ValidatorEngine};
use oparry_autofix::{AutoFixConfig, AutoFixer, FixStrategy};
use oparry_core::Result;
use oparry_parser::Language;
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Claude Code wrapper - intercepts and validates file operations
pub struct ClaudeWrapper {
    /// Validation engine
    validator: Arc<ValidatorEngine>,
    /// Auto-fix engine
    autofixer: Option<AutoFixer>,
    /// Wrapper configuration
    config: WrapConfig,
    /// IPC channel
    ipc: IpcChannel,
}

impl ClaudeWrapper {
    /// Create a new Claude wrapper
    pub fn new(validator: Arc<ValidatorEngine>, config: WrapConfig) -> Self {
        let autofixer = if config.enable_autofix {
            Some(AutoFixer::with_config(AutoFixConfig {
                strategy: match config.autofix_strategy.as_deref() {
                    Some("safe") => FixStrategy::Safe,
                    Some("aggressive") => FixStrategy::Aggressive,
                    _ => FixStrategy::Moderate,
                },
                dry_run: config.dry_run,
                ..Default::default()
            }))
        } else {
            None
        };

        Self {
            validator,
            autofixer,
            config,
            ipc: IpcChannel::stdio(),
        }
    }

    /// Create wrapper with custom IPC channel
    pub fn with_ipc(validator: Arc<ValidatorEngine>, config: WrapConfig, ipc: IpcChannel) -> Self {
        let autofixer = if config.enable_autofix {
            Some(AutoFixer::with_config(AutoFixConfig {
                strategy: match config.autofix_strategy.as_deref() {
                    Some("safe") => FixStrategy::Safe,
                    Some("aggressive") => FixStrategy::Aggressive,
                    _ => FixStrategy::Moderate,
                },
                dry_run: config.dry_run,
                ..Default::default()
            }))
        } else {
            None
        };

        Self {
            validator,
            autofixer,
            config,
            ipc,
        }
    }

    /// Enable or disable auto-fix
    pub fn with_autofix(mut self, enable: bool) -> Self {
        if enable && self.autofixer.is_none() {
            self.autofixer = Some(AutoFixer::new());
        } else if !enable {
            self.autofixer = None;
        }
        self
    }

    /// Run the wrapper - enters IPC loop
    ///
    /// This blocks and handles requests until stdin is closed
    pub fn run(&self) -> Result<()> {
        info!("Starting Claude Code wrapper (v{})", env!("CARGO_PKG_VERSION"));

        let validator = Arc::clone(&self.validator);
        let config = self.config.clone();
        let ipc = self.ipc.clone();

        ipc.run_loop(move |request| {
            Self::handle_request(request, &validator, &config, &None)
        })
    }

    /// Handle a single request
    fn handle_request(
        request: ClaudeRequest,
        validator: &ValidatorEngine,
        config: &WrapConfig,
        autofixer: &Option<AutoFixer>,
    ) -> Result<ClaudeResponse> {
        match request {
            ClaudeRequest::WriteFile(write_req) => {
                Self::handle_write_file(write_req, validator, config, autofixer)
            }
            ClaudeRequest::EditFile(edit_req) => {
                Self::handle_edit_file(edit_req, validator, config, autofixer)
            }
            ClaudeRequest::Ping => {
                debug!("Received ping, sending pong");
                Ok(ClaudeResponse::Pong)
            }
        }
    }

    /// Handle write file request
    fn handle_write_file(
        request: crate::protocol::WriteFileRequest,
        validator: &ValidatorEngine,
        config: &WrapConfig,
        autofixer: &Option<AutoFixer>,
    ) -> Result<ClaudeResponse> {
        let file_path = &request.path;
        let content = &request.content;
        let request_id = &request.id;

        info!("Validating write to: {}", file_path.display());

        // Check path validation first
        let wrapper = super::StdioWrapper::new(config.clone());
        if let Err(e) = wrapper.validate_path(file_path) {
            warn!("Path validation failed: {}", e);
            return Ok(ClaudeResponse::rejected(
                request_id,
                format!("Path not allowed: {}", e),
                vec![],
            ));
        }

        // Detect language from file extension
        let language = Self::detect_language(file_path);
        debug!("Detected language: {:?}", language);

        // Run validation
        let result = validator.validate_string(content, language, file_path);

        if result.passed {
            info!("Validation passed for: {}", file_path.display());
            Ok(ClaudeResponse::approved(request_id))
        } else {
            warn!("Validation failed for: {}", file_path.display());

            // Convert issues to protocol format
            let issues: Vec<IssueDetail> = result.issues
                .clone()
                .into_iter()
                .map(|i| i.into())
                .collect();

            // Check if we have errors (not just warnings)
            let has_errors = issues.iter()
                .any(|i| i.level == crate::protocol::IssueSeverity::Error);

            // Try auto-fix if enabled
            let modified_content = if let Some(fixer) = autofixer {
                match fixer.fix_issues(content, &result.issues, language, file_path) {
                    Ok(fix_app) if fix_app.has_changes() => {
                        info!("Auto-fix applied {} corrections", fix_app.fixes_applied);
                        Some(fix_app.modified)
                    }
                    Ok(_) => None,
                    Err(e) => {
                        debug!("Auto-fix failed: {}", e);
                        None
                    }
                }
            } else {
                None
            };

            if has_errors {
                // If we have a modified content from auto-fix, include it
                if let Some(modified) = modified_content {
                    Ok(ClaudeResponse::approved_with_fix(request_id, modified))
                } else {
                    Ok(ClaudeResponse::rejected(
                        request_id,
                        format!("Validation failed with {} error(s)", issues.len()),
                        issues,
                    ))
                }
            } else {
                // Warnings only - allow with warning response
                Ok(ClaudeResponse::warning(
                    request_id,
                    format!("Validation passed with {} warning(s)", issues.len()),
                    issues,
                ))
            }
        }
    }

    /// Handle edit file request
    fn handle_edit_file(
        request: crate::protocol::EditFileRequest,
        validator: &ValidatorEngine,
        config: &WrapConfig,
        autofixer: &Option<AutoFixer>,
    ) -> Result<ClaudeResponse> {
        let file_path = &request.path;
        let new_content = &request.new_string;
        let request_id = &request.id;

        info!("Validating edit to: {}", file_path.display());

        // Check path validation first
        let wrapper = super::StdioWrapper::new(config.clone());
        if let Err(e) = wrapper.validate_path(file_path) {
            warn!("Path validation failed: {}", e);
            return Ok(ClaudeResponse::rejected(
                request_id,
                format!("Path not allowed: {}", e),
                vec![],
            ));
        }

        // Detect language from file extension
        let language = Self::detect_language(file_path);
        debug!("Detected language: {:?}", language);

        // Run validation on the new content
        let result = validator.validate_string(new_content, language, file_path);

        if result.passed {
            info!("Edit validation passed for: {}", file_path.display());
            Ok(ClaudeResponse::approved(request_id))
        } else {
            warn!("Edit validation failed for: {}", file_path.display());

            let issues: Vec<IssueDetail> = result.issues
                .clone()
                .into_iter()
                .map(|i| i.into())
                .collect();

            let has_errors = issues.iter()
                .any(|i| i.level == crate::protocol::IssueSeverity::Error);

            // Try auto-fix if enabled
            let modified_content = if let Some(fixer) = autofixer {
                match fixer.fix_issues(new_content, &result.issues, language, file_path) {
                    Ok(fix_app) if fix_app.has_changes() => {
                        info!("Auto-fix applied {} corrections", fix_app.fixes_applied);
                        Some(fix_app.modified)
                    }
                    Ok(_) => None,
                    Err(e) => {
                        debug!("Auto-fix failed: {}", e);
                        None
                    }
                }
            } else {
                None
            };

            if has_errors {
                // If we have a modified content from auto-fix, include it
                if let Some(modified) = modified_content {
                    Ok(ClaudeResponse::approved_with_fix(request_id, modified))
                } else {
                    Ok(ClaudeResponse::rejected(
                        request_id,
                        format!("Edit validation failed with {} error(s)", issues.len()),
                        issues,
                    ))
                }
            } else {
                Ok(ClaudeResponse::warning(
                    request_id,
                    format!("Edit validation passed with {} warning(s)", issues.len()),
                    issues,
                ))
            }
        }
    }

    /// Detect programming language from file extension
    fn detect_language(path: &Path) -> Language {
        Language::from_path(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_language_detection() {
        assert_eq!(
            ClaudeWrapper::detect_language(Path::new("test.ts")),
            Language::TypeScript
        );
        assert_eq!(
            ClaudeWrapper::detect_language(Path::new("test.js")),
            Language::JavaScript
        );
        assert_eq!(
            ClaudeWrapper::detect_language(Path::new("test.rs")),
            Language::Rust
        );
    }

    #[test]
    fn test_ping_request() {
        // Verify ping returns pong
        let response = ClaudeWrapper::handle_request(
            ClaudeRequest::Ping,
            &ValidatorEngine::new(),
            &WrapConfig::default(),
            &None,
        ).unwrap();

        assert!(matches!(response, ClaudeResponse::Pong));
    }
}
