# Parry User Guide v0.3.0

> Everything you need to use Parry effectively with the simplified architecture.

## 📦 Installation

### From Source (Recommended during development)

```bash
cd parry
cargo install --path crates/parry
```

This installs the `parry` binary to `~/.cargo/bin/`.

### Verify Installation

```bash
parry --version
# Output: parry 0.3.0
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

# Strict mode
parry check --strict
```

**Exit Codes:**
- `0` — No issues found
- `1` — Issues found

### `parry watch`

Continuous validation as you code.

```bash
# Basic watch
parry watch

# Watch specific directories
parry watch src/ components/

# Clear screen between runs
parry watch --clear
```

### `parry run` (NEW in v0.3.0)

Run a command with file write interception.

```bash
# Run a dev server with validation
parry run npm run dev

# Run any command
parry run cargo build
```

**How it works:**
1. Parry spawns the command
2. Watches for file changes
3. Validates changed files
4. Reports violations (doesn't block)

### `parry init`

Initialize Parry configuration.

```bash
# Auto-detect stack
parry init

# Force overwrite
parry init --force

# Specify stack
parry init --stack nextjs
parry init --stack rust
parry init --stack vite
```

### `parry config`

Manage configuration.

```bash
# Get a value
parry config get tailwind.enabled

# Set a value (coming soon)
parry config set general.strict true

# List all values
parry config list

# Validate config
parry config validate
```

---

## 🎨 Configuration

### Config File Structure

`.parryrc.toml`:

```toml
[general]
strict = false
fail_fast = false
max_issues = 100

[output]
format = "human"

[tailwind]
enabled = true
safe_list = ["p-*", "m-*", "flex", "grid"]
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

### Environment Variables

```bash
# Disable color output
export NO_COLOR=1

# Verbose output
parry check --verbose
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
name: Parry Validation

on: [push, pull_request]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run Parry
        run: |
          cargo install parry --path crates/parry
          parry check --output sarif
```

### Claude Code Integration

See `integrations/README.md` for the single hook approach.

---

## 📊 Output Formats

### Human (Default)

```
✓ All checks passed!
Files checked: 42
Issues found: 0
```

### JSON

```json
{
  "passed": true,
  "files_checked": 42,
  "issues": []
}
```

---

## 🐛 Troubleshooting

### "No config found"

Run `parry init` to create a config file.

### "Too many false positives"

Adjust `safe_list` in your `.parryrc.toml`.

### "Hook not executing"

Make sure the hook is in `~/.claude/hooks/` and referenced in `~/.claude/settings.json`.

---

*Last Updated: 2025-03-22 (v0.3.0)*
