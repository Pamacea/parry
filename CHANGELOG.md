# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
