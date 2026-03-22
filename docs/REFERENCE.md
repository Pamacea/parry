# Parry Command Reference v0.3.0

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
| `--fix` | false | Auto-fix issues where possible |
| `--strict` | false | Treat warnings as errors |

**Available Validators:**
- `tailwind` — Tailwind class validation
- `imports` — Import structure validation
- `rust` — Rust-specific rules
- `react` — React best practices
- `components` — Component validation (shadcn/ui)
- `a11y` — Accessibility
- `security` — Security vulnerabilities
- `performance` — Performance anti-patterns
- `typescript` — TypeScript strict mode
- `testing` — Testing best practices

**Exit Codes:**
- `0` — Success
- `1` — Issues found

**Examples:**
```bash
parry check
parry check src/ components/
parry check --validators tailwind,imports
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

**Examples:**
```bash
parry watch
parry watch src/
parry watch --debounce 500 --clear
parry watch --validators tailwind
```

---

### `parry run` (NEW in v0.3.0)

Run a command with file write validation.

```bash
parry run [OPTIONS] -- <COMMAND> [ARGS]...
```

**Arguments:**
| Argument | Description |
|----------|-------------|
| `COMMAND` | Command to run |
| `ARGS` | Arguments for command |

**Options:**
| Option | Default | Description |
|--------|---------|-------------|
| `--block` | false | Block violating writes (exit on error) |
| `--validators <LIST>` | all | Comma-separated validators |

**Examples:**
```bash
parry run -- npm run dev
parry run -- cargo build
parry run --block -- npm test
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
| `--stack <STACK>` | auto | Stack preset: nextjs, rust, vite, auto |
| `--force` | false | Overwrite existing config |

**Stack Presets:**
| Stack | Description |
|-------|-------------|
| `nextjs` | Next.js + React + Tailwind |
| `rust` | Rust + Axum |
| `vite` | Vite + React/Vue + Tailwind |
| `auto` | Detect from project files |

**Examples:**
```bash
parry init
parry init --stack nextjs
parry init --force
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
Set a config value (coming soon).

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
| `format` | string | human | Output format: human, json, sarif |

### `[tailwind]`
| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `enabled` | bool | true | Enable Tailwind validator |
| `safe_list` | array | [] | Allowed class patterns |
| `block_list` | array | [] | Blocked class patterns |
| `max_arbitrary_values` | int | 5 | Max arbitrary values |

### `[imports]`
| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `enforce_alias` | bool | true | Enforce path aliases |
| `alias_map` | table | {} | Alias mappings |

### `[rust]`
| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `enabled` | bool | true | Enable Rust validator |
| `deny_unsafe` | string | warn | unsafe: deny, warn, allow |
| `warn_unwrap` | bool | true | Warn on unwrap() |

---

*Last Updated: 2025-03-22 (v0.3.0)*
