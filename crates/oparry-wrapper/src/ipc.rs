//! IPC (Inter-Process Communication) for Claude Code wrapper
//!
//! This module handles communication between Claude Code and Parry
//! using stdin/stdout JSON protocol.

use crate::protocol::{ClaudeRequest, ClaudeResponse};
use oparry_core::{Error, Result};
use std::io::{self, BufRead, BufReader, Write};
use std::sync::{Arc, Mutex};
use tracing::{debug, error, info, trace, warn};

/// IPC channel for Claude Code communication
pub struct IpcChannel {
    /// Input reader (stdin)
    reader: Arc<Mutex<BufReader<io::Stdin>>>,
    /// Output writer (stdout)
    writer: Arc<Mutex<io::Stdout>>,
    /// Whether we're running in interactive mode
    interactive: bool,
}

impl Clone for IpcChannel {
    fn clone(&self) -> Self {
        Self {
            reader: Arc::clone(&self.reader),
            writer: Arc::clone(&self.writer),
            interactive: self.interactive,
        }
    }
}

impl IpcChannel {
    /// Create a new IPC channel using stdin/stdout
    pub fn stdio() -> Self {
        Self {
            reader: Arc::new(Mutex::new(BufReader::new(io::stdin()))),
            writer: Arc::new(Mutex::new(io::stdout())),
            interactive: true,
        }
    }

    /// Create a non-interactive channel for testing
    pub fn buffered() -> Self {
        Self {
            reader: Arc::new(Mutex::new(BufReader::new(io::stdin()))),
            writer: Arc::new(Mutex::new(io::stdout())),
            interactive: false,
        }
    }

    /// Read a single JSON request from stdin
    ///
    /// Expected format: newline-delimited JSON
    pub fn read_request(&self) -> Result<Option<ClaudeRequest>> {
        let mut reader = self.reader.lock()
            .map_err(|e| Error::Wrapper(format!("Failed to lock reader: {}", e)))?;

        let mut line = String::new();
        let bytes_read = reader.read_line(&mut line)
            .map_err(|e| Error::Wrapper(format!("Failed to read from stdin: {}", e)))?;

        if bytes_read == 0 {
            // EOF - clean shutdown
            debug!("Received EOF on stdin, shutting down");
            return Ok(None);
        }

        let line = line.trim();
        if line.is_empty() {
            // Skip empty lines
            trace!("Skipping empty line");
            return Ok(None);
        }

        debug!("Received request: {}", line);

        // Parse JSON request
        let request = ClaudeRequest::from_json(line)?;
        Ok(Some(request))
    }

    /// Send a response to Claude Code via stdout
    pub fn send_response(&self, response: &ClaudeResponse) -> Result<()> {
        let json = response.to_json()?;
        let output = format!("{}\n", json);

        debug!("Sending response: {}", json);

        let mut writer = self.writer.lock()
            .map_err(|e| Error::Wrapper(format!("Failed to lock writer: {}", e)))?;

        writer.write_all(output.as_bytes())
            .map_err(|e| Error::Wrapper(format!("Failed to write to stdout: {}", e)))?;
        writer.flush()
            .map_err(|e| Error::Wrapper(format!("Failed to flush stdout: {}", e)))?;

        Ok(())
    }

