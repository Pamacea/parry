//! Core error types for Parry

use std::path::PathBuf;

/// Result type alias
pub type Result<T> = std::result::Result<T, Error>;

/// Core error type
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// File I/O error
    #[error("File error: {path} - {source}")]
    File {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Parsing error
    #[error("Parse error in {file}:{line}:{column}: {message}")]
    Parse {
        file: String,
        line: usize,
        column: usize,
        message: String,
    },

    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),

    /// Watcher error
    #[error("Watcher error: {0}")]
    Watcher(String),

    /// Wrapper error
    #[error("Wrapper error: {0}")]
    Wrapper(String),

    /// JSON error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// TOML error
    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),

    /// Regex error
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),

    /// Generic error
    #[error("{0}")]
    Other(String),
}

impl Error {
    /// Create a file error
    pub fn file(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::File {
            path: path.into(),
            source,
        }
    }

    /// Create a parse error
    pub fn parse(file: impl Into<String>, line: usize, column: usize, message: impl Into<String>) -> Self {
        Self::Parse {
            file: file.into(),
            line,
            column,
            message: message.into(),
        }
    }

    /// Create a validation error
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::validation("test error");
        assert_eq!(err.to_string(), "Validation error: test error");
    }

    #[test]
    fn test_error_config() {
        let err = Error::Config("invalid config".to_string());
        assert_eq!(err.to_string(), "Configuration error: invalid config");
    }

    #[test]
    fn test_error_file() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = Error::file("/path/to/file", io_err);
        assert!(err.to_string().contains("File error"));
        assert!(err.to_string().contains("/path/to/file"));
    }

    #[test]
    fn test_error_parse() {
        let err = Error::parse("test.js", 10, 5, "unexpected token");
        assert_eq!(err.to_string(), "Parse error in test.js:10:5: unexpected token");
    }

    #[test]
    fn test_error_validation() {
        let err = Error::validation("invalid input");
        assert_eq!(err.to_string(), "Validation error: invalid input");
    }

    #[test]
    fn test_error_watcher() {
        let err = Error::Watcher("watcher failed".to_string());
        assert_eq!(err.to_string(), "Watcher error: watcher failed");
    }

    #[test]
    fn test_error_wrapper() {
        let err = Error::Wrapper("wrap failed".to_string());
        assert_eq!(err.to_string(), "Wrapper error: wrap failed");
    }

    #[test]
    fn test_error_other() {
        let err = Error::Other("something happened".to_string());
        assert_eq!(err.to_string(), "something happened");
    }

    #[test]
    fn test_error_static_creation() {
        assert!(matches!(Error::validation("test"), Error::Validation(_)));
        assert!(matches!(Error::parse("", 0, 0, ""), Error::Parse { .. }));
    }
}
