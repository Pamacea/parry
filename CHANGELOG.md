# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.2] - 2026-04-02

### Fixed
- **Critical: Hook data source** — PostToolUse hook now correctly reads JSON from stdin instead of process.argv[2]
- **File extraction** — Properly extracts file path from `context.tool_input.file_path` for Write/Edit operations
- **Async structure** — Fixed callback-based async flow for proper stdin handling

## [0.3.0] - 2025-03-22

### Breaking Changes
- **Complete architecture redesign**: Multi-crate workspace (7 crates) simplified to 2 crates
- **Removed daemon**: `parryd` daemon and all IPC/async infrastructure removed
- **Removed commands**: `parryd status`, `parryd run`, `parry hook`, `parry install`
- **New package names**: CLI renamed to `oalacea-parry`, core to `oalacea-parry-core`
- **New import paths**: Use `oalacea_parry_core::` instead of `oparry_core::`

### Added
- **Synchronous wrapper mode**: `parry run <cmd>` — Run commands with file validation
- **Monolithic core library**: All modules (parser, validators, watcher, wrapper, autofix) in one crate
- **Simplified Claude Code integration**: Single `parry-post-write.cjs` hook replaces 3 separate hooks
- **Direct validation**: No daemon dependency — instant startup, no IPC overhead

### Changed
- **Architecture**: 7 crates → 2 crates (parry-core + parry)
- **Hook system**: 3 hook files → 1 unified hook file
- **Module system**: External crates → Rust modules within parry-core
- **Startup time**: Instant (no daemon spawn/check)

### Removed
- `parry-daemon` crate and binary
- IPC/sockets/gRPC communication
- `parryd` commands
- Async wrapper complexity
- Multiple hook files

### Migration from v0.2.x

```bash
# Uninstall old version
cargo uninstall parry-cli parry-daemon

# Install new version
cargo install --path crates/parry

# Update Claude Code hook
cp integrations/parry-post-write.cjs ~/.claude/hooks/
# Update ~/.claude/settings.json to use parry-post-write.cjs
```

### Technical
- Workspace members: `["crates/parry-core", "crates/parry"]`
- Binary name: `parry` (from `oalacea-parry` package)
- Core library: `oalacea-parry-core` (import as `oalacea_parry_core`)

## [0.2.3] - 2026-03-19

### Added
- **Automatic PARRY_CONFIG injection**: `oparry install` now injects `PARRY_CONFIG` environment variable into `~/.claude/settings.json` for cross-platform config discovery
- **Cross-platform config path detection**: Automatically detects correct path format for Windows (backslashes) and Unix (forward slashes)

### Fixed
- **Hook config path resolution**: Hooks now properly find Parry config via `PARRY_CONFIG` env var instead of hardcoded `.config/oparry/` path

## [0.2.2] - 2026-03-18

### Fixed
- **Missing PARRY.md**: `oparry install` now creates `~/.claude/PARRY.md` with CLI docs, validators, and config reference
- **Missing CLAUDE.md reference**: `oparry install` now injects `@PARRY.md` into `~/.claude/CLAUDE.md` (idempotent, preserves existing content)

### Changed
- Version bump from 0.2.1 to 0.2.2
- Installation process expanded from 6 to 8 steps

## [0.2.1] - 2025-03-17

### Added
- **Visible error messages** when Parry binaries are not found
- **Binary verification step** in installation process
- **Pre-flight check** to verify project directory
- **Duplicate hook cleanup** during installation
- **Absolute path support** for hooks on Windows
- Changelog tracking

### Fixed
- **Hook directory architecture**: Hooks now install to standard `.claude/hooks/` instead of `.claude/plugins/parry/`
- **Hook error visibility**: Hook now shows clear error message when `oparry` binary is missing instead of silently exiting with code 0
- **Windows path resolution**: Installer now uses absolute paths instead of `~` for Windows compatibility
- **Duplicate hooks**: Hooks in `settings.json` are now automatically cleaned up during installation
- **Daemon detection on Windows**: Fixed `tasklist` command for proper process detection
- **Daemon startup**: Uses `spawn` with `detached` option for proper background execution on all platforms

