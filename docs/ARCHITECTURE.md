# Parry Architecture

> Technical deep-dive into Parry's design, components, and data flow.

## 🏗️ Overview

Parry is a modular Rust CLI organized as a Cargo workspace with specialized crates:

```
parry/
├── Cargo.toml                    # Workspace root
├── crates/
│   ├── core/                     # Validation engine
│   ├── parser/                   # Multi-language parsers
│   ├── validators/               # Specialized validators
│   ├── watcher/                  # File system watcher
│   ├── wrapper/                  # Stdio interceptor
│   └── cli/                      # CLI interface
├── configs/                      # Default rule configurations
└── docs/                         # Documentation
```

---

## 📦 Crates

### 1. `core` — The Heart

**Purpose:** Shared types, error handling, reporting, and rule engine.

```rust
// Core abstractions
pub trait Validator {
    fn validate(&self, input: &Input) -> Result<Report, Vec<Error>>;
}

pub trait Reporter {
    fn report(&self, result: &ValidationResult) -> String;
}
```

**Key Components:**
- `error.rs` — Unified error types
- `report.rs` — Validation reports (JSON/SARIF)
- `config.rs` — Configuration loading and merging
- `rule.rs` — Rule engine (pattern matching, severity)

### 2. `parser` — Multi-Language Support

**Purpose:** Parse source code into ASTs for validation.

| Language | Parser | Crate |
|----------|--------|-------|
| JavaScript/TypeScript | Oxc | `oxc_parser` |
| Rust | Syn | `syn` |
| Generic (fallback) | Regex/Glob | `regex`, `glob` |

```rust
pub enum ParsedCode {
    JavaScript(oxc_ast::Program),
    Rust(syn::File),
    Generic(String),  // Raw text for pattern matching
}
```

### 3. `validators` — Specialized Rules

**Purpose:** Language and framework-specific validation.

| Validator | Responsibility |
|-----------|----------------|
| `TailwindValidator` | Class existence, safety, accessibility |
| `ImportValidator` | Alias enforcement, component paths |
| `ComponentValidator` | shadcn/ui usage, component props |
| `RustValidator` | Ownership, unsafe, unwrap patterns |
| `ReactValidator` | Hooks rules, RSC patterns |

```rust
pub struct TailwindValidator {
    config: TailwindConfig,
    safe_list: HashSet<String>,
    block_list: HashSet<String>,
}
```

### 4. `watcher` — File System Events

**Purpose:** Monitor files and trigger validation on changes.

**Dependencies:** `notify`, `tokio`

```rust
pub struct FileWatcher {
    debounce: Duration,
    filters: Vec<FileFilter>,
}
```

**Features:**
- Debouncing (avoid spam on rapid saves)
- Path filtering (only watch relevant files)
- Event batching (validate multiple changes together)

### 5. `wrapper` — Stdio Interceptor

**Purpose:** Intercept Claude Code writes before they reach disk.

**Protocol:**
```text
Claude Code → Parry Wrapper → [Validate] → Disk (if OK)
                          ↓
                    Error Report → Claude Code (retry)
```

```rust
pub struct StdioWrapper {
    allowed_patterns: Vec<Regex>,
    blocked_patterns: Vec<Regex>,
}
```

### 6. `cli` — User Interface

**Purpose:** Command-line interface with `clap`.

**Commands:**
- `parry check` — One-shot validation
- `parry watch` — Continuous validation
- `parry wrap -- <cmd>` — Wrap another command
- `parry init` — Initialize config

---

## 🔄 Data Flow

### Check Mode
```
Source Files
    ↓
Parser (AST)
    ↓
Validators (parallel)
    ↓
Report Aggregation
    ↓
Output (JSON/SARIF/Human)
```

### Watch Mode
```
File System
    ↓
notify Event
    ↓
Debounce
    ↓
Validate changed files
    ↓
Report (if errors)
```

### Wrap Mode
```
Wrapped Process Stdout
    ↓
Parse Write Operations
    ↓
Validate Content
    ↓
Allow/Deny Write
```

---

## 🎨 Configuration System

Parry uses a hierarchical configuration system:

```
1. Global config (~/.config/parry/config.toml)
2. Project config (.parryrc.toml)
3. Config files (configs/*)
4. CLI flags (override all)
```

### Config Structure

```toml
[general]
strict = true
fail_fast = false

[output]
format = "json"  # json | sarif | human
verbose = false

[[tailwind]]
enabled = true
config_path = "tailwind.config.ts"

[[imports]]
enforce_alias = true
alias_map = { "@/" = "./src" }

[validation.rust]
deny_unsafe = "warn"
warn_unwrap = true
```

---

## 🔌 Integration Points

### Claude Code
Parry integrates via the wrapper mode or as a post-generation hook in `CLAUDE.md`:

```bash
# In CLAUDE.md skills
After generating code:
parry check --fix
```

### CI/CD
SARIF output integrates with GitHub Security:

```yaml
- name: Run Parry
  run: parry check --output sarif --output-file results.sarif

- name: Upload SARIF
  uses: github/codeql-action/upload-sarif@v2
  with:
    sarif_file: results.sarif
```

---

## 🚀 Performance Considerations

- **Parallel Validation:** Use `rayon` for parallel file processing
- **Incremental:** Cache validation results, invalidate on change
- **Lazy Parsing:** Only parse files that match enabled validators
- **Streaming:** Process large files in chunks

---

## 🧪 Testing Strategy

```
tests/
├── unit/           # Unit tests per crate
├── integration/    # Cross-crate tests
├── fixtures/       # Sample code for validation
└── benchmarks/     # Performance benchmarks
```

---

*Last Updated: 2025-03-16*
