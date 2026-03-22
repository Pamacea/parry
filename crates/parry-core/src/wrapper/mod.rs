//! Synchronous wrapper for intercepting file writes
//!
//! This module provides a simple, synchronous wrapper mode that:
//! 1. Spawns a child process
//! 2. Watches for file changes
//! 3. Validates changed files
//! 4. Optionally reports violations

use crate::{Config, Result, validators::Validators, parser::parser_for_path};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use notify::{RecursiveMode, recommended_watcher, EventKind};
use notify::Watcher as NotifyWatcher;
use std::sync::mpsc::channel;

/// Configuration for the wrapper mode
#[derive(Debug, Clone)]
pub struct WrapperConfig {
    /// Whether to block writes that violate rules (not fully implemented yet)
    pub block: bool,
    /// Paths to exclude from watching
    pub exclude_paths: Vec<String>,
    /// Debounce delay for file events
    pub debounce_ms: u64,
}

impl Default for WrapperConfig {
    fn default() -> Self {
        Self {
            block: false,
            exclude_paths: vec![
                "node_modules".to_string(),
                ".git".to_string(),
                "target".to_string(),
                "dist".to_string(),
                "build".to_string(),
                ".next".to_string(),
            ],
            debounce_ms: 100,
        }
    }
}

/// Synchronous wrapper that runs a command and validates file writes
pub struct SyncWrapper {
    config: WrapperConfig,
    parry_config: Config,
    validators: Validators,
}

impl SyncWrapper {
    pub fn new(wrapper_config: WrapperConfig, parry_config: Config) -> Self {
        Self {
            config: wrapper_config,
            parry_config,
            validators: Validators::new(),
        }
    }

    /// Set validators to use
    pub fn with_validators(mut self, validators: Validators) -> Self {
        self.validators = validators;
        self
    }

    /// Run a command with file write interception
    pub fn run(&self, command: &str, args: &[String]) -> Result<i32> {
        tracing::info!("Starting wrapper for: {} {}", command, args.join(" "));

        // Spawn the child process
        let mut child = Command::new(command)
            .args(args)
            .spawn()
            .map_err(|e| crate::Error::Watcher(format!("Failed to spawn command: {}", e)))?;

        // Create a channel for file events
        let (tx, rx) = channel();

        // Create watcher - store it to keep it alive during the loop
        let mut watcher = recommended_watcher(move |res| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        })
        .map_err(|e| crate::Error::Watcher(format!("Failed to create watcher: {}", e)))?;

        // Watch current directory
        use notify::Watcher as _;
        watcher.watch(Path::new("."), RecursiveMode::Recursive)
            .map_err(|e| crate::Error::Watcher(format!("Failed to watch directory: {}", e)))?;

        // Collect violations
        let violations: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let exit_code = loop {
            // Check if child has exited
            match child.try_wait() {
                Ok(Some(status)) => {
                    break status.code().unwrap_or(0);
                }
                Ok(None) => {
                    // Child still running, process events
                }
                Err(e) => {
                    tracing::error!("Error waiting for child process: {}", e);
                    break 1;
                }
            }

            // Process file events with timeout
            let has_event = rx.recv_timeout(Duration::from_millis(self.config.debounce_ms)).is_ok();

            if has_event {
                // Debounce: wait for more events
                std::thread::sleep(Duration::from_millis(self.config.debounce_ms));

                // Drain remaining events
                while let Ok(event) = rx.try_recv() {
                    tracing::debug!("File event: {:?}", event);

                    // Validate changed files
                    for path in event.paths {
                        if let Some(path_str) = path.to_str() {
                            if !self.should_exclude(path_str) && self.is_valid_extension(path_str) {
                                if let Err(e) = self.validate_file(&path, &violations) {
                                    tracing::warn!("Failed to validate {:?}: {}", path, e);
                                }
                            }
                        }
                    }
                }
            }
        };

        // Report violations
        let violations = violations.lock().unwrap();
        if !violations.is_empty() {
            eprintln!("\n🔴 Parry found {} issue(s):\n", violations.len());
            for violation in &*violations {
                eprintln!("  {}", violation);
            }
        }

        Ok(exit_code)
    }

    fn should_exclude(&self, path: &str) -> bool {
        self.config.exclude_paths.iter().any(|exclude| path.contains(exclude))
    }

    fn is_valid_extension(&self, path: &str) -> bool {
        path.ends_with(".ts") || path.ends_with(".tsx") ||
        path.ends_with(".js") || path.ends_with(".jsx") ||
        path.ends_with(".rs")
    }

    fn validate_file(&self, path: &Path, violations: &Arc<Mutex<Vec<String>>>) -> Result<()> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| crate::Error::Wrapper(format!("Failed to read {}: {}", path.display(), e)))?;

        // Parse the file
        let parser = parser_for_path(path);
        let parsed = parser.parse(&content)?;

        // Validate
        let result = self.validators.validate(&parsed, path)?;

        // Collect violations
        let mut violations_guard = violations.lock().unwrap();
        for issue in &result.issues {
            violations_guard.push(format!(
                "{}:{}: {} - {}",
                path.display(),
                issue.line.unwrap_or(0),
                issue.level,
                issue.message
            ));
        }

        Ok(())
    }
}
