//! Claude Code integration hook
//!
//! This command installs hooks that integrate Parry with Claude Code
//! by intercepting file operations through the wrapper protocol.

use clap::{Parser, Subcommand};
use oparry_core::Result;
use std::path::PathBuf;
use std::{env, fs};

/// Hook configuration
#[derive(Parser)]
pub struct HookCommand {
    #[command(subcommand)]
    pub action: HookAction,
}

#[derive(Subcommand)]
pub enum HookAction {
    /// Install Claude Code hook
    Install {
        /// Force reinstall
        #[arg(long)]
        force: bool,
    },
    /// Uninstall Claude Code hook
    Uninstall,
    /// Show hook status
    Status,
    /// Generate hook script for manual installation
    Generate {
        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

impl HookCommand {
    pub fn new(action: HookAction) -> Self {
        Self { action }
    }

    pub fn run(self) -> Result<()> {
        match self.action {
            HookAction::Install { force } => install_hook(force),
            HookAction::Uninstall => uninstall_hook(),
            HookAction::Status => show_status(),
            HookAction::Generate { output } => generate_hook_script(output),
        }
    }
}

/// Get Claude Code config directory
fn claude_config_dir() -> Result<PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| oparry_core::Error::Config("No home directory".to_string()))?;

    Ok(home.join(".claude"))
}

/// Get Parry's hooks directory
fn parry_hooks_dir() -> Result<PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| oparry_core::Error::Config("No home directory".to_string()))?;

    Ok(home.join(".parry").join("hooks"))
}

/// Install Claude Code hook
fn install_hook(force: bool) -> Result<()> {
    println!("🔗 Installing Claude Code hook...");

    let claude_dir = claude_config_dir()?;
    let hooks_dir = parry_hooks_dir()?;

    // Create directories
    fs::create_dir_all(&claude_dir)
        .map_err(|e| oparry_core::Error::File {
            path: claude_dir.clone(),
            source: e,
        })?;
    fs::create_dir_all(&hooks_dir)
        .map_err(|e| oparry_core::Error::File {
            path: hooks_dir.clone(),
            source: e,
        })?;

    // Generate hook script
    let hook_script = generate_hook_script_content()?;

    // Write hook file
    let hook_file = hooks_dir.join("claude-code-hook.sh");
    if hook_file.exists() && !force {
        println!("Hook already exists. Use --force to reinstall.");
        println!("Current location: {}", hook_file.display());
        return Ok(());
    }

    fs::write(&hook_file, &hook_script)
        .map_err(|e| oparry_core::Error::File {
            path: hook_file.clone(),
            source: e,
        })?;

    // Make executable (Unix)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&hook_file)
            .map_err(|e| oparry_core::Error::File {
                path: hook_file.clone(),
                source: e,
            })?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&hook_file, perms)
            .map_err(|e| oparry_core::Error::File {
                path: hook_file.clone(),
                source: e,
            })?;
    }

    println!("✓ Hook installed at: {}", hook_file.display());

    // Update Claude Code config
    let config_file = claude_dir.join("config.json");
    let mut config: serde_json::Value = if config_file.exists() {
        let content = fs::read_to_string(&config_file)
            .map_err(|e| oparry_core::Error::File {
                path: config_file.clone(),
                source: e,
            })?;
        serde_json::from_str(&content)
            .map_err(|e| oparry_core::Error::Config(format!("Invalid config: {}", e)))?
    } else {
        serde_json::json!({})
    };

    // Add Parry wrapper to pre-write hooks
    let hook_path = format!("{} --wrap", env::current_exe()
        .unwrap_or_else(|_| PathBuf::from("parry"))
        .display());

    if let Some(obj) = config.as_object_mut() {
        obj.entry("preWriteHooks".to_string())
            .or_insert_with(|| serde_json::json!([]))
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!({
                "command": hook_path,
                "enabled": true
            }));
    }

    // Write updated config
    let config_json = serde_json::to_string_pretty(&config)
        .map_err(|e| oparry_core::Error::Config(format!("Failed to serialize config: {}", e)))?;

    fs::write(&config_file, config_json)
        .map_err(|e| oparry_core::Error::File {
            path: config_file.clone(),
            source: e,
        })?;

    println!("✓ Claude Code config updated: {}", config_file.display());
    println!("\n🎉 Hook installed successfully!");
    println!("Parry will now validate all Claude Code writes.");
    println!("\nTo test: echo '{{\"type\":\"write_file\",\"path\":\"test.txt\",\"content\":\"hello\"}}' | parry wrap");

    Ok(())
}

