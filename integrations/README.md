# Parry - Claude Code Integration

> **Post-write validation and auto-correction for AI-generated code**

This directory contains a single hook for integrating Parry with Claude Code's file operations.

## Architecture

```
┌─────────────────┐     Write/Edit Tool     ┌─────────────────┐
│   Claude Code   │ ───────────────────→   │  File System   │
│  (writes files) │                         │  (committed)    │
└─────────────────┘                         └─────────────────┘
         ↓                                            ↓
    ┌──────────────────────────────────────────────────────┐
    │           PostToolUse Hook (parry-post-write.cjs)  │
    │  - Validates AFTER write                            │
    │  - Auto-corrects fixable issues                     │
    │  - Logs non-fixable issues                          │
    └──────────────────────────────────────────────────────┘
                           ↓
                    ┌─────────────┐
                    │    Parry    │
                    │   (check)   │
                    └─────────────┘
```

## Important: PostToolUse Hook

**Claude Code does NOT call PreToolUse for Write/Edit operations.**

- ❌ **PreToolUse**: NOT called for Write/Edit → Cannot prevent writes
- ✅ **PostToolUse**: Called after Write/Edit → Can validate and auto-fix

This means:
- Files ARE written to disk first
- Parry validates afterward
- Fixable issues are auto-corrected
- Non-fixable issues are logged

## Installation

### Quick Install

```bash
# Copy the hook
cp integrations/parry-post-write.cjs ~/.claude/hooks/

# Add to ~/.claude/settings.json (see below)
```

### Manual Installation

1. **Add to `~/.claude/settings.json`:**

```json
{
  "hooks": {
    "PostToolUse": [
      {
        "hooks": [{
          "command": "node ~/.claude/hooks/parry-post-write.cjs",
          "timeout": 10000,
          "type": "command"
        }],
        "matcher": "Write|Edit"
      }
    ]
  },
  "env": {
    "PARRY_BIN": "parry",
    "PARRY_AUTO_FIX": "true",
    "PARRY_DEBUG": "false"
  }
}
```

### Paths

**Windows:**
- **Hooks**: `C:\Users\<user>\.claude\hooks\`
- **Config**: `C:\Users\<user>\.claude\settings.json`

**Unix/Linux/Mac:**
- **Hooks**: `~/.claude/hooks/`
- **Config**: `~/.claude/settings.json`

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `PARRY_BIN` | `parry` | Path to parry executable |
| `PARRY_CONFIG` | - | Config file path (optional) |
| `PARRY_AUTO_FIX` | `true` | Auto-fix fixable issues |
| `PARRY_DEBUG` | `false` | Debug logging |

## How It Works

1. **File Written**: Claude Code writes a file to disk
2. **Hook Triggered**: `parry-post-write.cjs` is called
3. **Validation**: Parry checks the file for issues
4. **Auto-Fix**: Fixable issues are automatically corrected
5. **Report**: Non-fixable issues are logged to console

## Validation Rules

Parry validates:

1. **Tailwind Classes**: Only classes from your design system
2. **Import Structure**: Enforces alias rules (@/ → ./src)
3. **React Patterns**: Function components, hooks limits
4. **CSS**: No !important, line length limits
5. **Rust**: Error handling, no unwraps
6. **TypeScript**: Type safety, proper typing

See main documentation for configuration.

## Troubleshooting

### Hook not executing

Check that:
1. Hook is in `~/.claude/hooks/` directory
2. Hook is referenced in `~/.claude/settings.json`
3. Node.js is installed and in PATH

### Parry not found

Set `PARRY_BIN` env var in settings.json or ensure parry is in PATH:

```json
{
  "env": {
    "PARRY_BIN": "C:/Users/YourUser/.cargo/bin/parry.exe"
  }
}
```

### Enable debug logging

```json
{
  "env": {
    "PARRY_DEBUG": "true"
  }
}
```

## License

MIT
