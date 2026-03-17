# Parry - Claude Code Integration

> **Deep integration with Claude Code for real-time validation**

This directory contains the hooks and bridges for integrating Parry with Claude Code's file operations.

## Overview

Parry intercepts Claude Code's file write operations through a bidirectional IPC protocol, validating code before it hits the filesystem. This ensures that AI-generated code always follows your project's conventions and design system.

## Architecture

```
┌─────────────────┐     IPC Protocol      ┌─────────────────┐
│   Claude Code   │ ←─────────────────→  │    Parry        │
│  (writes files) │   stdin/stdout JSON   │  (validates)    │
└─────────────────┘                       └─────────────────┘
         ↓                                          ↓
    ┌─────────┐                              ┌──────────┐
    │ Hook.js │                              │ Daemon   │
    │ Hook.py │                              │ Bridge   │
    └─────────┘                              └──────────┘
```

## Files

| File | Language | Purpose |
|------|----------|---------|
| `claude-hook.js` | Node.js | Hook for Claude Code (Node.js runtime) |
| `claude-hook.py` | Python | Hook for Claude Code (Python runtime) |
| `README.md` | Markdown | This documentation |

## Installation

### Quick Install

```bash
# Install Parry (if not already installed)
cargo install parry-cli

# Install the Claude Code hook
parry hook install
```

### Manual Installation

#### Using Node.js Hook

```bash
# Copy hook to Parry directory
cp integrations/claude-hook.js ~/.parry/hooks/

# Make executable (Unix only)
chmod +x ~/.parry/hooks/claude-hook.js

# Add to Claude Code config (~/.claude/config.json)
# See configuration section below
```

#### Using Python Hook

```bash
# Copy hook to Parry directory
cp integrations/claude-hook.py ~/.parry/hooks/

# Make executable (Unix only)
chmod +x ~/.parry/hooks/claude-hook.py

# Add to Claude Code config (~/.claude/config.json)
# See configuration section below
```

## Configuration

### Claude Code Config

Add the hook to your `~/.claude/config.json`:

```json
{
  "preWriteHooks": [
    {
      "command": "node ~/.parry/hooks/claude-hook.js",
      "enabled": true
    }
  ]
}
```

Or for Python:

```json
{
  "preWriteHooks": [
    {
      "command": "python3 ~/.parry/hooks/claude-hook.py",
      "enabled": true
    }
  ]
}
```

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `PARRY_EXE` | `parry` | Path to Parry executable |
| `PARRY_MODE` | `warn` | Operation mode: `strict`, `warn`, `disabled` |
| `PARRY_DEBUG` | `false` | Enable debug logging |
| `PARRY_LOG` | `~/.parry/hook.log` | Log file path |
| `PARRY_STRICT` | `false` | Block writes on validation errors |

### Modes

- **strict**: Block all writes that fail validation
- **warn** (default): Show warnings but allow writes
- **disabled**: Pass through without validation

## Protocol

### Request Format

```json
{
  "type": "write_file",
  "id": "req-1234567890",
  "path": "src/components/Button.tsx",
  "content": "export const Button = () => {...}",
  "encoding": "utf-8",
  "create_dirs": true
}
```

### Response Format

```json
{
  "type": "approved",
  "request_id": "req-1234567890",
  "modified_content": null
}
```

Or for rejected:

```json
{
  "type": "rejected",
  "request_id": "req-1234567890",
  "message": "Validation failed",
  "issues": [
    {
      "code": "tailwind-invalid-class",
      "level": "error",
      "message": "Invalid Tailwind class: w-xl",
      "line": 10,
      "column": 15,
      "suggestion": "Use max-w-xl instead",
      "context": "className=\"w-xl\""
    }
  ],
  "can_autofix": true
}
```

## Usage Examples

### Testing the Hook

```bash
# Test with a ping
echo '{"type":"ping"}' | node integrations/claude-hook.js

# Test with a write request
echo '{"type":"write_file","path":"test.ts","content":"export const x = 1;"}' | node integrations/claude-hook.js
```

### Running as a Daemon

```bash
# Start the Parry daemon with IPC
parry daemon --ipc

# The hook will automatically connect to the daemon
```

## Validation Rules

Parry validates:

1. **Tailwind Classes**: Only classes from your design system
2. **Import Structure**: Enforces alias rules (@/ → ./src)
3. **React Patterns**: Function components, hooks limits
4. **CSS**: No !important, line length limits
5. **Rust**: Error handling, no unwraps

See the main documentation for configuring these rules.

## Troubleshooting

### Hook not executing

1. Check Claude Code config: `cat ~/.claude/config.json`
2. Verify hook is executable: `ls -l ~/.parry/hooks/`
3. Check logs: `tail -f ~/.parry/logs/hook-errors.log`

### Parry not found

Set the `PARRY_EXE` environment variable:

```bash
export PARRY_EXE=/path/to/parry
```

### Timeout errors

Increase timeout in `~/.parry/daemon.toml`:

```toml
[bridge]
validation_timeout = 60  # seconds
```

## Development

### Running Tests

```bash
# Python tests
pytest integrations/

# Node.js tests
npm test -- integrations/
```

### Protocol Version

Current protocol version: `0.2.0`

Protocol changes are backwards compatible within major versions.

## License

MIT

## Contributing

Contributions welcome! Please read the main CONTRIBUTING.md file.
