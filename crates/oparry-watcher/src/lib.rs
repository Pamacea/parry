//! File system watcher for Parry

use notify::{Watcher, RecursiveMode, Event, EventKind};
use oparry_core::{Result, Error};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

/// Watch configuration
#[derive(Debug, Clone)]
pub struct WatchConfig {
    /// Debounce delay
    pub debounce: Duration,
    /// Paths to watch
    pub paths: Vec<PathBuf>,
    /// File filters (glob patterns)
    pub filters: Vec<String>,
}

impl Default for WatchConfig {
    fn default() -> Self {
        Self {
            debounce: Duration::from_millis(300),
            paths: vec![PathBuf::from(".")],
            filters: vec![
                "*.ts".to_string(),
                "*.tsx".to_string(),
                "*.js".to_string(),
                "*.jsx".to_string(),
                "*.rs".to_string(),
            ],
        }
    }
}

/// Debounced event
#[derive(Debug, Clone)]
pub struct DebouncedEvent {
    /// Paths that changed
    pub paths: Vec<PathBuf>,
    /// Event kind
    pub kind: EventKind,
}

/// File watcher
pub struct FileWatcher {
    watcher: notify::RecommendedWatcher,
    receiver: Receiver<std::result::Result<Event, notify::Error>>,
    last_event_time: std::time::Instant,
    pending_paths: Vec<PathBuf>,
    config: WatchConfig,
}

impl FileWatcher {
    /// Create new file watcher
    pub fn new(config: WatchConfig) -> Result<Self> {
        let (tx, rx) = mpsc::channel();

        let watcher = notify::recommended_watcher(move |res| {
            let _ = tx.send(res);
        }).map_err(|e| Error::Watcher(e.to_string()))?;

        Ok(Self {
            watcher,
            receiver: rx,
            last_event_time: std::time::Instant::now(),
            pending_paths: Vec::new(),
            config,
        })
    }

    /// Add a path to watch
    pub fn watch(&mut self, path: &Path) -> Result<()> {
        self.watcher
            .watch(path, RecursiveMode::Recursive)
            .map_err(|e| Error::Watcher(e.to_string()))?;
        Ok(())
    }

    /// Get next debounced event
    pub fn next_event(&mut self) -> Result<DebouncedEvent> {
        loop {
            let event_result = self.receiver.recv().map_err(|e| Error::Watcher(e.to_string()))?;
            let event = event_result.map_err(|e| Error::Watcher(e.to_string()))?;

            // Only care about modify events
            match event.kind {
                EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                    self.pending_paths.extend(event.paths);
                    self.last_event_time = std::time::Instant::now();
                }
                _ => continue,
            }

            // Check if debounce period has passed
            if self.last_event_time.elapsed() >= self.config.debounce {
                let paths = std::mem::take(&mut self.pending_paths);
                return Ok(DebouncedEvent {
                    paths,
                    kind: event.kind,
                });
            }
        }
    }
}

/// Async file watcher
pub struct AsyncFileWatcher {
    inner: FileWatcher,
}

impl AsyncFileWatcher {
    /// Create new async file watcher
    pub fn new(config: WatchConfig) -> Result<Self> {
        Ok(Self {
            inner: FileWatcher::new(config)?,
        })
    }

    /// Watch paths and return events via channel
    pub async fn watch_paths(
        &mut self,
        paths: &[PathBuf],
    ) -> Result<Receiver<std::result::Result<DebouncedEvent, String>>> {
        let (tx, rx) = mpsc::channel();
        let mut watcher = std::mem::replace(&mut self.inner, unsafe { std::mem::zeroed() });

        // Add all paths
        for path in paths {
            watcher.watch(path)?;
        }

        // Spawn background thread
        std::thread::spawn(move || {
            loop {
                match watcher.next_event() {
                    Ok(event) => {
                        if tx.send(Ok(event)).is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(e.to_string()));
                        break;
                    }
                }
            }
        });

        Ok(rx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watch_config_default() {
        let config = WatchConfig::default();
        assert_eq!(config.debounce, Duration::from_millis(300));
        assert_eq!(config.paths.len(), 1);
    }
}
