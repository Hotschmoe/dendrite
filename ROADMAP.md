# Dendrite Development Roadmap

> Last updated: 2025-01-13

This roadmap breaks development into phases, milestones, and atomic tasks. Each task should be completable in a single focused session (30-90 minutes).

## Legend

- ⬜ Not started
- 🟡 In progress  
- ✅ Complete
- 🔒 Blocked (see notes)
- 💡 Stretch goal

---

## Phase 1: Foundation

**Goal:** Working CLI that parses Zig files and outputs a dependency graph.

### Milestone 1.1: Project Scaffolding

| ID | Task | Status | Notes |
|----|------|--------|-------|
| 1.1.1 | Create Cargo.toml with initial dependencies | ⬜ | petgraph, serde, clap, walkdir, ignore, anyhow, thiserror |
| 1.1.2 | Set up project directory structure | ⬜ | src/{parser,graph,output,config}.rs + mod.rs files |
| 1.1.3 | Create lib.rs with module declarations | ⬜ | Public API surface |
| 1.1.4 | Create main.rs with clap CLI skeleton | ⬜ | --path, --json, --markdown flags |
| 1.1.5 | Add .gitignore for Rust projects | ⬜ | target/, Cargo.lock (for binary), .dendrite/ |
| 1.1.6 | Create test fixtures directory | ⬜ | test_fixtures/ with sample Zig project |
| 1.1.7 | Set up basic integration test harness | ⬜ | tests/integration.rs |

### Milestone 1.2: Zig Parser

| ID | Task | Status | Notes |
|----|------|--------|-------|
| 1.2.1 | Define `ZigImport` struct | ⬜ | target: String, line: usize, is_std: bool |
| 1.2.2 | Implement `extract_imports()` with regex | ⬜ | Match `@import("...")` pattern |
| 1.2.3 | Write unit tests for basic imports | ⬜ | Single import, multiple imports, multiline |
| 1.2.4 | Handle edge cases: comments, strings | ⬜ | Don't match imports inside comments or string literals |
| 1.2.5 | Implement `extract_doc_comment()` | ⬜ | Parse `//!` module-level documentation |
| 1.2.6 | Write tests for doc comment extraction | ⬜ | Empty, single line, multiline |
| 1.2.7 | Implement `extract_public_decls()` | ⬜ | Find `pub fn`, `pub const`, `pub var` |
| 1.2.8 | Write tests for public declaration extraction | ⬜ | Functions, constants, variables, nested |
| 1.2.9 | Create `ZigFile` struct combining all extractions | ⬜ | path, imports, doc_comment, exports, loc |
| 1.2.10 | Implement `parse_file()` orchestrating function | ⬜ | Read file, run all extractors |
| 1.2.11 | Write integration test: parse real Zig file | ⬜ | Use test_fixtures/simple.zig |

### Milestone 1.3: File Discovery

| ID | Task | Status | Notes |
|----|------|--------|-------|
| 1.3.1 | Implement `discover_files()` using walkdir | ⬜ | Find all .zig files recursively |
| 1.3.2 | Add .gitignore support via `ignore` crate | ⬜ | Respect existing ignore patterns |
| 1.3.3 | Add configurable exclude patterns | ⬜ | Skip test/, build/, zig-cache/ |
| 1.3.4 | Write tests for file discovery | ⬜ | Nested dirs, ignored files, symlinks |
| 1.3.5 | Handle file read errors gracefully | ⬜ | Log warning, continue processing |
| 1.3.6 | Implement parallel file parsing | ⬜ | Use rayon for multi-core speedup |

### Milestone 1.4: Graph Construction

| ID | Task | Status | Notes |
|----|------|--------|-------|
| 1.4.1 | Define `FileNode` struct | ⬜ | path, relative_path, layer, summary, exports, loc |
| 1.4.2 | Define `Layer` enum | ⬜ | Entry, App, Core, Platform, Driver, Arch, Unknown |
| 1.4.3 | Define `Import` edge struct | ⬜ | from, to, line number |
| 1.4.4 | Create type alias `DepGraph = DiGraph<FileNode, Import>` | ⬜ | petgraph directed graph |
| 1.4.5 | Implement `GraphBuilder::new()` | ⬜ | Initialize empty graph |
| 1.4.6 | Implement `GraphBuilder::add_file()` | ⬜ | Add node, return NodeIndex |
| 1.4.7 | Implement import path resolution | ⬜ | Resolve relative paths, handle @import("foo.zig") |
| 1.4.8 | Implement `GraphBuilder::add_edges()` | ⬜ | Connect imports to target nodes |
| 1.4.9 | Handle missing import targets | ⬜ | Log warning for unresolved imports |
| 1.4.10 | Handle std library imports | ⬜ | Option to include/exclude std |
| 1.4.11 | Implement `GraphBuilder::build()` | ⬜ | Finalize and return DepGraph |
| 1.4.12 | Write tests for graph construction | ⬜ | Linear chain, diamond, disconnected |

