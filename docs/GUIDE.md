# Parry User Guide

> Everything you need to use Parry effectively.

## 📦 Installation

### From Crates.io (Recommended)

```bash
cargo install parry-cli
```

### From Source

```bash
git clone https://github.com/yourusername/parry.git
cd parry
cargo install --path crates/cli
```

### Verify Installation

```bash
parry --version
# Output: parry 0.2.0
```

---

## 🚀 Quick Start

### 1. Initialize in Your Project

```bash
cd your-project
parry init
```

This creates `.parryrc.toml` with defaults for your detected stack.

### 2. Configure Rules

Edit `.parryrc.toml`:

```toml
[general]
strict = true

[tailwind]
enabled = true
config_path = "tailwind.config.ts"

[imports]
enforce_alias = true
```

### 3. Run Validation

```bash
# Check all files
parry check

# Check specific paths
parry check src/ components/

# Watch mode
parry watch
```

---

## 📖 Commands

### `parry check`

One-shot validation of your codebase.

```bash
# Basic usage
parry check

# Specific validators
parry check --validators tailwind,imports

# Output format
parry check --output json
parry check --output sarif

# Auto-fix (where possible)
parry check --fix
```

**Exit Codes:**
- `0` — No issues found
- `1` — Issues found (in non-strict mode)
- `2` — Errors found (in strict mode)

### `parry watch`

Continuous validation as you code.

```bash
# Basic watch
parry watch

# Watch specific directories
parry watch src/ components/

# Quiet mode (only report errors)
parry watch --quiet

# Clear screen between runs
parry watch --clear
```

### `parry wrap`

Intercept and validate writes from another process.

```bash
# Wrap Claude Code
parry wrap -- claude-code

# Wrap any command
parry wrap -- npm run dev
```

**How it works:**
1. Parry spawns the wrapped process
2. Intercepts file write operations
3. Validates content before write
4. Blocks writes that violate rules

### `parry init`

Initialize Parry configuration.

```bash
# Interactive
parry init

# Specify stack
parry init --stack nextjs
parry init --stack rust-axum
parry init --stack nestjs
```

---

## 🎨 Configuration

### Config File Structure

`.parryrc.toml`:

```toml
[general]
# Enable strict mode (errors instead of warnings)
strict = false

# Stop on first error
fail_fast = false

# Maximum number of issues to report
max_issues = 100

[output]
# Output format: human, json, sarif
format = "human"

# Include file paths in output
show_paths = true

# Color output (auto, always, never)
color = "auto"

# --- Tailwind Configuration ---
[tailwind]
# Enable Tailwind validation
enabled = true

# Path to Tailwind config
config_path = "tailwind.config.ts"

# Safe list: classes always allowed
safe_list = [
    "p-*",
    "m-*",
    "flex",
    "grid"
]

# Block list: classes never allowed
block_list = [
    "bg-red-500",
    "hover:*-shake"
]

# Enforce arbitrary value limits
max_arbitrary_values = 5

# --- Import Configuration ---
[imports]
# Enforce path aliases
enforce_alias = true

# Alias mappings
alias_map = {
    "@/" = "./src",
    "@/components" = "./components",
    "@/lib" = "./lib"
}

# Require explicit extensions
require_extensions = false

# --- Component Configuration ---
[components]
# Enforce shadcn/ui usage
enforce_shadcn = true

# shadcn/ui components path
shadcn_path = "@/components/ui"

# --- Rust Configuration ---
[rust]
# Enable Rust validation
enabled = true

# Deny unsafe code
deny_unsafe = "warn"  # deny | warn | allow

# Warn on .unwrap()
warn_unwrap = true

# Enforce error handling
enforce_result_handling = true

# --- Next.js Configuration ---
[nextjs]
# Enable Next.js validation
enabled = true

# Enforce App Router conventions
enforce_app_router = true

# Validate page exports
validate_page_exports = true

# --- NestJS Configuration ---
[nestjs]
# Enable NestJS validation
enabled = true

# Enforce decorator usage
enforce_decorators = true

# Validate module imports
validate_modules = true
```

### Environment Variables

```bash
# Parry config directory
export PARRY_CONFIG_DIR="$HOME/.config/parry"

# Disable color output
export NO_COLOR=1

# Verbose output
export PARRY_VERBOSE=1
```

---

## 🎯 Common Workflows

### Validate Before Commit

Add to `.git/hooks/pre-commit`:

```bash
#!/bin/bash
parry check || exit 1
```

### CI/CD Pipeline

```yaml
# .github/workflows/parry.yml
name: Parry Validation

on: [push, pull_request]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install Parry
        run: cargo install parry-cli
      - name: Run validation
        run: parry check --output sarif --output-file results.sarif
      - name: Upload results
        uses: github/codeql-action/upload-sarif@v2
        with:
          sarif_file: results.sarif
```

### Claude Code Integration

Add to your `CLAUDE.md` or skills:

```markdown
## Parry Integration

After any code generation:
1. Run `parry check`
2. Review and fix any issues
3. Only then commit

For auto-fix: `parry check --fix`
```

---

## 📊 Output Formats

### Human (Default)

```
✓ src/components/Button.tsx
✗ src/components/Card.tsx
  error: Invalid Tailwind class "bg-red-500"
    --> src/components/Card.tsx:15:10
     |
  15 |   <div className="bg-red-500 p-4">
     |            ^^^^^^^^^^^ use "bg-red-600" or define custom

1 error, 0 warnings
```

### JSON

```json
{
  "version": "0.2.0",
  "summary": {
    "errors": 1,
    "warnings": 0,
    "files_checked": 42
  },
  "issues": [
    {
      "level": "error",
      "code": "tailwind-invalid-class",
      "message": "Invalid Tailwind class",
      "file": "src/components/Card.tsx",
      "line": 15,
      "column": 10,
      "suggestion": "Use bg-red-600 or define custom"
    }
  ]
}
```

### SARIF

Standard SARIF format for tool integration.

---

## 🐛 Troubleshooting

### "No config found"

Run `parry init` to create a config file.

### "Too many false positives"

Adjust `safe_list` in your `.parryrc.toml` or set `strict = false`.

### "Slow performance"

1. Use `--validators` to only run needed validators
2. Exclude node_modules/target in config
3. Use incremental mode (coming in v0.3.0)

---

*Last Updated: 2025-03-16*
