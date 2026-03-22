//! Watch command - continuous validation

use oalacea_parry_core::{Config, watcher::FileWatcher, watcher::WatchConfig};
use std::path::PathBuf;
use std::time::Duration;

/// Run the `parry watch` command
pub fn run(
    _config: Config,
    paths: Vec<PathBuf>,
    debounce: u64,
    clear: bool,
) -> anyhow::Result<()> {
    let watch_config = WatchConfig {
        debounce: Duration::from_millis(debounce),
        paths: paths.clone(),
        filters: vec![
            "*.ts".to_string(),
            "*.tsx".to_string(),
            "*.js".to_string(),
            "*.jsx".to_string(),
            "*.rs".to_string(),
        ],
    };

    let mut watcher = FileWatcher::new(watch_config)?;

    for path in &paths {
        watcher.watch(path)?;
    }

    println!("🔍 Watching files. Press Ctrl+C to stop.");
    println!();

    loop {
        match watcher.next_event() {
            Ok(event) => {
                if clear {
                    print!("\x1b[2J\x1b[1;1H");
                }

                println!("Changes detected:");
                for path in &event.paths {
                    println!("  - {}", path.display());
                }
                println!();

                // Run validation on changed files
                for path in &event.paths {
                    println!("Validating: {}", path.display());
                    // TODO: Run actual validation
                }
            }
            Err(e) => {
                eprintln!("Watch error: {}", e);
            }
        }
    }
}