### Milestone 1.5: Basic JSON Output

| ID | Task | Status | Notes |
|----|------|--------|-------|
| 1.5.1 | Define `CodebaseMap` serializable struct | ⬜ | nodes: Vec<FileNode>, edges: Vec<Import>, metadata |
| 1.5.2 | Implement `DepGraph::to_codemap()` | ⬜ | Convert petgraph to serializable format |
| 1.5.3 | Implement JSON serialization | ⬜ | serde_json::to_string_pretty |
| 1.5.4 | Write JSON to file | ⬜ | codebase_map.json |
| 1.5.5 | Write test: serialize and deserialize | ⬜ | Round-trip test |

### Milestone 1.6: CLI Integration

| ID | Task | Status | Notes |
|----|------|--------|-------|
| 1.6.1 | Wire up `--path` argument | ⬜ | Default to current directory |
| 1.6.2 | Wire up `--json` flag | ⬜ | Output JSON only |
| 1.6.3 | Wire up `--output` directory option | ⬜ | Default to .dendrite/ |
| 1.6.4 | Add `--quiet` flag | ⬜ | Suppress stdout, only write files |
| 1.6.5 | Add `--verbose` flag | ⬜ | Debug logging with RUST_LOG |
| 1.6.6 | Print summary stats to stdout | ⬜ | File count, edge count, time elapsed |
| 1.6.7 | End-to-end test: CLI with test fixtures | ⬜ | Verify JSON output matches expected |

**Phase 1 Exit Criteria:**
- `dendrite --path ./test_fixtures --json` produces valid codebase_map.json
- All unit and integration tests pass
- Binary runs on Linux x86_64

---

## Phase 2: Analysis Engine

**Goal:** Detect cycles, compute metrics, identify architectural violations.

### Milestone 2.1: Cycle Detection

| ID | Task | Status | Notes |
|----|------|--------|-------|
| 2.1.1 | Implement `find_cycles()` using Kosaraju's algorithm | ⬜ | petgraph::algo::kosaraju_scc |
| 2.1.2 | Filter SCCs to only multi-node components | ⬜ | Single nodes aren't cycles |
| 2.1.3 | Extract cycle paths (not just nodes) | ⬜ | Show A → B → C → A |
| 2.1.4 | Find specific edges that form cycles | ⬜ | For targeted fix suggestions |
| 2.1.5 | Write tests: no cycles case | ⬜ | DAG should return empty |
| 2.1.6 | Write tests: simple cycle (A ↔ B) | ⬜ | Two-node mutual import |
| 2.1.7 | Write tests: complex cycle (A → B → C → A) | ⬜ | Three+ node cycle |
| 2.1.8 | Write tests: multiple independent cycles | ⬜ | Return all cycles |
| 2.1.9 | Add cycle info to analysis result | ⬜ | cycles: Vec<Cycle> |

### Milestone 2.2: Depth Metrics

| ID | Task | Status | Notes |
|----|------|--------|-------|
| 2.2.1 | Identify entry points (zero in-degree) | ⬜ | Files nothing imports |
| 2.2.2 | Implement BFS depth calculation from entries | ⬜ | Longest path to each node |
| 2.2.3 | Handle cycles in depth calculation | ⬜ | Mark cycle nodes specially |
| 2.2.4 | Find maximum depth across all nodes | ⬜ | max_depth metric |
| 2.2.5 | Find deepest path (entry → leaf) | ⬜ | deepest_path: Vec<String> |
| 2.2.6 | Calculate depth for each node | ⬜ | Store in FileNode or separate map |
| 2.2.7 | Write tests for depth calculation | ⬜ | Linear, branching, with cycles |

### Milestone 2.3: Fan-In/Fan-Out Metrics

