//! Install command - One-command setup for Parry

use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use crate::CliContext;

/// Install command
pub struct InstallCommand {
    /// Force reinstall
    pub force: bool,
    /// Global install (for all projects)
    pub global: bool,
    /// Skip daemon
    pub no_daemon: bool,
}

impl InstallCommand {
    pub fn new(force: bool, global: bool, no_daemon: bool) -> Self {
        Self {
            force,
            global,
            no_daemon,
        }
    }

    pub fn run(&self, _ctx: &CliContext) -> Result<()> {
        println!("🚀 Installing Parry...\n");

        // Step 1: Create directories
        self.create_directories()?;

        // Step 2: Create global config
        self.create_global_config()?;

        // Step 3: Create Claude Code hook
        self.create_hook()?;

        // Step 4: Register hook in settings.json
        self.register_hook()?;

        // Step 5: Install daemon service
        if !self.no_daemon {
            self.install_daemon()?;
        }

        println!("\n✅ Parry installed successfully!");
        println!("\nNext steps:");
        println!("  1. Restart Claude Code");
        println!("  2. Start coding - Parry will validate automatically!");
        println!("\nCommands:");
        println!("  parry status      - Check daemon status");
        println!("  parry check <file> - Manually validate a file");
        println!("  parry config       - Manage configuration");

        Ok(())
    }

    fn create_directories(&self) -> Result<()> {
        println!("📁 Creating directories...");

        let home = dirs::home_dir().context("Cannot find home directory")?;

        let dirs = [
            home.join(".config").join("parry"),
            home.join(".parry").join("hooks"),
            home.join(".claude").join("plugins"),
        ];

        for dir in &dirs {
            if !dir.exists() {
                fs::create_dir_all(dir)
                    .with_context(|| format!("Failed to create directory: {:?}", dir))?;
                println!("  ✓ Created: {}", dir.display());
            }
        }

        Ok(())
    }

    fn create_global_config(&self) -> Result<()> {
        println!("📝 Creating global config...");

        let home = dirs::home_dir().context("Cannot find home directory")?;
        let config_dir = home.join(".config").join("parry");
        let config_path = config_dir.join("config.toml");

        if config_path.exists() && !self.force {
            println!("  ℹ️  Config already exists, skipping (use --force to overwrite)");
            return Ok(());
        }

        let config = r#"# Parry Global Configuration
# This config applies to all projects unless overridden locally

[general]
# Auto-detect project type and apply appropriate validators
auto_detect = true
# Fail validation on first error
fail_fast = false
# Maximum issues to report per file
max_issues = 100

# Auto-fix settings
[auto_fix]
# Enable auto-fix for safe issues
enabled = true
# Maximum fixes to apply per file
max_fixes = 50
# Preserve code formatting when fixing
preserve_formatting = true

[output]
# Output format: human, json, sarif
format = "human"
# Show file paths in output
show_paths = true
# Color output: auto, always, never
color = "auto"

# Validation presets - enable all by default when auto_detect is true
[presets]
# Enable all validators for TypeScript/React projects
typescript_react = [
    "tailwind",
    "imports",
    "react",
    "accessibility",
    "security",
    "performance",
    "typescript",
]

# Enable all validators for Rust projects
rust = [
    "rust",
]

# Enable all validators for Python projects
python = [
    "python",  # TODO: implement
]
"#;

        fs::write(&config_path, config)
            .context("Failed to write config")?;

        println!("  ✓ Created: {}", config_path.display());
        Ok(())
    }

