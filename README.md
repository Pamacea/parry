# Parry

> **The Agentic Linter for AI-Generated Code** — Ultra-strict validation for Claude Code, Cursor, and AI assistants.

**Problem:** AI-generated code often violates design systems, uses invalid Tailwind classes, or breaks project conventions.

**Solution:** Parry is a Rust-based CLI that validates and enforces code quality rules.

## 🎯 What It Does

- **Tailwind Validator** — Ensures every class exists in your design system
- **Import Structure Checker** — Enforces alias rules and component imports
- **Multi-Stack Support** — Rust, Next.js, Vite with language-specific rules
- **Watch Mode** — Real-time validation as you code
- **Auto-Fix** — Automatically corrects common issues
- **Claude Code Hook** — Post-write validation and auto-correction

## ⚡ Quick Start

```bash
# Install from source
cd parry
cargo install --path crates/parry

# Initialize in your project
parry init

# Run validation
parry check
```

## 📖 Commands

| Command | Description |
|----------|-------------|
| `parry check [paths]` | Validate codebase |
| `parry watch [paths]` | Watch files and validate on changes |
| `parry run <cmd> [args]` | Run command with file validation |
| `parry init` | Initialize configuration |
| `parry config <subcmd>` | Manage configuration |

## 🔌 Claude Code Integration

### Single Hook Setup

```bash
# Copy the hook
cp integrations/parry-post-write.cjs ~/.claude/hooks/

# Add to ~/.claude/settings.json
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

See [integrations/README.md](integrations/README.md) for details.

## 🏗️ Architecture (v0.3.0)

```
parry/
├── crates/
│   ├── parry-core/       # Core library (all modules)
│   │   ├── parser/        # Language parsing
│   │   ├── validators/    # All validators
│   │   ├── watcher/       # File watching
│   │   ├── wrapper/       # Sync wrapper mode
│   │   └── autofix/       # Auto-fix engine
│   └── parry/            # CLI binary
└── integrations/
    └── parry-post-write.cjs  # Claude Code hook
```

**Simplified from 7 crates to 2!**

See [ARCHITECTURE.md](docs/ARCHITECTURE.md) for details.

## 📚 Documentation

- [ARCHITECTURE.md](docs/ARCHITECTURE.md) — Technical architecture
- [GUIDE.md](docs/GUIDE.md) — User guide
- [REFERENCE.md](docs/REFERENCE.md) — Command reference
- [integrations/README.md](integrations/README.md) — Claude Code integration

## 🚀 Version

**Current:** v0.3.0

## 📄 License

MIT

---

*Built with Rust, validated with love.*