    /// Enter the main IPC loop
    ///
    /// This blocks and handles requests until EOF is received
    pub fn run_loop<F>(self, mut handler: F) -> Result<()>
    where
        F: FnMut(ClaudeRequest) -> Result<ClaudeResponse>,
    {
        info!("Starting IPC loop in {} mode",
            if self.interactive { "interactive" } else { "buffered" });

        loop {
            match self.read_request() {
                Ok(Some(request)) => {
                    trace!("Processing request: {:?}", request.id());

                    let response = handler(request.clone());

                    match response {
                        Ok(resp) => {
                            if let Err(e) = self.send_response(&resp) {
                                error!("Failed to send response: {}", e);
                                // Try to send error response
                                let error_resp = ClaudeResponse::protocol_error(format!("Internal error: {}", e));
                                let _ = self.send_response(&error_resp);
                                return Err(e);
                            }
                        }
                        Err(e) => {
                            // Send protocol error for handler failures
                            warn!("Handler failed: {}", e);
                            let error_resp = ClaudeResponse::protocol_error(format!("Handler error: {}", e));
                            if let Err(send_err) = self.send_response(&error_resp) {
                                error!("Failed to send error response: {}", send_err);
                                return Err(e);
                            }
                        }
                    }
                }
                Ok(None) => {
                    // Empty line or EOF
                    if self.reader.lock()
                        .map(|mut r| r.fill_buf().map(|b| b.is_empty()).unwrap_or(false))
                        .unwrap_or(false)
                    {
                        info!("EOF detected, exiting IPC loop");
                        break;
                    }
                    continue;
                }
                Err(e) => {
                    error!("Failed to read request: {}", e);
                    let error_resp = ClaudeResponse::protocol_error(format!("Parse error: {}", e));
                    self.send_response(&error_resp)?;
                    return Err(e);
                }
            }
        }

        info!("IPC loop terminated cleanly");
        Ok(())
    }

    /// Check if this is an interactive channel
    pub fn is_interactive(&self) -> bool {
        self.interactive
    }
}

impl Default for IpcChannel {
    fn default() -> Self {
        Self::stdio()
    }
}

/// Simple in-memory channel for testing
#[cfg(test)]
pub struct MockIpcChannel {
    pub received_requests: Arc<Mutex<Vec<ClaudeRequest>>>,
    pub responses_to_send: Arc<Mutex<Vec<ClaudeResponse>>>,
}

#[cfg(test)]
impl MockIpcChannel {
    pub fn new() -> Self {
        Self {
            received_requests: Arc::new(Mutex::new(Vec::new())),
            responses_to_send: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn queue_response(&self, response: ClaudeResponse) {
        self.responses_to_send.lock().unwrap().push(response);
    }

    pub fn take_received(&self) -> Vec<ClaudeRequest> {
        let mut reqs = self.received_requests.lock().unwrap();
        std::mem::take(&mut *reqs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{WriteFileRequest, IssueDetail, IssueSeverity};
    use std::path::PathBuf;

    #[test]
    fn test_request_serialization_roundtrip() {
        let request = ClaudeRequest::WriteFile(WriteFileRequest {
            id: "test-1".to_string(),
            path: PathBuf::from("src/test.ts"),
            content: "export const test = true;".to_string(),
            encoding: Some("utf-8".to_string()),
            create_dirs: Some(true),
        });

        let json = request.to_json().unwrap();
        let parsed = ClaudeRequest::from_json(&json).unwrap();

        match parsed {
            ClaudeRequest::WriteFile(w) => {
                assert_eq!(w.id, "test-1");
                assert_eq!(w.path, PathBuf::from("src/test.ts"));
                assert_eq!(w.content, "export const test = true;");
            }
            _ => panic!("Wrong request type"),
        }
    }

    #[test]
    fn test_response_serialization() {
        let response = ClaudeResponse::rejected(
            "req-1",
            "Validation failed",
            vec![
                IssueDetail {
                    code: "test-error".to_string(),
                    level: IssueSeverity::Error,
                    message: "Test error".to_string(),
                    line: Some(10),
                    column: Some(5),
                    suggestion: Some("Fix it".to_string()),
                    context: None,
                }
            ],
        );

        let json = response.to_json().unwrap();
        assert!(json.contains("rejected"));
        assert!(json.contains("test-error"));
        assert!(json.contains("Validation failed"));

        // Parse it back
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["type"], "rejected");
        assert_eq!(parsed["request_id"], "req-1");
    }

    #[test]
    fn test_mock_channel() {
        let channel = MockIpcChannel::new();

        // Queue some responses
        channel.queue_response(ClaudeResponse::Pong);
        channel.queue_response(ClaudeResponse::approved("test-1"));

        assert_eq!(channel.responses_to_send.lock().unwrap().len(), 2);
    }
}