| ID | Task | Status | Notes |
|----|------|--------|-------|
| 2.3.1 | Implement `fan_out()` for each node | ⬜ | Count outgoing edges (imports) |
| 2.3.2 | Implement `fan_in()` for each node | ⬜ | Count incoming edges (imported by) |
| 2.3.3 | Find high fan-out files | ⬜ | Sorted list above threshold |
| 2.3.4 | Find high fan-in files | ⬜ | Core dependencies list |
| 2.3.5 | Identify orphan files | ⬜ | Zero fan-in AND zero fan-out |
| 2.3.6 | Add metrics to analysis result | ⬜ | high_fan_out, high_fan_in, orphans |
| 2.3.7 | Write tests for fan metrics | ⬜ | Various graph shapes |

### Milestone 2.4: Layer Classification

| ID | Task | Status | Notes |
|----|------|--------|-------|
| 2.4.1 | Define default layer patterns | ⬜ | main.zig → Entry, drivers/* → Driver, etc. |
| 2.4.2 | Implement glob pattern matching for layers | ⬜ | Use `glob` crate or simple impl |
| 2.4.3 | Classify files based on path patterns | ⬜ | Assign Layer enum to each FileNode |
| 2.4.4 | Handle Unknown layer for unmatched files | ⬜ | Default fallback |
| 2.4.5 | Allow config override for layer assignment | ⬜ | Explicit file → layer mapping |
| 2.4.6 | Write tests for layer classification | ⬜ | Pattern matching edge cases |

### Milestone 2.5: Layer Rule Enforcement

| ID | Task | Status | Notes |
|----|------|--------|-------|
| 2.5.1 | Define `LayerViolation` struct | ⬜ | file, imports, layer_from, layer_to, reason |
| 2.5.2 | Implement default layer rules | ⬜ | Deeper layers can't import shallower |
| 2.5.3 | Check all edges against layer rules | ⬜ | Iterate edges, compare layers |
| 2.5.4 | Generate violation messages | ⬜ | Human-readable explanations |
| 2.5.5 | Support custom allow rules | ⬜ | "driver -> arch" exceptions |
| 2.5.6 | Support custom deny rules | ⬜ | Additional restrictions |
| 2.5.7 | Write tests for layer violations | ⬜ | Valid hierarchy, violations, custom rules |

### Milestone 2.6: Analysis Result Aggregation

| ID | Task | Status | Notes |
|----|------|--------|-------|
| 2.6.1 | Define `AnalysisResult` struct | ⬜ | Combine all metrics and findings |
| 2.6.2 | Implement `DepGraph::analyze()` | ⬜ | Run all analyses, return result |
| 2.6.3 | Add severity levels to findings | ⬜ | Error (cycles), Warning (high fan-out), Info |
| 2.6.4 | Generate fix suggestions for cycles | ⬜ | "Extract shared types to common.zig" |
| 2.6.5 | Generate fix suggestions for violations | ⬜ | "Move X to platform layer or remove import" |
| 2.6.6 | Implement `AnalysisResult::has_errors()` | ⬜ | For CI exit code |
| 2.6.7 | Implement `AnalysisResult::to_json()` | ⬜ | Serializable format |
| 2.6.8 | Write integration test: full analysis | ⬜ | Complex fixture with issues |

### Milestone 2.7: CI Mode

| ID | Task | Status | Notes |
|----|------|--------|-------|
| 2.7.1 | Add `--check` flag to CLI | ⬜ | Exit 1 on cycles |
| 2.7.2 | Add `--strict` flag | ⬜ | Also exit 1 on violations |
| 2.7.3 | Print actionable error messages | ⬜ | File:line format for editor integration |
| 2.7.4 | Support `--check --json` combo | ⬜ | Machine-readable CI output |
| 2.7.5 | Write test: CI pass case | ⬜ | Clean codebase exits 0 |
| 2.7.6 | Write test: CI fail case | ⬜ | Cycle present exits 1 |

**Phase 2 Exit Criteria:**
- `dendrite --check` correctly detects cycles and violations
- All analysis metrics computed and available in JSON
- Fix suggestions generated for detected issues

---

## Phase 3: Markdown Output

**Goal:** Generate human and AI-readable documentation.

### Milestone 3.1: CODEBASE.md Generation

| ID | Task | Status | Notes |
|----|------|--------|-------|
| 3.1.1 | Create handlebars template for CODEBASE.md | ⬜ | templates/CODEBASE.md.hbs |
| 3.1.2 | Implement template data struct | ⬜ | All fields needed for template |
| 3.1.3 | Add overview section generation | ⬜ | File count, depth, layers, alerts |
| 3.1.4 | Add Mermaid diagram generation | ⬜ | flowchart LR with layer styling |
| 3.1.5 | Limit Mermaid to top N important nodes | ⬜ | Avoid huge unreadable diagrams |
| 3.1.6 | Add file index table generation | ⬜ | Sortable columns in markdown |
| 3.1.7 | Add alerts section with details | ⬜ | Cycles with line numbers, violations |
| 3.1.8 | Add layer definitions section | ⬜ | Document the architecture |
| 3.1.9 | Render template to string | ⬜ | handlebars.render() |
| 3.1.10 | Write CODEBASE.md to output dir | ⬜ | .dendrite/CODEBASE.md |
| 3.1.11 | Write test: markdown generation | ⬜ | Verify structure and content |

### Milestone 3.2: File Summaries

| ID | Task | Status | Notes |
|----|------|--------|-------|
| 3.2.1 | Create template for file summary | ⬜ | templates/file_summary.md.hbs |
| 3.2.2 | Include file metadata in summary | ⬜ | Path, layer, depth, loc |
| 3.2.3 | Include doc comment if present | ⬜ | Module-level documentation |
| 3.2.4 | Include imports list with line numbers | ⬜ | What this file depends on |
| 3.2.5 | Include dependents list | ⬜ | What depends on this file |
| 3.2.6 | Include public exports list | ⬜ | API surface |
| 3.2.7 | Mirror directory structure in summaries/ | ⬜ | summaries/kernel/scheduler.zig.md |
| 3.2.8 | Generate all file summaries | ⬜ | Iterate nodes, write files |
| 3.2.9 | Add `--summaries` flag to CLI | ⬜ | Generate only if requested |
| 3.2.10 | Write test: file summary content | ⬜ | Verify all sections present |

### Milestone 3.3: Metrics JSON

| ID | Task | Status | Notes |
|----|------|--------|-------|
| 3.3.1 | Define `Metrics` struct | ⬜ | Numerical analysis data |
| 3.3.2 | Include file count by layer | ⬜ | layer_counts: HashMap<Layer, usize> |
| 3.3.3 | Include depth histogram | ⬜ | How many files at each depth |
| 3.3.4 | Include fan-in/fan-out distributions | ⬜ | Average, median, max |
| 3.3.5 | Include cycle count and sizes | ⬜ | Quick health check |
| 3.3.6 | Include timestamp | ⬜ | When analysis was run |
| 3.3.7 | Write metrics.json to output | ⬜ | .dendrite/metrics.json |

### Milestone 3.4: Output Orchestration

| ID | Task | Status | Notes |
|----|------|--------|-------|
| 3.4.1 | Create output directory if missing | ⬜ | mkdir -p .dendrite |
| 3.4.2 | Implement `--all` flag | ⬜ | JSON + markdown + summaries |
| 3.4.3 | Implement `--markdown` flag | ⬜ | CODEBASE.md only |
| 3.4.4 | Add `--clean` flag | ⬜ | Remove old output before generating |
| 3.4.5 | Print output file paths to stdout | ⬜ | Confirm what was written |
| 3.4.6 | End-to-end test: all outputs | ⬜ | Verify all files created |

**Phase 3 Exit Criteria:**
- `dendrite --all` generates complete .dendrite/ directory
- CODEBASE.md is readable by humans and AI agents
- File summaries provide useful per-file context

---

## Phase 4: Configuration System

**Goal:** User-configurable layers, rules, and thresholds.

### Milestone 4.1: Config File Parsing

| ID | Task | Status | Notes |
|----|------|--------|-------|
| 4.1.1 | Add `toml` dependency | ⬜ | For config parsing |
| 4.1.2 | Define `Config` struct | ⬜ | All configurable options |
| 4.1.3 | Define `ProjectConfig` section | ⬜ | name, src, exclude |
| 4.1.4 | Define `LayerConfig` section | ⬜ | Layer name → glob patterns |
| 4.1.5 | Define `RulesConfig` section | ⬜ | allow, deny lists |
| 4.1.6 | Define `ThresholdsConfig` section | ⬜ | max_depth, max_fan_out, etc. |
| 4.1.7 | Define `OutputConfig` section | ⬜ | summary_dir, include_std |
| 4.1.8 | Implement config file discovery | ⬜ | Look for dendrite.toml |
| 4.1.9 | Implement config parsing | ⬜ | toml::from_str |
| 4.1.10 | Implement default config | ⬜ | Sensible defaults if no file |
| 4.1.11 | Merge CLI args with config file | ⬜ | CLI overrides config |
| 4.1.12 | Write tests for config parsing | ⬜ | Valid config, missing sections, invalid |

### Milestone 4.2: Config Validation

| ID | Task | Status | Notes |
|----|------|--------|-------|
| 4.2.1 | Validate glob patterns are valid | ⬜ | Catch syntax errors early |
| 4.2.2 | Validate layer names are unique | ⬜ | No duplicate definitions |
| 4.2.3 | Validate rules reference valid layers | ⬜ | "driver -> invalid" is error |
| 4.2.4 | Validate thresholds are positive | ⬜ | max_depth > 0 |
| 4.2.5 | Generate helpful error messages | ⬜ | Point to line in config |
| 4.2.6 | Implement `--validate-config` command | ⬜ | Check config without running |

### Milestone 4.3: Init Command

| ID | Task | Status | Notes |
|----|------|--------|-------|
| 4.3.1 | Implement `dendrite init` subcommand | ⬜ | Generate starter config |
| 4.3.2 | Auto-detect project structure | ⬜ | Scan for src/, lib/, etc. |
| 4.3.3 | Suggest layers based on directory names | ⬜ | drivers/ → driver layer |
| 4.3.4 | Write dendrite.toml with comments | ⬜ | Explain each option |
| 4.3.5 | Add `--force` to overwrite existing | ⬜ | Don't clobber by default |

**Phase 4 Exit Criteria:**
- `dendrite init` creates useful starter config
- Custom layers and rules work correctly
- Config errors have helpful messages

---

## Phase 5: Terminal UI (Ratatui)

**Goal:** Interactive graph exploration in the terminal.

### Milestone 5.1: TUI Framework Setup

| ID | Task | Status | Notes |
|----|------|--------|-------|
| 5.1.1 | Add ratatui and crossterm dependencies | ⬜ | TUI framework |
| 5.1.2 | Create `tui/` module structure | ⬜ | app.rs, ui.rs, events.rs |
| 5.1.3 | Implement basic app state struct | ⬜ | graph, selected_node, mode |
| 5.1.4 | Implement terminal setup/teardown | ⬜ | Raw mode, alternate screen |
| 5.1.5 | Implement main event loop | ⬜ | Poll events, update state, render |
| 5.1.6 | Implement graceful exit (q key) | ⬜ | Restore terminal properly |
| 5.1.7 | Handle terminal resize | ⬜ | Redraw on SIGWINCH |
| 5.1.8 | Add panic hook to restore terminal | ⬜ | Don't leave terminal broken |

### Milestone 5.2: Layout System

| ID | Task | Status | Notes |
|----|------|--------|-------|
| 5.2.1 | Design three-panel layout | ⬜ | Graph (main), Details (side), Status (bottom) |
| 5.2.2 | Implement responsive layout | ⬜ | Adjust panels to terminal size |
| 5.2.3 | Implement panel borders and titles | ⬜ | Visual separation |
| 5.2.4 | Implement tab bar for panel switching | ⬜ | [G]raph [F]iles [A]lerts |
| 5.2.5 | Implement status bar | ⬜ | File count, alerts, help hint |

### Milestone 5.3: Graph View (ASCII)

| ID | Task | Status | Notes |
|----|------|--------|-------|
| 5.3.1 | Implement hierarchical layout algorithm | ⬜ | Assign x,y to each node |
| 5.3.2 | Use Sugiyama-style layer assignment | ⬜ | Proper left-to-right layout |
| 5.3.3 | Implement edge routing | ⬜ | Avoid overlapping lines |
| 5.3.4 | Render nodes as boxes with filenames | ⬜ | Unicode box drawing |
| 5.3.5 | Render edges with arrows | ⬜ | ─, │, ┌, └, →, etc. |
| 5.3.6 | Implement layer coloring | ⬜ | Different colors per layer |
| 5.3.7 | Highlight selected node | ⬜ | Reverse video or bright |
| 5.3.8 | Highlight cycles in red | ⬜ | Error color for cycle nodes |
| 5.3.9 | Implement viewport scrolling | ⬜ | Pan for large graphs |
| 5.3.10 | Show truncated filenames for space | ⬜ | scheduler.zi… |

### Milestone 5.4: File Details Panel

| ID | Task | Status | Notes |
|----|------|--------|-------|
| 5.4.1 | Display selected file metadata | ⬜ | Path, layer, depth, loc |
| 5.4.2 | Display imports list | ⬜ | Scrollable if long |
| 5.4.3 | Display dependents list | ⬜ | Who imports this file |
| 5.4.4 | Display exports list | ⬜ | Public API |
| 5.4.5 | Display doc comment/summary | ⬜ | If available |
| 5.4.6 | Highlight if file is in a cycle | ⬜ | Warning banner |

### Milestone 5.5: Alerts Panel

| ID | Task | Status | Notes |
|----|------|--------|-------|
| 5.5.1 | List all cycles with file names | ⬜ | Clickable to jump to graph |
| 5.5.2 | List all layer violations | ⬜ | With explanations |
| 5.5.3 | List threshold warnings | ⬜ | High fan-out, etc. |
| 5.5.4 | Show fix suggestions | ⬜ | Inline help |
| 5.5.5 | Navigate alerts with j/k | ⬜ | Vim-style |
| 5.5.6 | Press Enter to jump to file | ⬜ | Switch to graph, select node |

### Milestone 5.6: Navigation

| ID | Task | Status | Notes |
|----|------|--------|-------|
| 5.6.1 | Implement j/k and arrow key movement | ⬜ | Navigate nodes |
| 5.6.2 | Implement Tab to switch panels | ⬜ | Cycle through panels |
| 5.6.3 | Implement Enter to select/expand | ⬜ | Context-dependent action |
| 5.6.4 | Implement 'd' for dependents mode | ⬜ | Highlight what imports this |
| 5.6.5 | Implement 'i' for imports mode | ⬜ | Highlight what this imports |
| 5.6.6 | Implement 'p' for path trace | ⬜ | Show path to entry point |
| 5.6.7 | Implement 'c' to jump to next cycle | ⬜ | Quick cycle navigation |
| 5.6.8 | Implement 'l' to toggle layer colors | ⬜ | On/off for accessibility |
| 5.6.9 | Implement '/' for search | ⬜ | Filter files by name |
| 5.6.10 | Implement '?' for help overlay | ⬜ | Keybinding reference |

### Milestone 5.7: Search

| ID | Task | Status | Notes |
|----|------|--------|-------|
| 5.7.1 | Implement search input field | ⬜ | Text input at bottom |
| 5.7.2 | Implement fuzzy matching | ⬜ | Match partial filenames |
| 5.7.3 | Show search results as filtered list | ⬜ | Update as you type |
| 5.7.4 | Jump to selected result | ⬜ | Enter selects, Esc cancels |
| 5.7.5 | Highlight matches in graph | ⬜ | Dim non-matching nodes |

### Milestone 5.8: Polish

| ID | Task | Status | Notes |
|----|------|--------|-------|
| 5.8.1 | Add loading indicator for large codebases | ⬜ | Spinner while parsing |
| 5.8.2 | Add error display for parse failures | ⬜ | Show in status bar |
| 5.8.3 | Test on small terminals (80x24) | ⬜ | Ensure usable minimum |
| 5.8.4 | Test on large terminals | ⬜ | Use space well |
| 5.8.5 | Add mouse support (optional) | ⬜ | Click to select nodes |
| 5.8.6 | Profile and optimize rendering | ⬜ | 60fps target |

**Phase 5 Exit Criteria:**
- `dendrite --tui` launches interactive interface
- Can navigate full graph, view file details, see alerts
- Responsive to terminal size, graceful error handling

---

## Phase 6: Additional Parsers

**Goal:** Support assembly and prepare for other languages.

### Milestone 6.1: Assembly Parser (GAS)

| ID | Task | Status | Notes |
|----|------|--------|-------|
| 6.1.1 | Create `parser/asm.rs` module | ⬜ | GNU assembler syntax |
| 6.1.2 | Parse `.include "file"` directives | ⬜ | File includes |
| 6.1.3 | Parse `.extern symbol` directives | ⬜ | External dependencies |
| 6.1.4 | Parse `.global symbol` declarations | ⬜ | Exports |
| 6.1.5 | Handle comments (# and /* */) | ⬜ | Don't match in comments |
| 6.1.6 | Write tests for assembly parsing | ⬜ | ARM, x86 syntax variations |
| 6.1.7 | Integrate into file discovery | ⬜ | .s and .S extensions |
| 6.1.8 | Connect assembly exports to Zig imports | ⬜ | Cross-language edges |

