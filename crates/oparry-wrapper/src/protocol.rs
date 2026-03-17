//! Protocol for Claude Code wrapper communication
//!
//! This module defines the JSON-based protocol used between
//! Claude Code and Parry for intercepting and validating file writes.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Protocol version
pub const PROTOCOL_VERSION: &str = "0.2.0";

/// Request types from Claude Code
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClaudeRequest {
    /// Request to write a file
    WriteFile(WriteFileRequest),
    /// Request to edit a file
    EditFile(EditFileRequest),
    /// Ping/heartbeat
    Ping,
}

/// Request to write a complete file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteFileRequest {
    /// Unique request ID
    pub id: String,
    /// File path to write
    pub path: PathBuf,
    /// File content
    pub content: String,
    /// File encoding (default: utf-8)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding: Option<String>,
    /// Create directories if needed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub create_dirs: Option<bool>,
}

/// Request to edit a file (partial replacement)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditFileRequest {
    /// Unique request ID
    pub id: String,
    /// File path to edit
    pub path: PathBuf,
    /// Old string to replace
    pub old_string: String,
    /// New string to replace with
    pub new_string: String,
    /// Replace all occurrences
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replace_all: Option<bool>,
}

/// Response types to Claude Code
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClaudeResponse {
    /// Validation passed - proceed with write
    Approved(ApprovedResponse),
    /// Validation failed - block or warn
    Rejected(RejectedResponse),
    /// Validation passed with warnings
    Warning(WarningResponse),
    /// Pong response
    Pong,
    /// Error in protocol handling
    ProtocolError(ProtocolErrorResponse),
}

/// Response indicating validation passed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovedResponse {
    /// Original request ID
    pub request_id: String,
    /// Optional modified content (if Parry made fixes)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_content: Option<String>,
}

/// Response indicating validation failed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RejectedResponse {
    /// Original request ID
    pub request_id: String,
    /// Error message
    pub message: String,
    /// List of validation issues
    pub issues: Vec<IssueDetail>,
    /// Whether Claude should attempt auto-fix
    pub can_autofix: bool,
}

/// Response with warnings but allowed to proceed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarningResponse {
    /// Original request ID
    pub request_id: String,
    /// Warning message
    pub message: String,
    /// List of warnings
    pub warnings: Vec<IssueDetail>,
    /// Optional modified content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_content: Option<String>,
}

/// Protocol error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolErrorResponse {
    /// Error message
    pub message: String,
    /// Error code for programmatic handling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

/// Detailed issue information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueDetail {
    /// Issue code (e.g., "tailwind-invalid-class")
    pub code: String,
    /// Severity level
    pub level: IssueSeverity,
    /// Human-readable message
    pub message: String,
    /// Line number (0-indexed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    /// Column number (0-indexed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<usize>,
    /// Suggested fix
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
    /// Context snippet
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

/// Issue severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueSeverity {
    /// Informational note
    Note,
    /// Warning (doesn't block)
    Warning,
    /// Error (blocks in strict mode)
    Error,
}

impl From<oparry_core::IssueLevel> for IssueSeverity {
    fn from(level: oparry_core::IssueLevel) -> Self {
        match level {
            oparry_core::IssueLevel::Note => IssueSeverity::Note,
            oparry_core::IssueLevel::Warning => IssueSeverity::Warning,
            oparry_core::IssueLevel::Error => IssueSeverity::Error,
        }
    }
}

impl From<oparry_core::Issue> for IssueDetail {
    fn from(issue: oparry_core::Issue) -> Self {
        Self {
            code: issue.code,
            level: issue.level.into(),
            message: issue.message,
            line: issue.line,
            column: issue.column,
            suggestion: issue.suggestion,
            context: issue.context,
        }
    }
}

impl ClaudeRequest {
    /// Parse a JSON string into a request
    pub fn from_json(json: &str) -> crate::Result<Self> {
        serde_json::from_str(json).map_err(|e| {
            oparry_core::Error::Wrapper(format!("Failed to parse request JSON: {}", e))
        })
    }

