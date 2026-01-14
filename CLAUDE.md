# CLAUDE.md - Dendrite

## RULE 1 - ABSOLUTE (DO NOT EVER VIOLATE THIS)

You may NOT delete any file or directory unless I explicitly give the exact command **in this session**.

- This includes files you just created (tests, tmp files, scripts, etc.).
- You do not get to decide that something is "safe" to remove.
- If you think something should be removed, stop and ask. You must receive clear written approval **before** any deletion command is even proposed.

Treat "never delete files without permission" as a hard invariant.

---

### IRREVERSIBLE GIT & FILESYSTEM ACTIONS

Absolutely forbidden unless I give the **exact command and explicit approval** in the same message:

- `git reset --hard`
- `git clean -fd`
- `rm -rf`
- Any command that can delete or overwrite code/data

Rules:

1. If you are not 100% sure what a command will delete, do not propose or run it. Ask first.
2. Prefer safe tools: `git status`, `git diff`, `git stash`, copying to backups, etc.
3. After approval, restate the command verbatim, list what it will affect, and wait for confirmation.
4. When a destructive command is run, record in your response:
   - The exact user text authorizing it
   - The command run
   - When you ran it

If that audit trail is missing, then you must act as if the operation never happened.

### Version Updates (SemVer)

When making commits, update the `version` in `Cargo.toml` (workspace root) following [Semantic Versioning](https://semver.org/):

- **MAJOR** (X.0.0): Breaking changes or incompatible API modifications
- **MINOR** (0.X.0): New features, backward-compatible additions
- **PATCH** (0.0.X): Bug fixes, small improvements, documentation

---

### Code Editing Discipline

- Do **not** run scripts that bulk-modify code (codemods, invented one-off scripts, giant `sed`/regex refactors).
- Large mechanical changes: break into smaller, explicit edits and review diffs.
- Subtle/complex changes: edit by hand, file-by-file, with careful reasoning.
- **NO EMOJIS** - do not use emojis or non-textual characters.
- ASCII diagrams are encouraged for visualizing flows.
- Keep in-line comments to a minimum. Use external documentation for complex logic.
- In-line commentary should be value-add, concise, and focused on info not easily gleaned from the code.

---

### No Legacy Code - Full Migrations Only

We optimize for clean architecture, not backwards compatibility. **When we refactor, we fully migrate.**

- No "compat shims", "v2" file clones, or deprecation wrappers
- When changing behavior, migrate ALL callers and remove old code **in the same commit**
- No `_legacy` suffixes, no `_old` prefixes, no "will remove later" comments
- New files are only for genuinely new domains that don't fit existing modules
- The bar for adding files is very high

**Rationale**: Legacy compatibility code creates technical debt that compounds. A clean break is always better than a gradual migration that never completes.

---

## Beads (bd) - Task Management

Beads is a git-backed graph issue tracker. Use `--json` flags for all programmatic operations.

### Session Workflow

```
1. bd prime              # Auto-injected via SessionStart hook
2. bd ready --json       # Find unblocked work
3. bd update <id> --status in_progress --json   # Claim task
4. (do the work)
5. bd close <id> --reason "Done" --json         # Complete task
6. bd sync && git push   # End session - REQUIRED
```

### Key Commands

| Action | Command |
|--------|---------|
| Find ready work | `bd ready --json` |
| Find stale work | `bd stale --days 30 --json` |
| Create issue | `bd create "Title" --description="Context" -t bug\|feature\|task -p 0-4 --json` |
| Create discovered work | `bd create "Found bug" -t bug -p 1 --deps discovered-from:<parent-id> --json` |
| Claim task | `bd update <id> --status in_progress --json` |
| Complete task | `bd close <id> --reason "Done" --json` |
| Find duplicates | `bd duplicates` |
| Merge duplicates | `bd merge <id1> <id2> --into <canonical> --json` |

### Critical Rules

- Always include `--description` when creating issues - context prevents rework
- Use `discovered-from` links to connect work found during implementation
- Run `bd sync` at session end before pushing to git
- **Work is incomplete until `git push` succeeds**
- `.beads/` is authoritative state and **must always be committed** with code changes

### Dependency Thinking

Use requirement language, not temporal language:
```bash
bd dep add rendering layout      # rendering NEEDS layout (correct)
# NOT: bd dep add phase1 phase2   (temporal - inverts direction)
```

### After bd Upgrades

```bash
bd info --whats-new              # Check workflow-impacting changes
bd hooks install                 # Update git hooks
bd daemons killall               # Restart daemons
```

### Context Preservation During Debugging

Long debugging sessions can lose context during compaction. **Commit frequently to preserve investigation state.**

```bash
# During debugging - commit investigation findings periodically
git add -A && git commit -m "WIP: investigating X, found Y"
bd create "Discovered: Z needs fixing" -t bug -p 2 --description="Found while debugging X"
bd sync

# At natural breakpoints (every 30-60 min of active debugging)
bd sync  # Capture bead state changes
git push  # Push to remote
```

**Why this matters:**
- Compaction events lose conversational context but git history persists
- Beads issues survive across sessions - use them to capture findings
- "WIP" commits are fine - squash later when the fix is complete
- A partially-documented investigation beats starting over

---

## Session Completion Checklist

```
[ ] File issues for remaining work (bd create)
[ ] Run quality gates (cargo test, cargo clippy)
[ ] Update issue statuses (bd update/close)
[ ] Run bd sync
[ ] Run git push and verify success
[ ] Confirm git status shows "up to date"
```

**Work is not complete until `git push` succeeds.**

---

## Claude Agents

Specialized agents are available in `.claude/agents/`. Agents use YAML frontmatter format:

```yaml
---
name: agent-name
description: What this agent does
model: sonnet|haiku|opus
tools:
  - Bash
  - Read
  - Edit
---
```

### Available Agents

| Agent | Model | Purpose |
|-------|-------|---------|
| coder-sonnet | sonnet | Fast, precise code changes with atomic commits |
| build-verifier | sonnet | Validates all targets compile and pass tests |
| gemini-analyzer | sonnet | Large-context analysis via Gemini CLI (1M+ context) |

### Disabling Agents

To disable specific agents in `settings.json` or `--disallowedTools`:
```json
{
  "disallowedTools": ["Task(build-verifier)", "Task(gemini-analyzer)"]
}
```

---

## Claude Skills

Skills are invoked via `/skill-name`. Available in `.claude/skills/`.

### Skill Frontmatter (v2.1+)

Skills now support YAML frontmatter with advanced options:

```yaml
---
name: skill-name
description: What this skill does
# Run in forked sub-agent context (isolated from main conversation)
context: fork
# Specify which agent executes this skill
agent: coder-sonnet
---
```

| Field | Description |
|-------|-------------|
| `context: fork` | Run skill in isolated sub-agent context |
| `agent: <name>` | Execute skill using specified agent type |

### Built-in Commands

| Command | Purpose |
|---------|---------|
| `/plan` | Enter plan mode for implementation design |
| `/context` | Manage context files and imports |
| `/help` | Show available commands |

### Project Skills

| Skill | Purpose |
|-------|---------|
| `/test` | Run cargo test with optional filtering |
| `/check` | Run cargo check + clippy on all targets |
| `/verify` | Full build verification before merge/release |

### Skill Hot-Reload

Skills in `.claude/skills/` are automatically discovered without restart. Edit or add skills and they become immediately available.

---

# PROJECT-LANGUAGE-SPECIFIC: Rust (Edition 2021)

## Project Overview

Dendrite is a codebase mapping and dependency analysis tool for systems programmers. It provides:

- **Dependency Graph Visualization** - Interactive TUI with hierarchical layout
- **Cycle Detection** - Find and highlight circular imports before they become problems
- **Layer Enforcement** - Define architectural boundaries and catch violations
- **AI-Ready Output** - Generates `CODEBASE.md` and structured summaries for LLM agents
- **CI Integration** - Exit codes for automated checks in your pipeline

**Key principle**: Built for operating systems and embedded development where dependency discipline is critical. Primary target is Zig codebases, with future support for assembly and C.

---

## Rust Toolchain

- **Rust Edition**: 2021
- **Target**: Native (Windows/Mac/Linux) - single static binary
- **TUI**: Ratatui + crossterm
- **Graph**: petgraph for dependency graph data structures
- **CLI**: clap for argument parsing

### Build Commands

```bash
# Development build
cargo build

# Release build
cargo build --release

# Run dendrite
cargo run -- --path ./test_fixtures

# Run with TUI
cargo run -- --tui

# Run tests
cargo test

# Check all targets (fast feedback)
cargo check --all-targets

# Clippy lints
cargo clippy --all-targets -- -D warnings

# Format code
cargo fmt
```

---

## Architecture

### Key Directories (Planned)

```
dendrite/
  src/
    main.rs              # CLI entry point (clap)
    lib.rs               # Public API surface
    parser/              # Language-specific parsers
      mod.rs             # Parser trait and registry
      zig.rs             # Zig @import extraction
      asm.rs             # Assembly .include/.extern parsing
    graph/               # Dependency graph construction
      mod.rs             # GraphBuilder, DepGraph types
      node.rs            # FileNode, Layer enum
      edge.rs            # Import edge with line info
      analysis.rs        # Cycles, metrics, violations
    output/              # Output generation
      mod.rs             # Output orchestration
      json.rs            # codebase_map.json
      markdown.rs        # CODEBASE.md, file summaries
      metrics.rs         # metrics.json
    config/              # Configuration system
      mod.rs             # Config parsing and validation
      layers.rs          # Layer definitions and rules
    tui/                 # Terminal UI (Ratatui)
      mod.rs             # App state and main loop
      ui.rs              # Layout and rendering
      events.rs          # Keyboard handling
      graph_view.rs      # ASCII graph visualization
      details.rs         # File details panel
      alerts.rs          # Cycles/violations panel
  test_fixtures/         # Sample Zig projects for testing
  templates/             # Handlebars templates for markdown
```

### Design Principles

- **Parallel parsing**: Use rayon for multi-core file processing
- **Clean graph API**: petgraph DiGraph with typed nodes and edges
- **Regex-based parsing**: Fast, simple extraction (not full AST)
- **Layered analysis**: Build graph first, then run analysis passes
- **JSON-first**: All output types implement `Serialize`/`Deserialize`

---

## Core Types

### Parser Types

```rust
pub struct ZigImport {
    pub target: String,     // "scheduler.zig" or "std"
    pub line: usize,        // Line number for error messages
    pub is_std: bool,       // true for @import("std")
}

pub struct ZigFile {
    pub path: PathBuf,
    pub imports: Vec<ZigImport>,
    pub doc_comment: Option<String>,  // //! module docs
    pub exports: Vec<String>,         // pub fn, pub const
    pub loc: usize,                   // Lines of code
}
```

### Graph Types

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Layer {
    Entry,      // main.zig
    App,        // shell/*, init.zig
    Core,       // kernel/*, memory/*
    Platform,   // platform/*, exceptions.zig
    Driver,     // drivers/*
    Arch,       // arch/*
    Unknown,    // Unclassified files
}

pub struct FileNode {
    pub path: PathBuf,
    pub relative_path: String,
    pub layer: Layer,
    pub depth: usize,
    pub summary: Option<String>,
    pub exports: Vec<String>,
    pub loc: usize,
}

pub struct Import {
    pub line: usize,
}

pub type DepGraph = DiGraph<FileNode, Import>;
```

### Analysis Types

```rust
pub struct Cycle {
    pub nodes: Vec<String>,      // File paths in cycle
    pub edges: Vec<(String, String, usize)>,  // (from, to, line)
}

pub struct LayerViolation {
    pub file: String,
    pub imports: String,
    pub from_layer: Layer,
    pub to_layer: Layer,
    pub line: usize,
    pub reason: String,
}

pub struct AnalysisResult {
    pub cycles: Vec<Cycle>,
    pub violations: Vec<LayerViolation>,
    pub max_depth: usize,
    pub deepest_path: Vec<String>,
    pub high_fan_out: Vec<(String, usize)>,
    pub high_fan_in: Vec<(String, usize)>,
    pub orphans: Vec<String>,
}
```

---

## Rust Best Practices

### Error Handling

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DendriteError {
    #[error("Failed to read file: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid import path: {0}")]
    InvalidImport(String),
    #[error("Config error: {0}")]
    Config(String),
    #[error("Unresolved import: {target} in {file}:{line}")]
    UnresolvedImport {
        file: String,
        target: String,
        line: usize,
    },
}

pub type Result<T> = std::result::Result<T, DendriteError>;
```

### Option Handling

```rust
// Use .ok_or() to convert Option to Result
let layer = config.layers.get(&pattern)
    .ok_or_else(|| DendriteError::Config(format!("Unknown layer: {}", pattern)))?;

// Use .unwrap_or_default() for safe defaults
let exclude = config.exclude.clone().unwrap_or_default();
```

### Regex Parsing

```rust
use regex::Regex;
use once_cell::sync::Lazy;

static IMPORT_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"@import\("([^"]+)"\)"#).unwrap()
});

pub fn extract_imports(source: &str) -> Vec<ZigImport> {
    IMPORT_REGEX.captures_iter(source)
        .enumerate()
        .map(|(line, cap)| ZigImport {
            target: cap[1].to_string(),
            line: line + 1,  // 1-indexed
            is_std: &cap[1] == "std",
        })
        .collect()
}
```

### Parallel File Processing

```rust
use rayon::prelude::*;

pub fn parse_all_files(paths: &[PathBuf]) -> Vec<Result<ZigFile>> {
    paths.par_iter()
        .map(|path| parse_file(path))
        .collect()
}
```

---

## TUI Development (Ratatui)

### App State

```rust
pub struct App {
    pub graph: DepGraph,
    pub analysis: AnalysisResult,
    pub selected_node: Option<NodeIndex>,
    pub mode: ViewMode,
    pub panel: ActivePanel,
    pub search_query: String,
    pub viewport: Viewport,
}

pub enum ViewMode {
    Normal,
    Imports,      // Highlight what this file imports
    Dependents,   // Highlight what imports this file
    PathTrace,    // Show path to entry point
    Search,       // Filter by name
}

pub enum ActivePanel {
    Graph,
    Details,
    Alerts,
}
```

### Event Loop

```rust
pub fn run(mut app: App) -> io::Result<()> {
    let mut terminal = setup_terminal()?;

    loop {
        terminal.draw(|f| ui::render(f, &app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match handle_key(key, &mut app) {
                    Some(Action::Quit) => break,
                    Some(action) => apply_action(action, &mut app),
                    None => {}
                }
            }
        }
    }

    restore_terminal(terminal)
}
```

---

## Testing Guidelines

### Test Commands

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_cycle_detection

# Run parser tests only
cargo test parser::

# Run ignored (slow) tests
cargo test -- --ignored
```

### Test Organization

- Unit tests go in the same file as the code (`#[cfg(test)] mod tests`)
- Integration tests go in `tests/` directory
- Test fixtures go in `test_fixtures/` with sample Zig projects

### Test Fixtures

```
test_fixtures/
  simple/               # Basic Zig project, no issues
    main.zig
    lib.zig
  with_cycle/           # Contains circular imports
    a.zig -> b.zig -> a.zig
  layered/              # Multi-layer project
    main.zig
    kernel/
    drivers/
    arch/
```

---

## Common Development Workflows

### Adding a New Language Parser

1. Create `src/parser/{lang}.rs`
2. Implement the `Parser` trait:
   ```rust
   pub trait Parser {
       fn extensions(&self) -> &[&str];
       fn parse(&self, source: &str, path: &Path) -> Result<ParseResult>;
   }
   ```
3. Register in `src/parser/mod.rs`
4. Add test fixtures in `test_fixtures/`
5. Write unit tests

### Adding a New Analysis Pass

1. Add function in `src/graph/analysis.rs`
2. Add result type to `AnalysisResult`
3. Call from `DepGraph::analyze()`
4. Add to JSON/markdown output
5. Write tests with fixture graphs

### Adding a TUI Feature

1. Add state to `App` struct if needed
2. Add keybinding in `events.rs`
3. Update rendering in `ui.rs`
4. Add to help overlay

---

## Roadmap Task IDs

Use task IDs from ROADMAP.md in commit messages:

```bash
git commit -m "1.2.3: Handle imports inside comments"
```

Phases:
1. **Foundation** - CLI, parsing, graph, JSON output
2. **Analysis** - Cycles, metrics, layer enforcement
3. **Markdown** - CODEBASE.md, file summaries
4. **Config** - dendrite.toml, custom layers/rules
5. **TUI** - Interactive graph exploration
6. **Parsers** - Assembly, C support
7. **Advanced** - Watch mode, queries
8. **Distribution** - Cross-platform releases

---

## Bug Severity (Rust)

### Critical - Must Fix Immediately

- `.unwrap()` or `.expect()` on file I/O (panic in release)
- Index out of bounds in graph traversal
- Infinite loop in cycle detection
- Terminal left in raw mode after panic

### Important - Fix Before Merge

- Missing error handling (using `.unwrap()` where `?` should be used)
- Clippy warnings (especially `clippy::pedantic` findings)
- Inconsistent public API (missing `pub` or wrong visibility)
- Missing documentation on public items

### Contextual - Address When Convenient

- TODO/FIXME comments
- Unused imports or variables
- Suboptimal iterator usage
- Missing `#[must_use]` on functions returning Result

---

## Development Philosophy

**Make it work, make it right, make it fast** - in that order.

- Use `dbg!()` macro during development, remove before commit
- Run `cargo clippy` before every commit
- Run `cargo fmt` to maintain consistent style
- Keep dependencies minimal - prefer std library when possible

**The goal**: A fast, reliable dependency analysis tool that helps developers maintain clean codebases.

---

we love you, Claude! do your best today