### Milestone 6.2: Parser Abstraction

| ID | Task | Status | Notes |
|----|------|--------|-------|
| 6.2.1 | Define `Parser` trait | ⬜ | parse_file() → ParseResult |
| 6.2.2 | Define `ParseResult` struct | ⬜ | imports, exports, doc, loc |
| 6.2.3 | Implement trait for ZigParser | ⬜ | Wrap existing code |
| 6.2.4 | Implement trait for AsmParser | ⬜ | Wrap assembly parser |
| 6.2.5 | Create parser registry | ⬜ | Extension → Parser mapping |
| 6.2.6 | Auto-select parser by file extension | ⬜ | .zig, .s, .S, .c, etc. |

### Milestone 6.3: C Parser (Stretch)

| ID | Task | Status | Notes |
|----|------|--------|-------|
| 6.3.1 | Create `parser/c.rs` module | 💡 | For mixed codebases |
| 6.3.2 | Parse `#include "file.h"` | 💡 | Local includes only |
| 6.3.3 | Parse function declarations | 💡 | Exports from headers |
| 6.3.4 | Handle include guards | 💡 | Don't double-count |
| 6.3.5 | Write tests for C parsing | 💡 | Headers and sources |

**Phase 6 Exit Criteria:**
- Assembly files included in dependency graph
- Parser system extensible for new languages

