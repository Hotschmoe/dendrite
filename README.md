# Dendrite 🌿

**A codebase mapping and dependency analysis tool for systems programmers.**

Dendrite visualizes your codebase as a dependency graph, detects circular imports, enforces architectural layering, and generates documentation that both humans and AI agents can navigate.

Built for operating systems and embedded development where dependency discipline is critical.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)

## Features

- **Dependency Graph Visualization** — Interactive TUI with hierarchical left-to-right layout
- **Cycle Detection** — Find and highlight circular imports before they become problems
- **Layer Enforcement** — Define architectural boundaries (driver → platform → core → app) and catch violations
- **AI-Ready Output** — Generates `CODEBASE.md` and structured summaries for LLM agents
- **Zero Dependencies for Users** — Single static binary per platform
- **CI Integration** — Exit codes for automated checks in your pipeline

## Quick Start

```bash
# Download the latest release for your platform
curl -L https://github.com/yourname/dendrite/releases/latest/download/dendrite-linux-x86_64 -o dendrite
chmod +x dendrite

# Analyze current directory
./dendrite

# Launch interactive TUI
./dendrite --tui

# CI mode (fails on cycles)
./dendrite --check
```

## Installation

### Pre-built Binaries

Download from [Releases](https://github.com/yourname/dendrite/releases):

| Platform | Binary |
|----------|--------|
| Linux x86_64 | `dendrite-linux-x86_64` |
| Linux ARM64 | `dendrite-linux-aarch64` |
| macOS x86_64 | `dendrite-macos-x86_64` |
| macOS ARM64 | `dendrite-macos-aarch64` |
| Windows x86_64 | `dendrite-windows-x86_64.exe` |

### Build from Source

```bash
git clone https://github.com/yourname/dendrite
cd dendrite
cargo build --release
# Binary at target/release/dendrite
```

## Usage

### Basic Analysis

```bash
# Analyze src/ directory (default)
dendrite

# Analyze specific path
dendrite --path ./kernel

# Output formats
dendrite --json              # codebase_map.json only
dendrite --markdown          # CODEBASE.md only  
dendrite --all               # Everything + file summaries
```

### Interactive TUI

```bash
dendrite --tui
# or
dendrite -i
```

**Keybindings:**

| Key | Action |
|-----|--------|
| `Tab` | Switch panels (Graph / Files / Alerts) |
| `j/k` or `↑/↓` | Navigate |
| `Enter` | Select file / Expand node |
| `d` | Show dependents (what imports this) |
| `i` | Show imports (what this imports) |
| `p` | Trace path to entry point |
| `/` | Search files |
| `c` | Jump to next cycle |
| `l` | Toggle layer highlighting |
| `?` | Help |
| `q` | Quit |

### Dependency Queries

```bash
# What does this file import?
dendrite deps kernel/scheduler.zig

# What files import this?
dendrite rdeps drivers/pl011.zig

# Find path between two files
dendrite path main.zig pl011.zig

# List all files at a layer
dendrite list --layer driver
```

### CI Integration

```bash
# Fail if cycles detected
dendrite --check

# Fail on cycles OR layer violations
dendrite --check --strict

# Output JSON for further processing
dendrite --check --json > analysis.json
```

### Watch Mode

```bash
# Re-analyze on file changes
dendrite --watch

# Watch with TUI
dendrite --watch --tui
```

## Configuration

Create `dendrite.toml` in your project root:

```toml
[project]
name = "laminae"
src = "src"
exclude = ["tests", "examples", "build"]

[layers]
# Define your architectural layers (order matters: later = deeper)
entry = ["main.zig"]
app = ["shell/*", "init.zig"]
core = ["kernel/*", "memory/*", "scheduler.zig"]
platform = ["platform/*", "exceptions.zig"]
driver = ["drivers/*"]
arch = ["arch/*"]

[rules]
# Custom layer rules (who can import whom)
# By default, layers can only import same level or deeper
allow = [
    "core -> platform",    # Core can use platform abstractions
    "driver -> arch",      # Drivers can use arch-specific code
]
deny = [
    "driver -> core",      # Drivers must not import core
    "arch -> *",           # Arch should be leaf nodes
]

[thresholds]
max_depth = 8              # Warn if dependency chain exceeds this
max_fan_out = 10           # Warn if file imports more than this
max_fan_in = 15            # Warn if file is imported by more than this

[output]
summary_dir = ".dendrite"  # Where to put generated files
include_std = false        # Include @import("std") in graph
```

## Output Files

After running `dendrite --all`:

```
.dendrite/
├── codebase_map.json      # Machine-readable full graph
├── CODEBASE.md            # Human/AI-readable overview
├── metrics.json           # Numerical analysis results
└── summaries/
    ├── main.zig.md
    ├── kernel/
    │   ├── scheduler.zig.md
    │   └── memory.zig.md
    └── drivers/
        └── pl011.zig.md
```

### CODEBASE.md Structure

The generated `CODEBASE.md` is designed to be read by AI agents (Claude Code, Gemini CLI, etc.) at the start of a session:

```markdown
# Project Codebase Map

## Overview
- Total files, max depth, layer distribution
- Active alerts (cycles, violations)

## Architecture
- Mermaid diagram of high-level structure
- Layer descriptions

## File Index
- Table with path, layer, depth, fan-in/out, one-line summary

## Alerts
- Detailed cycle descriptions with line numbers
- Layer violation explanations
- Suggested fixes

## Entry Points
- Where to start reading the code
```

## AI Agent Integration

### Claude Code / Gemini CLI

Add to your project's `CLAUDE.md` or agent instructions:

```markdown
## Codebase Navigation

Before modifying code, read `.dendrite/CODEBASE.md` for:
- Overall architecture and file relationships  
- Which layer each file belongs to
- Current dependency issues to avoid

For detailed context on a specific file, read its summary:
`.dendrite/summaries/{path}.md`

After making changes, run `dendrite --check` to verify no cycles introduced.
```

### Session Close Hook

Add to your workflow (e.g., git pre-commit hook):

```bash
#!/bin/bash
# .git/hooks/pre-commit

# Update codebase map
dendrite --all --quiet

# Stage the updated files
git add .dendrite/

# Fail commit if cycles exist
dendrite --check || {
    echo "❌ Circular imports detected. See .dendrite/CODEBASE.md"
    exit 1
}
```

## Supported Languages

| Language | Status | Import Pattern |
|----------|--------|----------------|
| Zig | ✅ Full | `@import("...")` |
| Assembly (GAS) | ✅ Full | `.include`, `.extern` |
| C | 🚧 Planned | `#include "..."` |
| Rust | 🚧 Planned | `mod`, `use crate::` |

## Roadmap

See [ROADMAP.md](./ROADMAP.md) for detailed development plans.

**Upcoming:**
- [ ] Iced GUI with GPU-accelerated graph rendering
- [ ] sqlite-vec integration for semantic search
- [ ] Language server protocol (LSP) for editor integration
- [ ] Custom rule DSL for complex architectural constraints

## Contributing

Contributions welcome! Please read the roadmap first to see what's in progress.

```bash
# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run -- --path ./test_fixtures

# Run TUI in development
cargo run -- --tui
```

## License

MIT License. See [LICENSE](./LICENSE).

## Acknowledgments

- [petgraph](https://github.com/petgraph/petgraph) — Graph data structures
- [ratatui](https://github.com/ratatui-org/ratatui) — Terminal UI framework
- Inspired by the need to keep OS kernels from becoming spaghetti

---

*Dendrite: See the shape of your code.*
