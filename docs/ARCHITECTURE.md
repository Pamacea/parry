# Parry Architecture v0.3.0

> Technical deep-dive into Parry's simplified design, components, and data flow.

## 🏗️ Overview (v0.3.0 - Simplified)

Parry v0.3.0 is a **radically simplified** Rust CLI organized as a 2-crate workspace:

```
parry/
├── Cargo.toml                    # Workspace root (2 crates only)
├── crates/
│   ├── parry-core/               # Core library (oalacea-parry-core)
│   │   ├── src/
│   │   │   ├── lib.rs           # Module exports
│   │   │   ├── config.rs
│   │   │   ├── error.rs
│   │   │   ├── report.rs
│   │   │   ├── rule.rs
│   │   │   ├── parser/          # Module (not a crate)
│   │   │   ├── validators/      # Module (not a crate)
│   │   │   ├── watcher/         # Module (not a crate)
│   │   │   ├── wrapper/         # Module (not a crate)
│   │   │   └── autofix/         # Module (not a crate)
│   └── parry/                   # CLI binary (oalacea-parry)
│       └── src/
│           ├── main.rs
│           └── commands/
├── integrations/
│   ├── parry-post-write.cjs     # Single Claude Code hook
│   └── README.md
└── docs/                        # Documentation
```

## 🔄 What Changed in v0.3.0

### Removed
- ❌ `parry-daemon/` — No more background daemon
- ❌ 7-crates architecture → 2-crates
- ❌ IPC/sockets/gRPC communication
- ❌ Commands: `parryd status`, `parryd run`, `hook`, `install`

### Simplified
- ✅ **Monolithic core library** — All modules in one crate
- ✅ **Direct CLI** — No daemon dependency
- ✅ **Single hook** — `parry-post-write.cjs` for Claude Code
- ✅ **Synchronous wrapper** — Simple `parry run <cmd>` mode

---

## 📦 Crate Structure

### 1. `parry-core` — The Monolithic Library

**Package Name:** `oalacea-parry-core`
**Import Path:** `use oalacea_parry_core::`

**Purpose:** All validation logic in one crate using Rust modules.

```rust
// Re-exports
pub use config::{Config, OutputFormat};
pub use error::{Error, Result};
pub use report::{Issue, IssueLevel, Report, ValidationResult};

// Module structure
pub mod parser;      // Language parsing (JS/TS, Rust)
pub mod validators;  // All validators
pub mod watcher;     // File watching
pub mod wrapper;     // Sync wrapper mode
pub mod autofix;     // Auto-fix engine
```

**Key Files:**
- `lib.rs` — Module exports and re-exports
- `config.rs` — Configuration loading
- `error.rs` — Unified error types
- `report.rs` — Validation reports
- `rule.rs` — Rule engine

### 2. `parry` — CLI Binary

**Package Name:** `oalacea-parry`
**Binary Name:** `parry`

**Purpose:** Command-line interface using `clap`.

**Commands:**
```bash
parry check [paths]      # Validate codebase
parry watch [paths]      # Watch for changes
parry run <cmd> [args]   # Run with interception
parry init               # Initialize config
parry config <subcmd>    # Manage config
```

---

## 🔄 Data Flow

### Check Mode
```
Source Files
    ↓
Parser (AST) — parser::parser_for_path()
    ↓
Validators — validators::Validators
    ↓
Report Aggregation
    ↓
Output (JSON/SARIF/Human)
```

### Watch Mode
```
File System
    ↓
notify::RecommendedWatcher
    ↓
Debounce (300ms default)
    ↓
Validate changed files
    ↓
Report (if errors)
```

### Run Mode (New!)
```
Command (e.g., "npm run dev")
    ↓
Spawn Process + Watch Files
    ↓
Validate on file changes
    ↓
Report violations (doesn't block)
```

---

## 🎨 Configuration System

Parry uses a hierarchical configuration system:

```
1. Project config (.parryrc.toml)
2. CLI flags (override)
```

### Config Structure

```toml
[general]
strict = true
fail_fast = false

[output]
format = "human"  # json | sarif | human

[tailwind]
enabled = true
safe_list = ["p-*", "m-*"]
block_list = ["bg-red-500"]
max_arbitrary_values = 5

[imports]
enforce_alias = true
alias_map = { "@/" = "./src" }

[rust]
enabled = true
deny_unsafe = "warn"
warn_unwrap = true
```

---

## 🔌 Claude Code Integration

### Single Hook Approach

**File:** `integrations/parry-post-write.cjs`

```javascript
// Triggered after Write/Edit operations
// Validates and auto-fixes code
node ~/.claude/hooks/parry-post-write.cjs
```

**Installation:**
```bash
# 1. Copy hook
cp integrations/parry-post-write.cjs ~/.claude/hooks/

# 2. Add to ~/.claude/settings.json
{
  "hooks": {
    "PostToolUse": [{
      "hooks": [{
        "command": "node ~/.claude/hooks/parry-post-write.cjs",
        "timeout": 10000
      }],
      "matcher": "Write|Edit"
    }]
  }
}
```

---

## 🚀 Performance Considerations (v0.3.0)

- **No IPC overhead** — Direct function calls
- **No daemon latency** — Instant startup
- **Single binary** — Faster installation
- **Minimal dependencies** — Only essential crates

---

*Last Updated: 2025-03-22 (v0.3.0)*