---

## Phase 7: Advanced Features

**Goal:** Watch mode, queries, and developer experience.

### Milestone 7.1: Watch Mode

| ID | Task | Status | Notes |
|----|------|--------|-------|
| 7.1.1 | Add `notify` dependency | ⬜ | File system watcher |
| 7.1.2 | Implement `--watch` flag | ⬜ | Re-analyze on changes |
| 7.1.3 | Debounce rapid changes | ⬜ | Don't re-run for every keystroke |
| 7.1.4 | Incremental update (optional) | 💡 | Only re-parse changed files |
| 7.1.5 | Update TUI in watch mode | ⬜ | Live refresh |
| 7.1.6 | Print timestamp on each update | ⬜ | Confirm it's working |

### Milestone 7.2: Query Commands

| ID | Task | Status | Notes |
|----|------|--------|-------|
| 7.2.1 | Implement `dendrite deps <file>` | ⬜ | List imports |
| 7.2.2 | Implement `dendrite rdeps <file>` | ⬜ | List dependents |
| 7.2.3 | Implement `dendrite path <from> <to>` | ⬜ | Find connection path |
| 7.2.4 | Implement `dendrite list --layer X` | ⬜ | Files in layer |
| 7.2.5 | Implement `dendrite cycles` | ⬜ | List all cycles |
| 7.2.6 | Implement `dendrite stats` | ⬜ | Quick metrics summary |
| 7.2.7 | Support JSON output for queries | ⬜ | `--json` on all queries |