### Changed
- Version bump from 0.2.0 to 0.2.1
- Hook files renamed from `parryd-*.cjs` to `oparryd-*.cjs` for consistency
- **Hook location**: Hooks now use standard Claude Code hooks directory (`.claude/hooks/`)

### Technical
- Improved error handling in JavaScript hooks
- Better cross-platform compatibility for daemon startup
- Enhanced installation process with step-by-step verification

## [0.2.0] - 2026-03-17

### Added

#### One-Command Installation
- **`oparry install`** - Complete setup in a single command
  - Creates global configuration in `~/.config/oparry/`
  - Installs Claude Code hooks automatically
  - Registers hooks in `~/.claude/settings.json`
  - No per-project `oparry init` needed anymore

#### Claude Code Integration
- **PreToolUse Hook (`oparryd-hook.cjs`)**
  - Intercepts file writes before they hit disk
  - Validates code with `oparry check`
  - Blocks writes with errors (exit code 2)
  - Warns on warnings (exit code 1)
  - Supports auto-fix with `--fix` flag

- **SessionStart Hook (`oparryd-start.cjs`)**
  - Auto-starts daemon on Claude Code startup
  - Multi-session and multi-project support
  - Checks if daemon already running
  - Cross-platform (Windows/Unix) background execution

#### Daemon (`oparryd`)
- Background validation daemon
- Detects active Claude Code sessions
- Multi-project support
- IPC bridge for real-time validation
- Session scanning with `oparryd scan`

#### Validators
- **Tailwind Validator** - Blocks invalid width classes (`w-xl`, `w-2xl`, `max-w-md`, etc.)
- **Import Validator** - Enforces path alias rules
- **React Validator** - Component structure and fragments
- **Rust Validator** - Idiomatic Rust patterns
- **CSS Validator** - Unit handling and best practices
- **Accessibility Validator** - ARIA attributes and semantic HTML
- **Security Validator** - Common security patterns
- **Performance Validator** - useEffect usage and component optimization
- **TypeScript Validator** - Type assertions and patterns
- **Testing Validator** - Test structure and patterns

#### Commands
- `oparry check <file>` - Validate a file
- `oparry check .` - Validate entire project
- `oparry watch` - Watch files and validate on changes
- `oparry init` - Initialize configuration (optional with global config)
- `oparry install` - One-command setup
- `oparry config get/set/list/validate` - Configuration management
- `oparryd status` - Check daemon status
- `oparryd scan` - Scan for Claude Code sessions
- `oparryd validate` - Validate detected sessions
- `oparryd run` - Start daemon
- `oparryd bridge` - Run IPC bridge

#### Output Formats
- Human-readable output (default)
- JSON output for automation
- SARIF output for CI/CD integration

#### Configuration
- Global configuration in `~/.config/oparry/config.toml`
- Auto-detect project type and apply appropriate validators
- Per-project configuration support
- Auto-fix settings with configurable strategies

#### Packaging
- Published on crates.io as `oparry-*` crates
- `cargo install oparry-cli oparry-daemon`
- All 8 crates available: core, parser, validators, autofix, watcher, wrapper, daemon, cli

### Changed
- **BREAKING**: Project renamed to `oparry-*` on crates.io
- All internal dependencies use version 0.2
- Workspace metadata (license, repository, etc.) properly configured

### Fixed
- Empty validators list bug - now correctly defaults to all validators
- Hook temporary files now use correct extensions for language detection
- Fixed hook logger console[level] syntax error
- Fixed cargo package naming conflicts

### Technical Details
- Built with Rust 1.75+ (workspace edition 2021)
- Uses Oxc for JavaScript/TypeScript parsing
- Syn/Quote for Rust parsing
- Tokio for async runtime
- Clap for CLI argument parsing
- Serde for JSON serialization

### Documentation
- README updated with new installation instructions
- Integration hooks documented
- Claude Code compatibility verified

### Dependencies
- oxc_parser 0.30
- tokio 1.35
- serde 1.0
- clap 4.4
- notify 6.1
- thiserror 1.0
- regex 1.10
- chrono 0.4

## [0.1.0] - 2025-03-XX

### Added
- Initial release
- Basic validation framework
- Core types and traits
