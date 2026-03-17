# Parry Command Reference

> Quick reference for all Parry commands and options.

## 📖 Commands

### `parry`

Main CLI entry point.

```bash
parry [OPTIONS] <COMMAND>
```

**Global Options:**
| Option | Short | Description |
|--------|-------|-------------|
| `--config <PATH>` | `-c` | Path to config file |
| `--verbose` | `-v` | Verbose output |
| `--quiet` | `-q` | Suppress non-error output |
| `--color <WHEN>` | | Color output: auto, always, never |
| `--version` | `-V` | Show version |
| `--help` | `-h` | Show help |

---

### `parry check`

Validate codebase.

```bash
parry check [OPTIONS] [PATHS]...
```

**Arguments:**
| Argument | Description |
|----------|-------------|
| `PATHS` | Paths to validate (default: all) |

**Options:**
| Option | Default | Description |
|--------|---------|-------------|
| `--validators <LIST>` | all | Comma-separated validators |
| `--output <FORMAT>` | human | Output: human, json, sarif |
| `--output-file <PATH>` | - | Write output to file |
| `--fix` | false | Auto-fix issues where possible |
| `--strict` | false | Treat warnings as errors |
| `--fail-fast` | false | Stop on first error |

**Available Validators:**
- `tailwind` — Tailwind class validation
- `imports` — Import structure validation
- `components` — Component validation (shadcn/ui)
- `rust` — Rust-specific rules
- `nextjs` — Next.js conventions
- `nestjs` — NestJS conventions
- `all` — All validators (default)

**Exit Codes:**
- `0` — Success
- `1` — Issues found (non-strict)
- `2` — Errors found (strict)

**Examples:**
```bash
parry check
parry check src/ components/
parry check --validators tailwind,imports
parry check --output json --output-file report.json
parry check --fix --strict
```

---

### `parry watch`

Watch files and validate on changes.

```bash
parry watch [OPTIONS] [PATHS]...
```

**Arguments:**
| Argument | Description |
|----------|-------------|
| `PATHS` | Paths to watch (default: all) |

**Options:**
| Option | Default | Description |
|--------|---------|-------------|
| `--debounce <MS>` | 300 | Debounce delay in milliseconds |
| `--clear` | false | Clear screen between runs |
| `--verbose` | false | Show all checked files |
| `--validators <LIST>` | all | Comma-separated validators |
| `--strict` | false | Treat warnings as errors |

**Key Bindings:**
| Key | Action |
|-----|--------|
| `r` | Re-run all checks |
| `q` | Quit |
| `c` | Clear screen |

**Examples:**
```bash
parry watch
parry watch src/
parry watch --debounce 500 --clear
parry watch --validators tailwind
```

---

### `parry wrap`

Wrap a command and intercept file writes.

```bash
parry wrap [OPTIONS] -- <COMMAND> [ARGS]...
```

**Arguments:**
| Argument | Description |
|----------|-------------|
| `COMMAND` | Command to wrap |
| `ARGS` | Arguments for command |

**Options:**
| Option | Default | Description |
|--------|---------|-------------|
| `--block` | false | Block violating writes (vs warn only) |
| `--verbose` | false | Show intercepted writes |
| `--allow <PATTERN>` | - | Regex pattern to always allow |
| `--deny <PATTERN>` | - | Regex pattern to always deny |

**Examples:**
```bash
parry wrap -- claude-code
parry wrap --block -- npm run dev
parry wrap --allow "package-lock.json" -- npm install
```

---

### `parry init`

Initialize Parry configuration.

```bash
parry init [OPTIONS]
```

**Options:**
| Option | Default | Description |
|--------|---------|-------------|
| `--stack <STACK>` | auto | Stack preset: nextjs, rust-axum, nestjs, vitejs, auto |
| `--force` | false | Overwrite existing config |
| `--sample` | false | Create sample config with comments |

**Stack Presets:**
| Stack | Description |
|-------|-------------|
| `nextjs` | Next.js + React + Tailwind + shadcn/ui |
| `rust-axum` | Rust + Axum web framework |
| `nestjs` | NestJS + TypeScript + decorators |
| `vitejs` | Vite + React/Vue + Tailwind |
| `auto` | Detect from project files |

**Examples:**
```bash
parry init
parry init --stack nextjs
parry init --force --sample
```

---

### `parry config`

Manage configuration.

```bash
parry config <SUBCOMMAND>
```

**Subcommands:**

#### `parry config get <KEY>`
Get a config value.

```bash
parry config get tailwind.enabled
```

#### `parry config set <KEY> <VALUE>`
Set a config value.

```bash
parry config set general.strict true
```

#### `parry config list`
List all config values.

```bash
parry config list
```

#### `parry config validate`
Validate current configuration.

```bash
parry config validate
```

---

### `parry completion`

Generate shell completion script.

```bash
parry completion <SHELL>
```

**Shells:**
- `bash`
- `elvish`
- `fish`
- `nushell`
- `powershell`
- `zsh`

**Examples:**
```bash
# Generate for bash
parry completion bash > /etc/bash_completion.d/parry

# Generate for zsh
parry completion zsh > ~/.zfunc/_parry

# Generate for fish
parry completion fish > ~/.config/fish/completions/parry.fish
```

---

## 📋 Configuration Reference

### `[general]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `strict` | bool | false | Warnings as errors |
| `fail_fast` | bool | false | Stop on first error |
| `max_issues` | int | 100 | Max issues to report |

### `[output]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `format` | string | human | Output format |
| `show_paths` | bool | true | Show file paths |
| `color` | string | auto | Color output |

### `[tailwind]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `enabled` | bool | true | Enable Tailwind validator |
| `config_path` | string | tailwind.config.ts | Path to config |
| `safe_list` | array | [] | Allowed class patterns |
| `block_list` | array | [] | Blocked class patterns |
| `max_arbitrary` | int | 5 | Max arbitrary values |

### `[imports]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `enforce_alias` | bool | true | Enforce path aliases |
| `alias_map` | table | {} | Alias mappings |
| `require_extensions` | bool | false | Require file extensions |

### `[components]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `enforce_shadcn` | bool | true | Enforce shadcn/ui |
| `shadcn_path` | string | @/components/ui | shadcn path |

### `[rust]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `enabled` | bool | true | Enable Rust validator |
| `deny_unsafe` | string | warn | unsafe: deny, warn, allow |
| `warn_unwrap` | bool | true | Warn on unwrap() |
| `enforce_result` | bool | true | Enforce Result handling |

---

*Last Updated: 2025-03-16*