    /// Convert request to JSON
    pub fn to_json(&self) -> crate::Result<String> {
        serde_json::to_string(self).map_err(|e| {
            oparry_core::Error::Wrapper(format!("Failed to serialize request: {}", e))
        })
    }

    /// Get the request ID for tracking
    pub fn id(&self) -> Option<&str> {
        match self {
            ClaudeRequest::WriteFile(w) => Some(&w.id),
            ClaudeRequest::EditFile(e) => Some(&e.id),
            ClaudeRequest::Ping => None,
        }
    }

    /// Get the file path for this request
    pub fn path(&self) -> Option<&PathBuf> {
        match self {
            ClaudeRequest::WriteFile(w) => Some(&w.path),
            ClaudeRequest::EditFile(e) => Some(&e.path),
            ClaudeRequest::Ping => None,
        }
    }
}

impl ClaudeResponse {
    /// Convert response to JSON
    pub fn to_json(&self) -> crate::Result<String> {
        serde_json::to_string(self).map_err(|e| {
            oparry_core::Error::Wrapper(format!("Failed to serialize response: {}", e))
        })
    }

    /// Create an approved response
    pub fn approved(request_id: impl Into<String>) -> Self {
        ClaudeResponse::Approved(ApprovedResponse {
            request_id: request_id.into(),
            modified_content: None,
        })
    }

    /// Create an approved response with modified content
    pub fn approved_with_fix(request_id: impl Into<String>, content: impl Into<String>) -> Self {
        ClaudeResponse::Approved(ApprovedResponse {
            request_id: request_id.into(),
            modified_content: Some(content.into()),
        })
    }

    /// Create a rejected response
    pub fn rejected(
        request_id: impl Into<String>,
        message: impl Into<String>,
        issues: Vec<IssueDetail>,
    ) -> Self {
        ClaudeResponse::Rejected(RejectedResponse {
            request_id: request_id.into(),
            message: message.into(),
            issues,
            can_autofix: true, // By default, allow Claude to attempt fixes
        })
    }

    /// Create a warning response
    pub fn warning(
        request_id: impl Into<String>,
        message: impl Into<String>,
        warnings: Vec<IssueDetail>,
    ) -> Self {
        ClaudeResponse::Warning(WarningResponse {
            request_id: request_id.into(),
            message: message.into(),
            warnings,
            modified_content: None,
        })
    }

    /// Create a protocol error response
    pub fn protocol_error(message: impl Into<String>) -> Self {
        ClaudeResponse::ProtocolError(ProtocolErrorResponse {
            message: message.into(),
            code: None,
        })
    }

    /// Check if this response allows the write to proceed
    pub fn is_allowed(&self) -> bool {
        matches!(self, ClaudeResponse::Approved(_) | ClaudeResponse::Warning(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_file_serialization() {
        let request = ClaudeRequest::WriteFile(WriteFileRequest {
            id: "test-123".to_string(),
            path: PathBuf::from("test.ts"),
            content: "console.log('hello');".to_string(),
            encoding: Some("utf-8".to_string()),
            create_dirs: Some(true),
        });

        let json = request.to_json().unwrap();
        let parsed = ClaudeRequest::from_json(&json).unwrap();

        assert_eq!(parsed.id(), Some("test-123"));
        assert_eq!(parsed.path(), Some(&PathBuf::from("test.ts")));
    }

    #[test]
    fn test_response_creation() {
        let response = ClaudeResponse::approved("test-123");
        assert!(response.is_allowed());

        let response = ClaudeResponse::rejected(
            "test-456",
            "Validation failed",
            vec![],
        );
        assert!(!response.is_allowed());
    }

    #[test]
    fn test_issue_conversion() {
        let core_issue = oparry_core::Issue::error("test-error", "Test error message")
            .with_line(10)
            .with_column(5)
            .with_suggestion("Fix it");

        let detail: IssueDetail = core_issue.into();
        assert_eq!(detail.code, "test-error");
        assert_eq!(detail.level, IssueSeverity::Error);
        assert_eq!(detail.line, Some(10));
        assert_eq!(detail.suggestion, Some("Fix it".to_string()));
    }
}
