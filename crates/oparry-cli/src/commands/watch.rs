//! Watch command - continuous validation

use crate::CliContext;
use oparry_core::Result;
use oparry_watcher::FileWatcher;
use std::path::PathBuf;
use std::time::Duration;

/// Watch command
pub struct WatchCommand {
    /// Paths to watch
    paths: Vec<PathBuf>,
    /// Debounce delay
    debounce: Duration,
    /// Clear screen between runs
    clear: bool,
}

impl WatchCommand {
    /// Create new watch command
    pub fn new(paths: Vec<PathBuf>, debounce: Duration, clear: bool) -> Self {
        Self {
            paths,
            debounce,
            clear,
        }
    }

    /// Run the watch command
    pub fn run(&self, _ctx: &CliContext) -> Result<()> {
        use oparry_watcher::WatchConfig;

        let config = WatchConfig {
            debounce: self.debounce,
            paths: self.paths.clone(),
            filters: vec![
                "*.ts".to_string(),
                "*.tsx".to_string(),
                "*.js".to_string(),
                "*.jsx".to_string(),
                "*.rs".to_string(),
            ],
        };

        let mut watcher = FileWatcher::new(config)?;

        for path in &self.paths {
            watcher.watch(path)?;
        }

        println!("🔍 Watching files. Press Ctrl+C to stop.");
        println!();

        loop {
            match watcher.next_event() {
                Ok(event) => {
                    if self.clear {
                        print!("\x1b[2J\x1b[1;1H");
                    }

                    println!("Changes detected:");
                    for path in &event.paths {
                        println!("  - {}", path.display());
                    }
                    println!();

                    // Run validation on changed files
                    for path in &event.paths {
                        // For now, just indicate validation would run
                        println!("Validating: {}", path.display());
                    }
                }
                Err(e) => {
                    eprintln!("Watch error: {}", e);
                }
            }
        }
    }
}