### Milestone 7.3: Editor Integration

| ID | Task | Status | Notes |
|----|------|--------|-------|
| 7.3.1 | Output errors in file:line:col format | ⬜ | For editor jump-to |
| 7.3.2 | Support `--format=gcc` | ⬜ | Standard error format |
| 7.3.3 | Support `--format=json` | ⬜ | For IDE plugins |
| 7.3.4 | Document VS Code integration | 💡 | Problem matcher config |
| 7.3.5 | Document Neovim integration | 💡 | Quickfix list |

**Phase 7 Exit Criteria:**
- Watch mode works reliably
- Query commands provide quick answers
- Errors parseable by editors

---

## Phase 8: Distribution

**Goal:** Easy installation across all platforms.

### Milestone 8.1: Cross-Compilation

| ID | Task | Status | Notes |
|----|------|--------|-------|
| 8.1.1 | Set up GitHub Actions workflow | ⬜ | .github/workflows/release.yml |
| 8.1.2 | Build for x86_64-unknown-linux-gnu | ⬜ | Linux x64 |
| 8.1.3 | Build for aarch64-unknown-linux-gnu | ⬜ | Linux ARM64 |
| 8.1.4 | Build for x86_64-apple-darwin | ⬜ | macOS x64 |
| 8.1.5 | Build for aarch64-apple-darwin | ⬜ | macOS ARM64 |
| 8.1.6 | Build for x86_64-pc-windows-msvc | ⬜ | Windows x64 |
| 8.1.7 | Strip binaries for size | ⬜ | strip command |
| 8.1.8 | Compress with UPX (optional) | 💡 | Smaller downloads |