/// Uninstall Claude Code hook
fn uninstall_hook() -> Result<()> {
    println!("🔌 Uninstalling Claude Code hook...");

    let claude_dir = claude_config_dir()?;
    let config_file = claude_dir.join("config.json");
    let hooks_dir = parry_hooks_dir()?;
    let hook_file = hooks_dir.join("claude-code-hook.sh");

    // Remove hook file
    if hook_file.exists() {
        fs::remove_file(&hook_file)
            .map_err(|e| oparry_core::Error::File {
                path: hook_file.clone(),
                source: e,
            })?;
        println!("✓ Removed hook file");
    }

    // Update Claude Code config to remove Parry
    if config_file.exists() {
        let content = fs::read_to_string(&config_file)
            .map_err(|e| oparry_core::Error::File {
                path: config_file.clone(),
                source: e,
            })?;

        if let Ok(mut config) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(obj) = config.as_object_mut() {
                if let Some(hooks) = obj.get_mut("preWriteHooks")
                    .and_then(|v| v.as_array_mut())
                {
                    hooks.retain(|hook| {
                        hook.get("command")
                            .and_then(|c| c.as_str())
                            .map(|c| !c.contains("parry"))
                            .unwrap_or(true)
                    });
                }
            }

            let config_json = serde_json::to_string_pretty(&config)
                .map_err(|e| oparry_core::Error::Config(format!("Failed to serialize: {}", e)))?;

            fs::write(&config_file, config_json)
                .map_err(|e| oparry_core::Error::File {
                    path: config_file.clone(),
                    source: e,
                })?;

            println!("✓ Updated Claude Code config");
        }
    }

    println!("\n🎉 Hook uninstalled successfully!");

    Ok(())
}

/// Show hook status
fn show_status() -> Result<()> {
    println!("🔍 Claude Code Hook Status");
    println!("{}\n", "=".repeat(40));

    let hooks_dir = parry_hooks_dir()?;
    let hook_file = hooks_dir.join("claude-code-hook.sh");

    if hook_file.exists() {
        println!("✓ Hook file exists: {}", hook_file.display());
    } else {
        println!("✗ Hook file not found: {}", hook_file.display());
    }

    let claude_dir = claude_config_dir()?;
    let config_file = claude_dir.join("config.json");

    if config_file.exists() {
        println!("✓ Claude Code config exists");

        let content = fs::read_to_string(&config_file)
            .map_err(|e| oparry_core::Error::File {
                path: config_file.clone(),
                source: e,
            })?;

        if let Ok(config) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(hooks) = config.get("preWriteHooks")
                .and_then(|v| v.as_array())
            {
                let parry_hook = hooks.iter()
                    .filter(|h| h.get("command")
                        .and_then(|c| c.as_str())
                        .map(|c| c.contains("parry"))
                        .unwrap_or(false))
                    .count();

                if parry_hook > 0 {
                    println!("✓ Parry hook registered in Claude Code config");
                } else {
                    println!("✗ Parry hook not found in Claude Code config");
                }
            }
        }
    } else {
        println!("✗ Claude Code config not found");
    }

    Ok(())
}

/// Generate hook script for manual installation
fn generate_hook_script(output: Option<PathBuf>) -> Result<()> {
    let script = generate_hook_script_content()?;

    let output_path = output.unwrap_or_else(|| {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("claude-code-hook.sh")
    });

    fs::write(&output_path, script)
        .map_err(|e| oparry_core::Error::File {
            path: output_path.clone(),
            source: e,
        })?;

    println!("✓ Hook script generated: {}", output_path.display());
    println!("\nTo use this hook:");
    println!("1. Make it executable: chmod + {}", output_path.display());
    println!("2. Add to Claude Code config manually");

    Ok(())
}

/// Generate the hook script content
fn generate_hook_script_content() -> Result<String> {
    Ok(r#"#!/bin/bash
# Claude Code integration hook for Parry
# This script intercepts file writes and validates them through Parry

set -euo pipefail

# Parry executable
PARRY_EXE="${PARRY_EXE:-parry}"

# Log file for debugging
PARRY_LOG="${PARRY_LOG:-${XDG_DATA_HOME:-$HOME/.local/share}/parry/hook.log}"

# Create log directory if needed
mkdir -p "$(dirname "$PARRY_LOG")"

log() {
    echo "[$(date -Iseconds)] $*" >> "$PARRY_LOG"
}

# Main hook function
parry_hook() {
    local request="$1"

    log "Processing request: ${request:0:100}..."

    # Send to Parry wrapper
    local response
    response=$(echo "$request" | "$PARRY_EXE" wrap 2>&1)

    local exit_code=$?

    if [ $exit_code -eq 0 ]; then
        # Parse response
        local allowed
        allowed=$(echo "$response" | jq -r '.type // empty')

        case "$allowed" in
            approved|warning)
                log "Request approved"
                echo "$response"
                return 0
                ;;
            rejected)
                log "Request rejected: $(echo "$response" | jq -r '.message // "Unknown error"')"
                echo "$response"
                return 1
                ;;
            *)
                log "Unknown response: $allowed"
                echo "$response"
                return 1
                ;;
        esac
    else
        log "Parry error (exit $exit_code): $response"
        return $exit_code
    fi
}

# If this script is run directly, process stdin
if [ "${BASH_SOURCE[0]}" = "$0" ]; then
    while IFS= read -r line; do
        [ -n "$line" ] || continue
        parry_hook "$line"
    done
fi
"#.to_string())
}
