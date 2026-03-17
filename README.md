# Parry

> **The Agentic Linter for AI-Generated Code** — Ultra-strict validation for Claude Code, Cursor, and AI assistants.

**Problem:** AI-generated code often violates design systems, uses invalid Tailwind classes, or breaks project conventions.

**Solution:** Parry is a Rust-based CLI that intercepts, validates, and enforces code quality rules before code hits your filesystem.

## 🎯 What It Does

- **Tailwind Validator** — Ensures every class exists in your design system
- **Import Structure Checker** — Enforces alias rules and component imports (shadcn/ui, etc.)
- **Multi-Stack Support** — Rust/Axum, Next.js, NestJS, ViteJS with strict language-specific rules
- **Watch Mode** — Real-time validation as you code
- **Wrapper Mode** — Intercepts Claude Code writes before they reach disk
- **Smart Outputs** — JSON for automation, SARIF for CI/CD integration

## ⚡ Quick Start

```bash
# Install (one-command setup)
cargo install oparry-cli oparry-daemon
parry install

# Restart Claude Code - Oparry will validate automatically!
```

That's it! Parry is now integrated with Claude Code:
- ✅ Files are validated **before** they're written
- ✅ Daemon auto-starts with Claude Code
- ✅ Multi-session, multi-project support
- ✅ Works with all your projects immediately

## 📖 Commands

| Command | Description |
|----------|-------------|
| `oparry install` | One-command setup for Claude Code integration |
| `oparry check <file>` | Manually validate a file |
| `oparry check .` | Validate entire project |
| `oparry watch` | Watch files and validate on changes |
| `oparryd status` | Check daemon status |
| `oparryd run` | Start daemon manually |

## 📖 Use Cases

| Scenario | Command |
|----------|---------|
| Validate Tailwind classes | `parry check --validators tailwind` |
| Enforce shadcn/ui imports | `parry check --validators imports` |
| Real-time development | `parry watch` |
| Claude Code integration | `parry wrap` |
| CI/CD pipeline | `parry check --output sarif` |

## 🏗️ Architecture

```
parry/
├── core/         # Validation engine & abstractions
├── parser/       # Multi-language parsers (Oxc, Syn)
├── validators/   # Specialized validators
├── watcher/      # File system watcher
├── wrapper/      # Stdio interceptor
└── cli/          # Command-line interface
```

See [ARCHITECTURE.md](docs/ARCHITECTURE.md) for details.

## 📚 Documentation

- [ARCHITECTURE.md](docs/ARCHITECTURE.md) — Technical architecture
- [GUIDE.md](docs/GUIDE.md) — User guide
- [REFERENCE.md](docs/REFERENCE.md) — Command reference
- [ROADMAP.md](docs/ROADMAP.md) — Roadmap

## 🚀 Version

**Current:** v0.2.0 (Alpha)

See [ROADMAP.md](docs/ROADMAP.md) for what's next.

## 📄 License

MIT

---

*Built with Rust, validated with love.*