### Milestone 8.2: Release Automation

| ID | Task | Status | Notes |
|----|------|--------|-------|
| 8.2.1 | Trigger build on git tag push | ⬜ | v* pattern |
| 8.2.2 | Generate checksums for binaries | ⬜ | SHA256 sums |
| 8.2.3 | Create GitHub release automatically | ⬜ | With changelog |
| 8.2.4 | Upload all binaries to release | ⬜ | Artifacts |
| 8.2.5 | Generate changelog from commits | 💡 | Conventional commits |

### Milestone 8.3: Installation Methods

| ID | Task | Status | Notes |
|----|------|--------|-------|
| 8.3.1 | Document curl install script | ⬜ | One-liner in README |
| 8.3.2 | Publish to crates.io | ⬜ | `cargo install dendrite` |
| 8.3.3 | Create Homebrew formula | 💡 | macOS package manager |
| 8.3.4 | Create AUR package | 💡 | Arch Linux |
| 8.3.5 | Create Nix flake | 💡 | NixOS |

**Phase 8 Exit Criteria:**
- Single command installation on all platforms
- Automated releases on git tag
- Binary size < 10MB

---

## Phase 9: Stretch Goals

### Milestone 9.1: Iced GUI

| ID | Task | Status | Notes |
|----|------|--------|-------|
| 9.1.1 | Create `gui/` module with Iced | 💡 | GPU-accelerated UI |
| 9.1.2 | Implement graph canvas with pan/zoom | 💡 | Smooth navigation |
| 9.1.3 | Implement node rendering with icons | 💡 | File type icons |
| 9.1.4 | Implement edge rendering with curves | 💡 | Bezier curves |
| 9.1.5 | Implement sidebar with file details | 💡 | Same data as TUI |
| 9.1.6 | Implement search with dropdown | 💡 | Fuzzy matching |
| 9.1.7 | Add dark/light theme toggle | 💡 | User preference |
| 9.1.8 | Export graph as SVG | 💡 | For documentation |