    fn create_hook(&self) -> Result<()> {
        println!("🪝 Creating Claude Code hook...");

        // Read hooks from integrations directory
        let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let project_root = manifest_dir.parent().unwrap().parent().unwrap();

        let hooks = [
            ("oparryd-hook.cjs", "PreToolUse"),
            ("oparryd-start.cjs", "SessionStart"),
        ];

        let home = dirs::home_dir().context("Cannot find home directory")?;
        let hook_dir = home.join(".claude").join("plugins").join("parry");

        // Create plugin directory
        fs::create_dir_all(&hook_dir)
            .context("Failed to create plugin directory")?;

        for (hook_file_name, hook_type) in hooks {
            let hook_file = project_root.join("integrations").join(hook_file_name);

            if !hook_file.exists() {
                println!("  ⚠️  {} not found, skipping", hook_file_name);
                continue;
            }

            let hook_source = fs::read_to_string(&hook_file)
                .with_context(|| format!("Failed to read {}", hook_file_name))?;

            let hook_path = hook_dir.join(hook_file_name);

            if hook_path.exists() && !self.force {
                println!("  ℹ️  {} already exists, skipping", hook_file_name);
                continue;
            }

            fs::write(&hook_path, hook_source)
                .with_context(|| format!("Failed to write {}", hook_file_name))?;

            // Make hook executable (on Unix)
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&hook_path)?.permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&hook_path, perms)?;
            }

            println!("  ✓ Created: {} ({})", hook_path.display(), hook_type);
        }

        Ok(())
    }

    fn register_hook(&self) -> Result<()> {
        println!("🔗 Registering hooks in Claude Code...");

        let home = dirs::home_dir().context("Cannot find home directory")?;
        let settings_path = home.join(".claude").join("settings.json");

        // Read existing settings or create new
        let mut settings: serde_json::Value = if settings_path.exists() {
            let content = fs::read_to_string(&settings_path)
                .context("Failed to read settings.json")?;
            serde_json::from_str(&content)
                .context("Failed to parse settings.json")?
        } else {
            serde_json::json!({})
        };

        let settings_obj = settings.as_object_mut().unwrap();

        // Get or create hooks object
        let hooks = settings_obj.entry("hooks")
            .or_insert_with(|| serde_json::json!({}))
            .as_object_mut()
            .unwrap();

        // Define hooks to register
        let hook_path = if cfg!(windows) {
            r#"node ~/.claude/plugins/parry/oparryd-hook.cjs"#
        } else {
            r#"node ~/.claude/plugins/parry/oparryd-hook.cjs"#
        };

        let start_hook_path = if cfg!(windows) {
            r#"node ~/.claude/plugins/parry/oparryd-start.cjs"#
        } else {
            r#"node ~/.claude/plugins/parry/oparryd-start.cjs"#
        };

        // Register PreToolUse hook for validation
        let pre_tool_use = hooks.entry("PreToolUse")
            .or_insert_with(|| serde_json::json!([]))
            .as_array_mut()
            .unwrap();

        let mut pre_tool_registered = false;
        for hook in pre_tool_use.iter() {
            if let Some(command) = hook.get("command") {
                if command.as_str() == Some(hook_path) {
                    pre_tool_registered = true;
                    println!("  ℹ️  PreToolUse hook already registered");
                    break;
                }
            }
        }

        if !pre_tool_registered {
            pre_tool_use.push(serde_json::json!({
                "matcher": "text_editor",
                "hooks": [{
                    "type": "command",
                    "command": hook_path,
                    "timeout": 30
                }]
            }));
            println!("  ✓ Registered PreToolUse hook");
        }

        // Register SessionStart hook for daemon auto-start
        let session_start = hooks.entry("SessionStart")
            .or_insert_with(|| serde_json::json!([]))
            .as_array_mut()
            .unwrap();

        let mut session_start_registered = false;
        for hook in session_start.iter() {
            if let Some(command) = hook.get("command") {
                if command.as_str() == Some(start_hook_path) {
                    session_start_registered = true;
                    println!("  ℹ️  SessionStart hook already registered");
                    break;
                }
            }
        }

        if !session_start_registered {
            session_start.push(serde_json::json!({
                "matcher": "*",
                "hooks": [{
                    "type": "command",
                    "command": start_hook_path,
                    "timeout": 10
                }]
            }));
            println!("  ✓ Registered SessionStart hook");
        }

        // Write back settings
        let settings_json = serde_json::to_string_pretty(&settings)?;
        fs::write(&settings_path, settings_json)
            .context("Failed to write settings.json")?;

        println!("  ✓ Hooks registered in {}", settings_path.display());
        Ok(())
    }

    fn install_daemon(&self) -> Result<()> {
        println!("🔧 Setting up daemon...");

        // Check if parryd exists
        let parryd_path = if cfg!(windows) {
            "parryd.exe"
        } else {
            "parryd"
        };

        // Try to run parryd to verify it works
        let test_result = std::process::Command::new(parryd_path)
            .arg("--version")
            .output();

        if test_result.is_err() {
            println!("  ⚠️  Daemon not found. Run: cargo build --release --workspace");
            println!("  ℹ️  You can start the daemon manually later with: parryd run");
            return Ok(());
        }

        println!("  ✓ Daemon available");
        println!("  ℹ️  Start the daemon with: parryd run");
        println!("  ℹ️  Or install as a service: parryd install --service (TODO)");

        Ok(())
    }
}
