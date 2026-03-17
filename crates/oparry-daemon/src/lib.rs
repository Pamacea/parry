//! Parry Daemon - Auto-detection and validation for Claude Code sessions

use crate::claude::{ClaudeSession, SessionDetector};
use crate::validator::DaemonValidator;
use oparry_core::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod bridge;
pub mod claude;
pub mod config;
pub mod validator;

pub use bridge::{BridgeConfig, BridgeState, ClaudeBridge, ClaudeBridgeBuilder};

/// Daemon state
#[derive(Clone)]
pub struct DaemonState {
    pub sessions: Arc<RwLock<Vec<ClaudeSession>>>,
    pub config: config::DaemonConfig,
}

impl DaemonState {
    pub fn new(config: config::DaemonConfig) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(Vec::new())),
            config,
        }
    }

    /// Scan for Claude Code sessions
    pub async fn scan_sessions(&self) -> Result<usize> {
        let detector = SessionDetector::new();
        let sessions = detector.detect_sessions().await?;

        let mut guard = self.sessions.write().await;
        let _previous_count = guard.len();
        *guard = sessions;
        Ok(guard.len())
    }

    /// Get current sessions
    pub async fn get_sessions(&self) -> Vec<ClaudeSession> {
        self.sessions.read().await.clone()
    }
}

/// Main daemon
pub struct ParryDaemon {
    state: DaemonState,
}

impl ParryDaemon {
    /// Create new daemon
    pub fn new(config: config::DaemonConfig) -> Self {
        Self {
            state: DaemonState::new(config),
        }
    }

    /// Create with default config
    pub fn default_config() -> Result<Self> {
        Ok(Self::new(config::DaemonConfig::default()))
    }

    /// Run the daemon
    pub async fn run(&self) -> Result<()> {
        tracing::info!("🦀 Parry Daemon v{} starting", env!("CARGO_PKG_VERSION"));

        // Initial scan
        let session_count = self.state.scan_sessions().await?;
        tracing::info!("📁 Detected {} Claude Code sessions", session_count);

        // Start validation loop
        self.validation_loop().await
    }

    /// Main validation loop
    async fn validation_loop(&self) -> Result<()> {
        use tokio::time::{interval, Duration};

        let mut scan_interval = interval(Duration::from_secs(30));
        let mut validate_interval = interval(Duration::from_secs(5));

        loop {
            tokio::select! {
                _ = scan_interval.tick() => {
                    // Rescan sessions periodically
                    if let Ok(count) = self.state.scan_sessions().await {
                        tracing::debug!("Rescanned: {} sessions", count);
                    }
                }
                _ = validate_interval.tick() => {
                    // Validate active sessions
                    if let Err(e) = self.validate_sessions().await {
                        tracing::error!("Validation error: {}", e);
                    }
                }
            }
        }
    }

    /// Validate all active sessions
    async fn validate_sessions(&self) -> Result<()> {
        let sessions = self.state.get_sessions().await;
        let validator = DaemonValidator::new(self.state.config.clone());

        for session in sessions {
            if let Err(e) = validator.validate_session(&session).await {
                tracing::warn!("Failed to validate session {}: {}", session.id, e);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_daemon_creation() {
        let daemon = ParryDaemon::default_config().unwrap();
        assert_eq!(daemon.state.config.auto_validate, true);
    }
}
