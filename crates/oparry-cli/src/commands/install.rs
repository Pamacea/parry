//! Install command - One-command setup for Parry

use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use std::env;
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
        println!("🚀 Installing Parry v0.2.2...\n");

        // Step 0: Pre-flight check - verify we're in the right directory
        self.preflight_check()?;

        // Step 1: Create directories
        self.create_directories()?;

        // Step 2: Create global config
        self.create_global_config()?;

        // Step 3: Create Claude Code hook
        self.create_hook()?;

        // Step 4: Clean up duplicate hooks
        self.cleanup_duplicate_hooks()?;

        // Step 5: Register hook in settings.json
        self.register_hook()?;

        // Step 6: Create PARRY.md in ~/.claude/
        self.create_parry_md()?;

        // Step 7: Inject @PARRY.md reference in ~/.claude/CLAUDE.md
        self.inject_claude_md_reference()?;

        // Step 8: Verify and report binary status
        self.verify_binaries()?;

        println!("\n✅ Parry installed successfully!");
        println!("\n📋 Next steps:");
        println!("  1. Build the binaries:");
        println!("     cargo build --release --workspace");
        println!("     (OR: cargo install --path crates/oparry-cli)");
        println!("     (OR: cargo install --path crates/oparry-daemon)");
        println!("  2. Restart Claude Code");
        println!("  3. Start coding - Parry will validate automatically!");
        println!("\n📝 Commands:");
        println!("  oparry check <file> - Manually validate a file");
        println!("  oparryd status      - Check daemon status");
        println!("  oparry config       - Manage configuration");

        Ok(())
    }

    fn preflight_check(&self) -> Result<()> {
        // Verify we're in the Parry project directory
        let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let project_root = manifest_dir.parent().and_then(|p| p.parent());

        if let Some(root) = project_root {
            let cargo_toml = root.join("Cargo.toml");
            if !cargo_toml.exists() {
                println!("  ⚠️  Warning: May not be in Parry project directory");
            } else {
                println!("  ✓ Project directory verified");
            }
        }

        Ok(())
    }

    fn create_directories(&self) -> Result<()> {
        println!("\n📁 Creating directories...");

        let home = dirs::home_dir().context("Cannot find home directory")?;

        let dirs = [
            home.join(".config").join("parry"),
            home.join(".claude").join("hooks"),  // Standard Claude Code hooks directory
        ];

        for dir in &dirs {
            if !dir.exists() {
                fs::create_dir_all(dir)
                    .with_context(|| format!("Failed to create directory: {:?}", dir))?;
                println!("  ✓ Created: {}", dir.display());
            } else {
                println!("  ✓ Exists: {}", dir.display());
            }
        }

        Ok(())
    }

    fn create_global_config(&self) -> Result<()> {
        println!("\n📝 Creating global config...");

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
        println!("\n🪝 Creating Claude Code hooks...");

        // Read hooks from integrations directory
        let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let project_root = manifest_dir.parent().unwrap().parent().unwrap();

        let hooks = [
            ("oparryd-hook.cjs", "PreToolUse"),
            ("oparryd-start.cjs", "SessionStart"),
        ];

        let home = dirs::home_dir().context("Cannot find home directory")?;
        let hook_dir = home.join(".claude").join("hooks");  // Standard hooks directory

        // Create hooks directory
        fs::create_dir_all(&hook_dir)
            .context("Failed to create hooks directory")?;

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

    /// Clean up duplicate hooks that may have been registered
    fn cleanup_duplicate_hooks(&self) -> Result<()> {
        println!("\n🧹 Checking for duplicate hooks...");

        let home = dirs::home_dir().context("Cannot find home directory")?;
        let settings_path = home.join(".claude").join("settings.json");

        if !settings_path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&settings_path)
            .context("Failed to read settings.json")?;

        let mut settings: serde_json::Value = serde_json::from_str(&content)
            .context("Failed to parse settings.json")?;

        let settings_obj = settings.as_object_mut().unwrap();

        // Get hooks object if it exists
        if let Some(hooks_value) = settings_obj.get_mut("hooks") {
            if let Some(hooks) = hooks_value.as_object_mut() {
                // Clean up PreToolUse duplicates
                if let Some(pre_tool) = hooks.get_mut("PreToolUse") {
                    if let Some(pre_tool_arr) = pre_tool.as_array_mut() {
                        let unique_hooks: Vec<serde_json::Value> = pre_tool_arr
                            .iter()
                            .filter(|hook| {
                                // Keep hooks that are NOT parry hooks (they'll be re-registered)
                                if let Some(hook_arr) = hook.get("hooks").and_then(|h| h.as_array()) {
                                    for h in hook_arr {
                                        if let Some(cmd) = h.get("command").and_then(|c| c.as_str()) {
                                            if cmd.contains("parry") {
                                        return false; // Remove old parry hooks
                                            }
                                        }
                                    }
                                }
                                true // Keep non-parry hooks
                            })
                            .cloned()
                            .collect();

                        *pre_tool_arr = unique_hooks;
                    }
                }

                // Clean up SessionStart duplicates
                if let Some(session_start) = hooks.get_mut("SessionStart") {
                    if let Some(session_start_arr) = session_start.as_array_mut() {
                        let unique_hooks: Vec<serde_json::Value> = session_start_arr
                            .iter()
                            .filter(|hook| {
                                if let Some(hook_arr) = hook.get("hooks").and_then(|h| h.as_array()) {
                                    for h in hook_arr {
                                        if let Some(cmd) = h.get("command").and_then(|c| c.as_str()) {
                                            if cmd.contains("parry") {
                                                return false; // Remove old parry hooks
                                            }
                                        }
                                    }
                                }
                                true
                            })
                            .cloned()
                            .collect();

                        *session_start_arr = unique_hooks;
                    }
                }

                // Write back cleaned settings
                let settings_json = serde_json::to_string_pretty(&settings)?;
                fs::write(&settings_path, settings_json)
                    .context("Failed to write settings.json")?;

                println!("  ✓ Cleaned up duplicate hooks");
            }
        }

        Ok(())
    }

    fn register_hook(&self) -> Result<()> {
        println!("\n🔗 Registering hooks in Claude Code...");

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

        // Use standard .claude/hooks directory
        let hook_dir = home.join(".claude").join("hooks");

        // Build hook commands with absolute paths (important for Windows)
        let hook_path = format!(
            r#"node "{}""#,
            hook_dir.join("oparryd-hook.cjs").display()
        );

        let start_hook_path = format!(
            r#"node "{}""#,
            hook_dir.join("oparryd-start.cjs").display()
        );

        // Register PreToolUse hook for validation
        let pre_tool_use = hooks.entry("PreToolUse")
            .or_insert_with(|| serde_json::json!([]))
            .as_array_mut()
            .unwrap();

        let pre_tool_registered = pre_tool_use.iter().any(|hook| {
            if let Some(hook_arr) = hook.get("hooks").and_then(|h| h.as_array()) {
                hook_arr.iter().any(|h| {
                    h.get("command")
                        .and_then(|c| c.as_str())
                        .map(|c| c.contains("oparryd-hook.cjs"))
                        .unwrap_or(false)
                })
            } else {
                false
            }
        });

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
        } else {
            println!("  ℹ️  PreToolUse hook already registered");
        }

        // Register SessionStart hook for daemon auto-start
        let session_start = hooks.entry("SessionStart")
            .or_insert_with(|| serde_json::json!([]))
            .as_array_mut()
            .unwrap();

        let session_start_registered = session_start.iter().any(|hook| {
            if let Some(hook_arr) = hook.get("hooks").and_then(|h| h.as_array()) {
                hook_arr.iter().any(|h| {
                    h.get("command")
                        .and_then(|c| c.as_str())
                        .map(|c| c.contains("oparryd-start.cjs"))
                        .unwrap_or(false)
                })
            } else {
                false
            }
        });

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
        } else {
            println!("  ℹ️  SessionStart hook already registered");
        }

        // Write back settings
        let settings_json = serde_json::to_string_pretty(&settings)?;
        fs::write(&settings_path, settings_json)
            .context("Failed to write settings.json")?;

        println!("  ✓ Hooks registered in {}", settings_path.display());
        Ok(())
    }

    fn create_parry_md(&self) -> Result<()> {
        println!("\n📄 Creating PARRY.md...");

        let home = dirs::home_dir().context("Cannot find home directory")?;
        let parry_md_path = home.join(".claude").join("PARRY.md");

        if parry_md_path.exists() && !self.force {
            println!("  ℹ️  PARRY.md already exists, skipping (use --force to overwrite)");
            return Ok(());
        }

        let parry_md = r#"# PARRY - The Agentic Linter

**Version:** 0.2.2 | **CLI:** `oparry`

## Hook-Based Usage

Parry hooks run automatically with Claude Code:
- **PreToolUse** → Validates file writes before they reach disk
- **SessionStart** → Auto-starts the Parry daemon

## Commands

```bash
oparry check <file>     # Validate a file
oparry check .          # Validate entire project
oparry watch            # Watch and validate on changes
oparry init [--stack]   # Initialize project config
oparryd status          # Check daemon status
oparryd run             # Start daemon manually
```

## Validators

| Validator | What It Checks |
|-----------|---------------|
| tailwind | Valid Tailwind classes against design system |
| imports | Alias rules, component imports (shadcn/ui) |
| react | React best practices, hook rules |
| accessibility | A11y compliance |
| security | Security vulnerabilities |
| performance | Performance anti-patterns |
| typescript | TypeScript strict mode compliance |
| rust | Rust idioms and safety |

## Configuration

- **Global:** `~/.config/parry/config.toml`
- **Project:** `.parryrc.toml` or `parry.toml`

## Environment Variables

```bash
PARRY_STRICT=1        # Strict mode (block on warnings)
PARRY_AUTO_FIX=1      # Auto-fix safe issues
PARRY_DEBUG=1         # Debug output
```

---

*Auto-generated by oparry install*
"#;

        fs::write(&parry_md_path, parry_md)
            .context("Failed to write PARRY.md")?;

        println!("  ✓ Created: {}", parry_md_path.display());
        Ok(())
    }

    fn inject_claude_md_reference(&self) -> Result<()> {
        println!("\n📝 Injecting @PARRY.md reference in CLAUDE.md...");

        let home = dirs::home_dir().context("Cannot find home directory")?;
        let claude_md_path = home.join(".claude").join("CLAUDE.md");

        if !claude_md_path.exists() {
            println!("  ⚠️  ~/.claude/CLAUDE.md not found, skipping reference injection");
            return Ok(());
        }

        let content = fs::read_to_string(&claude_md_path)
            .context("Failed to read CLAUDE.md")?;

        // Check if @PARRY.md reference already exists
        if content.contains("@PARRY.md") {
            println!("  ℹ️  @PARRY.md reference already present in CLAUDE.md");
            return Ok(());
        }

        // Append @PARRY.md reference at the end of the file
        // Try to insert near other @ references if they exist
        let new_content = if content.contains("@ARGUS.md") {
            content.replace("@ARGUS.md", "@ARGUS.md\n@PARRY.md")
        } else if content.contains("@AUREUS.md") {
            content.replace("@AUREUS.md", "@AUREUS.md\n@PARRY.md")
        } else if content.contains("@RTK.md") {
            content.replace("@RTK.md", "@RTK.md\n@PARRY.md")
        } else {
            // No existing @ references, append at end
            format!("{}\n@PARRY.md\n", content.trim_end())
        };

        fs::write(&claude_md_path, new_content)
            .context("Failed to write CLAUDE.md")?;

        println!("  ✓ Added @PARRY.md reference to CLAUDE.md");
        Ok(())
    }

    fn verify_binaries(&self) -> Result<()> {
        println!("\n🔧 Verifying binaries...");

        let home = dirs::home_dir().context("Cannot find home directory")?;
        let cargo_bin = home.join(".cargo").join("bin");

        let oparry_bin = if cfg!(windows) {
            cargo_bin.join("oparry.exe")
        } else {
            cargo_bin.join("oparry")
        };

        let parryd_bin = if cfg!(windows) {
            cargo_bin.join("oparryd.exe")
        } else {
            cargo_bin.join("oparryd")
        };

        let mut found_count = 0;

        if oparry_bin.exists() {
            println!("  ✓ oparry binary found");
            found_count += 1;
        } else {
            println!("  ⚠️  oparry NOT found at: {}", oparry_bin.display());
        }

        if parryd_bin.exists() {
            println!("  ✓ oparryd binary found");
            found_count += 1;
        } else {
            println!("  ⚠️  oparryd NOT found at: {}", parryd_bin.display());
        }

        if found_count == 0 {
            println!("\n  🔴 BINARIES NOT INSTALLED!");
            println!("  Run the following command to build and install:");
            println!("  ");
            println!("  cargo build --release --workspace");
            println!("  ");
            println!("  Then copy the binaries:");
            println!("  ");
            println!("  # On Windows:");
            println!("  copy target\\release\\oparry.exe %USERPROFILE%\\.cargo\\bin\\");
            println!("  copy target\\release\\oparryd.exe %USERPROFILE%\\.cargo\\bin\\");
            println!("  ");
            println!("  # On Unix:");
            println!("  cp target/release/oparry ~/.cargo/bin/");
            println!("  cp target/release/oparryd ~/.cargo/bin/");
        }

        Ok(())
    }
}