### Milestone 9.2: sqlite-vec Integration

| ID | Task | Status | Notes |
|----|------|--------|-------|
| 9.2.1 | Add sqlite and sqlite-vec dependencies | 💡 | Vector search |
| 9.2.2 | Create embeddings schema | 💡 | file_id, embedding, metadata |
| 9.2.3 | Integrate with OpenRouter API | 💡 | text-embedding-3-small |
| 9.2.4 | Generate embeddings for file summaries | 💡 | Async batch processing |
| 9.2.5 | Implement semantic search query | 💡 | "find files about interrupts" |
| 9.2.6 | Cache embeddings, update on change | 💡 | Incremental updates |
| 9.2.7 | Add `dendrite search "query"` command | 💡 | Natural language queries |

### Milestone 9.3: LSP Integration

| ID | Task | Status | Notes |
|----|------|--------|-------|
| 9.3.1 | Implement LSP server skeleton | 💡 | tower-lsp crate |
| 9.3.2 | Provide diagnostics for cycles | 💡 | Inline warnings |
| 9.3.3 | Provide code lens for import counts | 💡 | "5 files import this" |
| 9.3.4 | Provide hover info for imports | 💡 | Show target file summary |
| 9.3.5 | Document VS Code extension | 💡 | User setup guide |

---

## Task Summary

| Phase | Tasks | Stretch |
|-------|-------|---------|
| 1. Foundation | 45 | 0 |
| 2. Analysis | 41 | 0 |
| 3. Markdown | 22 | 0 |
| 4. Configuration | 17 | 0 |
| 5. TUI | 47 | 0 |
| 6. Parsers | 15 | 5 |
| 7. Advanced | 17 | 3 |
| 8. Distribution | 16 | 5 |
| 9. Stretch | 0 | 22 |
| **Total** | **220** | **35** |

---

## Version Milestones

| Version | Phases | Features |
|---------|--------|----------|
| 0.1.0 | 1 | Basic parsing and JSON output |
| 0.2.0 | 1-2 | Analysis engine, CI mode |
| 0.3.0 | 1-3 | Markdown generation |
| 0.4.0 | 1-4 | Configuration system |
| 0.5.0 | 1-5 | Full TUI |
| 0.6.0 | 1-6 | Assembly support |
| 0.7.0 | 1-7 | Watch mode, queries |
| 1.0.0 | 1-8 | Full release, all platforms |
| 1.x | 9 | GUI, semantic search, LSP |

---

## Getting Started

To begin development:

```bash
# Start with task 1.1.1
cargo new dendrite
cd dendrite

# Follow tasks in order within each milestone
# Complete milestone before moving to next
# Use this file to track progress (change ⬜ to ✅)
```

Each task ID can be used in commit messages:

```bash
git commit -m "1.2.3: Handle imports inside comments"
```

---

*This roadmap is a living document. Update status as tasks complete.*
