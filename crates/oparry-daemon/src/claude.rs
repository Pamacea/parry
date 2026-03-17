//! Claude Code session detection

use oparry_core::Result;
use std::path::{Path, PathBuf};
use sysinfo::{Process, System};
use walkdir::WalkDir;

/// Detected Claude Code session
#[derive(Debug, Clone)]
pub struct ClaudeSession {
    /// Session ID (PID)
    pub id: u32,
    /// Working directory
    pub work_dir: PathBuf,
    /// Repository being worked on
    pub repository: Option<PathBuf>,
    /// Active files being edited
    pub active_files: Vec<PathBuf>,
}

/// Detects Claude Code sessions
pub struct SessionDetector {
    claude_processes: Vec<String>,
}

impl SessionDetector {
    /// Create new detector
    pub fn new() -> Self {
        Self {
            claude_processes: vec![
                "claude-code".to_string(),
                "claude".to_string(),
                "anthropic-claude".to_string(),
            ],
        }
    }

    /// Detect all active Claude Code sessions
    pub async fn detect_sessions(&self) -> Result<Vec<ClaudeSession>> {
        let mut sessions = Vec::new();
        let mut sys = System::new_all();
        sys.refresh_all();

        for (pid, process) in sys.processes() {
            let name = process.name();
            if self.is_claude_process(name) {
                let pid_u32 = pid.as_u32();
                if let Ok(session) = self.create_session(pid_u32, process).await {
                    sessions.push(session);
                }
            }
        }

        Ok(sessions)
    }

    /// Check if process is Claude Code
    fn is_claude_process(&self, name: &str) -> bool {
        let name_lower = name.to_lowercase();
        self.claude_processes.iter().any(|p| name_lower.contains(p))
    }

    /// Create session from process
    async fn create_session(&self, pid: u32, process: &Process) -> Result<ClaudeSession> {
        let cwd = process
            .cwd()
            .map(|p| PathBuf::from(p))
            .unwrap_or(PathBuf::from("."));

        // Detect repository
        let repository = self.detect_repository(&cwd).await;

        // Detect active files (recently modified)
        let active_files = self.detect_active_files(&cwd).await;

        Ok(ClaudeSession {
            id: pid,
            work_dir: cwd,
            repository,
            active_files,
        })
    }

    /// Detect if working directory is in a git repository
    async fn detect_repository(&self, path: &Path) -> Option<PathBuf> {
        // Walk up to find .git directory
        let mut current = Some(path.to_path_buf());

        while let Some(dir) = current {
            let git_dir = dir.join(".git");
            if git_dir.exists() {
                return Some(dir);
            }

            current = dir.parent().map(|p| p.to_path_buf());
        }

        None
    }

    /// Detect recently modified files (last 5 minutes)
    async fn detect_active_files(&self, path: &Path) -> Vec<PathBuf> {
        let mut files = Vec::new();
        let five_mins_ago = std::time::SystemTime::now() - std::time::Duration::from_secs(300);

        for entry in WalkDir::new(path)
            .follow_links(true)
            .max_depth(10)
            .into_iter()
            .filter_map(std::result::Result::ok)
        {
            if let Ok(metadata) = entry.metadata() {
                if let Ok(modified) = metadata.modified() {
                    if modified > five_mins_ago {
                        if let Some(ext) = entry.path().extension() {
                            if let Some(ext_str) = ext.to_str() {
                                if matches!(ext_str, "ts" | "tsx" | "js" | "jsx" | "rs" | "css") {
                                    files.push(entry.path().to_path_buf());
                                }
                            }
                        }
                    }
                }
            }
        }

        files.sort();
        files.truncate(10); // Keep last 10 files
        files
    }
}

impl Default for SessionDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detector_creation() {
        let detector = SessionDetector::new();
        assert_eq!(detector.claude_processes.len(), 3);
    }

    #[test]
    fn test_detector_default() {
        let detector = SessionDetector::default();
        assert_eq!(detector.claude_processes.len(), 3);
    }

    #[test]
    fn test_is_claude_process() {
        let detector = SessionDetector::new();

        assert!(detector.is_claude_process("claude-code"));
        assert!(detector.is_claude_process("Claude-Code"));
        assert!(detector.is_claude_process("CLAUDE"));
        assert!(detector.is_claude_process("anthropic-claude"));

        assert!(!detector.is_claude_process("node"));
        assert!(!detector.is_claude_process("python"));
        assert!(!detector.is_claude_process(""));
    }

    #[test]
    fn test_claude_session_creation() {
        let session = ClaudeSession {
            id: 12345,
            work_dir: PathBuf::from("/home/user/project"),
            repository: Some(PathBuf::from("/home/user/project")),
            active_files: vec![PathBuf::from("/home/user/project/src/main.rs")],
        };

        assert_eq!(session.id, 12345);
        assert_eq!(session.work_dir, PathBuf::from("/home/user/project"));
        assert!(session.repository.is_some());
        assert_eq!(session.active_files.len(), 1);
    }

    #[test]
    fn test_claude_session_empty() {
        let session = ClaudeSession {
            id: 1,
            work_dir: PathBuf::from("."),
            repository: None,
            active_files: vec![],
        };

        assert!(session.repository.is_none());
        assert!(session.active_files.is_empty());
    }
}
