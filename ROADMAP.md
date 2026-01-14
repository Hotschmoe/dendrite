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

| ID | Task | Status | Bead | Notes |
|----|------|--------|------|-------|
| 1.1.1 | Create Cargo.toml with initial dependencies | ⬜ | dendrite-uc5 | petgraph, serde, clap, walkdir, ignore, anyhow, thiserror |
| 1.1.2 | Set up project directory structure | ⬜ | dendrite-5u8 | src/{parser,graph,output,config}.rs + mod.rs files |
| 1.1.3 | Create lib.rs with module declarations | ⬜ | dendrite-73g | Public API surface |
| 1.1.4 | Create main.rs with clap CLI skeleton | ⬜ | dendrite-hkj | --path, --json, --markdown flags |
| 1.1.5 | Add .gitignore for Rust projects | ⬜ | dendrite-lhj | target/, Cargo.lock (for binary), .dendrite/ |
| 1.1.6 | Create test fixtures directory | ⬜ | dendrite-4le | test_fixtures/ with sample Zig project |
| 1.1.7 | Set up basic integration test harness | ⬜ | dendrite-gqx | tests/integration.rs |

### Milestone 1.2: Zig Parser

| ID | Task | Status | Bead | Notes |
|----|------|--------|------|-------|
| 1.2.1 | Define `ZigImport` struct | ⬜ | dendrite-ixc | target: String, line: usize, is_std: bool |
| 1.2.2 | Implement `extract_imports()` with regex | ⬜ | dendrite-awo | Match `@import("...")` pattern |
| 1.2.3 | Write unit tests for basic imports | ⬜ | dendrite-ykf | Single import, multiple imports, multiline |
| 1.2.4 | Handle edge cases: comments, strings | ⬜ | dendrite-7zf | Don't match imports inside comments or string literals |
| 1.2.5 | Implement `extract_doc_comment()` | ⬜ | dendrite-93q | Parse `//!` module-level documentation |
| 1.2.6 | Write tests for doc comment extraction | ⬜ | dendrite-dqb | Empty, single line, multiline |
| 1.2.7 | Implement `extract_public_decls()` | ⬜ | dendrite-5ub | Find `pub fn`, `pub const`, `pub var` |
| 1.2.8 | Write tests for public declaration extraction | ⬜ | dendrite-55a | Functions, constants, variables, nested |
| 1.2.9 | Create `ZigFile` struct combining all extractions | ⬜ | dendrite-6ga | path, imports, doc_comment, exports, loc |
| 1.2.10 | Implement `parse_file()` orchestrating function | ⬜ | dendrite-9ip | Read file, run all extractors |
| 1.2.11 | Write integration test: parse real Zig file | ⬜ | dendrite-0xb | Use test_fixtures/simple.zig |

### Milestone 1.3: File Discovery

| ID | Task | Status | Bead | Notes |
|----|------|--------|------|-------|
| 1.3.1 | Implement `discover_files()` using walkdir | ⬜ | dendrite-for | Find all .zig files recursively |
| 1.3.2 | Add .gitignore support via `ignore` crate | ⬜ | dendrite-nzb | Respect existing ignore patterns |
| 1.3.3 | Add configurable exclude patterns | ⬜ | dendrite-b5k | Skip test/, build/, zig-cache/ |
| 1.3.4 | Write tests for file discovery | ⬜ | dendrite-zmh | Nested dirs, ignored files, symlinks |
| 1.3.5 | Handle file read errors gracefully | ⬜ | dendrite-u21 | Log warning, continue processing |
| 1.3.6 | Implement parallel file parsing | ⬜ | dendrite-6j5 | Use rayon for multi-core speedup |

### Milestone 1.4: Graph Construction

| ID | Task | Status | Bead | Notes |
|----|------|--------|------|-------|
| 1.4.1 | Define `FileNode` struct | ⬜ | dendrite-fhu | path, relative_path, layer, summary, exports, loc |
| 1.4.2 | Define `Layer` enum | ⬜ | dendrite-y21 | Entry, App, Core, Platform, Driver, Arch, Unknown |
| 1.4.3 | Define `Import` edge struct | ⬜ | dendrite-hr1 | from, to, line number |
| 1.4.4 | Create type alias `DepGraph = DiGraph<FileNode, Import>` | ⬜ | dendrite-2n5 | petgraph directed graph |
| 1.4.5 | Implement `GraphBuilder::new()` | ⬜ | dendrite-mqx | Initialize empty graph |
| 1.4.6 | Implement `GraphBuilder::add_file()` | ⬜ | dendrite-s4c | Add node, return NodeIndex |
| 1.4.7 | Implement import path resolution | ⬜ | dendrite-57f | Resolve relative paths, handle @import("foo.zig") |
| 1.4.8 | Implement `GraphBuilder::add_edges()` | ⬜ | dendrite-wf7 | Connect imports to target nodes |
| 1.4.9 | Handle missing import targets | ⬜ | dendrite-ajk | Log warning for unresolved imports |
| 1.4.10 | Handle std library imports | ⬜ | dendrite-col | Option to include/exclude std |
| 1.4.11 | Implement `GraphBuilder::build()` | ⬜ | dendrite-28s | Finalize and return DepGraph |
| 1.4.12 | Write tests for graph construction | ⬜ | dendrite-lnu | Linear chain, diamond, disconnected |

### Milestone 1.5: Basic JSON Output

| ID | Task | Status | Bead | Notes |
|----|------|--------|------|-------|
| 1.5.1 | Define `CodebaseMap` serializable struct | ⬜ | dendrite-q2c | nodes: Vec<FileNode>, edges: Vec<Import>, metadata |
| 1.5.2 | Implement `DepGraph::to_codemap()` | ⬜ | dendrite-jew | Convert petgraph to serializable format |
| 1.5.3 | Implement JSON serialization | ⬜ | dendrite-1on | serde_json::to_string_pretty |
| 1.5.4 | Write JSON to file | ⬜ | dendrite-pzv | codebase_map.json |
| 1.5.5 | Write test: serialize and deserialize | ⬜ | dendrite-mdo | Round-trip test |

### Milestone 1.6: CLI Integration

| ID | Task | Status | Bead | Notes |
|----|------|--------|------|-------|
| 1.6.1 | Wire up `--path` argument | ⬜ | dendrite-454 | Default to current directory |
| 1.6.2 | Wire up `--json` flag | ⬜ | dendrite-dr8 | Output JSON only |
| 1.6.3 | Wire up `--output` directory option | ⬜ | dendrite-flm | Default to .dendrite/ |
| 1.6.4 | Add `--quiet` flag | ⬜ | dendrite-ama | Suppress stdout, only write files |
| 1.6.5 | Add `--verbose` flag | ⬜ | dendrite-vzu | Debug logging with RUST_LOG |
| 1.6.6 | Print summary stats to stdout | ⬜ | dendrite-7lu | File count, edge count, time elapsed |
| 1.6.7 | End-to-end test: CLI with test fixtures | ⬜ | dendrite-lv3 | Verify JSON output matches expected |

**Phase 1 Exit Criteria:**
- `dendrite --path ./test_fixtures --json` produces valid codebase_map.json
- All unit and integration tests pass
- Binary runs on Linux x86_64

---

## Phase 2: Analysis Engine

**Goal:** Detect cycles, compute metrics, identify architectural violations.

### Milestone 2.1: Cycle Detection

| ID | Task | Status | Bead | Notes |
|----|------|--------|------|-------|
| 2.1.1 | Implement `find_cycles()` using Kosaraju's algorithm | ⬜ | dendrite-jqd | petgraph::algo::kosaraju_scc |
| 2.1.2 | Filter SCCs to only multi-node components | ⬜ | dendrite-2e8 | Single nodes aren't cycles |
| 2.1.3 | Extract cycle paths (not just nodes) | ⬜ | dendrite-td0 | Show A → B → C → A |
| 2.1.4 | Find specific edges that form cycles | ⬜ | dendrite-3v8 | For targeted fix suggestions |
| 2.1.5 | Write tests: no cycles case | ⬜ | dendrite-mh8 | DAG should return empty |
| 2.1.6 | Write tests: simple cycle (A ↔ B) | ⬜ | dendrite-dk4 | Two-node mutual import |
| 2.1.7 | Write tests: complex cycle (A → B → C → A) | ⬜ | dendrite-etb | Three+ node cycle |
| 2.1.8 | Write tests: multiple independent cycles | ⬜ | dendrite-02p | Return all cycles |
| 2.1.9 | Add cycle info to analysis result | ⬜ | dendrite-pkp | cycles: Vec<Cycle> |

### Milestone 2.2: Depth Metrics

| ID | Task | Status | Bead | Notes |
|----|------|--------|------|-------|
| 2.2.1 | Identify entry points (zero in-degree) | ⬜ | dendrite-zy4 | Files nothing imports |
| 2.2.2 | Implement BFS depth calculation from entries | ⬜ | dendrite-txd | Longest path to each node |
| 2.2.3 | Handle cycles in depth calculation | ⬜ | dendrite-b0k | Mark cycle nodes specially |
| 2.2.4 | Find maximum depth across all nodes | ⬜ | dendrite-jxe | max_depth metric |
| 2.2.5 | Find deepest path (entry → leaf) | ⬜ | dendrite-dxs | deepest_path: Vec<String> |
| 2.2.6 | Calculate depth for each node | ⬜ | dendrite-020 | Store in FileNode or separate map |
| 2.2.7 | Write tests for depth calculation | ⬜ | dendrite-0bp | Linear, branching, with cycles |

### Milestone 2.3: Fan-In/Fan-Out Metrics

| ID | Task | Status | Bead | Notes |
|----|------|--------|------|-------|
| 2.3.1 | Implement `fan_out()` for each node | ⬜ | dendrite-cmh | Count outgoing edges (imports) |
| 2.3.2 | Implement `fan_in()` for each node | ⬜ | dendrite-1uw | Count incoming edges (imported by) |
| 2.3.3 | Find high fan-out files | ⬜ | dendrite-f91 | Sorted list above threshold |
| 2.3.4 | Find high fan-in files | ⬜ | dendrite-gwe | Core dependencies list |
| 2.3.5 | Identify orphan files | ⬜ | dendrite-tjn | Zero fan-in AND zero fan-out |
| 2.3.6 | Add metrics to analysis result | ⬜ | dendrite-gcm | high_fan_out, high_fan_in, orphans |
| 2.3.7 | Write tests for fan metrics | ⬜ | dendrite-bof | Various graph shapes |

### Milestone 2.4: Layer Classification

| ID | Task | Status | Bead | Notes |
|----|------|--------|------|-------|
| 2.4.1 | Define default layer patterns | ⬜ | dendrite-k86 | main.zig → Entry, drivers/* → Driver, etc. |
| 2.4.2 | Implement glob pattern matching for layers | ⬜ | dendrite-wik | Use `glob` crate or simple impl |
| 2.4.3 | Classify files based on path patterns | ⬜ | dendrite-76u | Assign Layer enum to each FileNode |
| 2.4.4 | Handle Unknown layer for unmatched files | ⬜ | dendrite-aiq | Default fallback |
| 2.4.5 | Allow config override for layer assignment | ⬜ | dendrite-nfq | Explicit file → layer mapping |
| 2.4.6 | Write tests for layer classification | ⬜ | dendrite-aha | Pattern matching edge cases |

### Milestone 2.5: Layer Rule Enforcement

| ID | Task | Status | Bead | Notes |
|----|------|--------|------|-------|
| 2.5.1 | Define `LayerViolation` struct | ⬜ | dendrite-kyt | file, imports, layer_from, layer_to, reason |
| 2.5.2 | Implement default layer rules | ⬜ | dendrite-rgl | Deeper layers can't import shallower |
| 2.5.3 | Check all edges against layer rules | ⬜ | dendrite-aqe | Iterate edges, compare layers |
| 2.5.4 | Generate violation messages | ⬜ | dendrite-dvn | Human-readable explanations |
| 2.5.5 | Support custom allow rules | ⬜ | dendrite-4o1 | "driver -> arch" exceptions |
| 2.5.6 | Support custom deny rules | ⬜ | dendrite-9x9 | Additional restrictions |
| 2.5.7 | Write tests for layer violations | ⬜ | dendrite-ep5 | Valid hierarchy, violations, custom rules |

### Milestone 2.6: Analysis Result Aggregation

| ID | Task | Status | Bead | Notes |
|----|------|--------|------|-------|
| 2.6.1 | Define `AnalysisResult` struct | ⬜ | dendrite-9nb | Combine all metrics and findings |
| 2.6.2 | Implement `DepGraph::analyze()` | ⬜ | dendrite-ojh | Run all analyses, return result |
| 2.6.3 | Add severity levels to findings | ⬜ | dendrite-kbo | Error (cycles), Warning (high fan-out), Info |
| 2.6.4 | Generate fix suggestions for cycles | ⬜ | dendrite-2ck | "Extract shared types to common.zig" |
| 2.6.5 | Generate fix suggestions for violations | ⬜ | dendrite-sly | "Move X to platform layer or remove import" |
| 2.6.6 | Implement `AnalysisResult::has_errors()` | ⬜ | dendrite-0fj | For CI exit code |
| 2.6.7 | Implement `AnalysisResult::to_json()` | ⬜ | dendrite-gz2 | Serializable format |
| 2.6.8 | Write integration test: full analysis | ⬜ | dendrite-yww | Complex fixture with issues |

### Milestone 2.7: CI Mode

| ID | Task | Status | Bead | Notes |
|----|------|--------|------|-------|
| 2.7.1 | Add `--check` flag to CLI | ⬜ | dendrite-5xr | Exit 1 on cycles |
| 2.7.2 | Add `--strict` flag | ⬜ | dendrite-ldr | Also exit 1 on violations |
| 2.7.3 | Print actionable error messages | ⬜ | dendrite-aye | File:line format for editor integration |
| 2.7.4 | Support `--check --json` combo | ⬜ | dendrite-avx | Machine-readable CI output |
| 2.7.5 | Write test: CI pass case | ⬜ | dendrite-xu2 | Clean codebase exits 0 |
| 2.7.6 | Write test: CI fail case | ⬜ | dendrite-2gp | Cycle present exits 1 |

**Phase 2 Exit Criteria:**
- `dendrite --check` correctly detects cycles and violations
- All analysis metrics computed and available in JSON
- Fix suggestions generated for detected issues

---

## Phase 3: Markdown Output

**Goal:** Generate human and AI-readable documentation.

### Milestone 3.1: CODEBASE.md Generation

| ID | Task | Status | Bead | Notes |
|----|------|--------|------|-------|
| 3.1.1 | Create handlebars template for CODEBASE.md | ⬜ | dendrite-0z2 | templates/CODEBASE.md.hbs |
| 3.1.2 | Implement template data struct | ⬜ | dendrite-lw4 | All fields needed for template |
| 3.1.3 | Add overview section generation | ⬜ | dendrite-6rv | File count, depth, layers, alerts |
| 3.1.4 | Add Mermaid diagram generation | ⬜ | dendrite-p67 | flowchart LR with layer styling |
| 3.1.5 | Limit Mermaid to top N important nodes | ⬜ | dendrite-k5g | Avoid huge unreadable diagrams |
| 3.1.6 | Add file index table generation | ⬜ | dendrite-88q | Sortable columns in markdown |
| 3.1.7 | Add alerts section with details | ⬜ | dendrite-auq | Cycles with line numbers, violations |
| 3.1.8 | Add layer definitions section | ⬜ | dendrite-6dt | Document the architecture |
| 3.1.9 | Render template to string | ⬜ | dendrite-iz7 | handlebars.render() |
| 3.1.10 | Write CODEBASE.md to output dir | ⬜ | dendrite-h6o | .dendrite/CODEBASE.md |
| 3.1.11 | Write test: markdown generation | ⬜ | dendrite-6gm | Verify structure and content |

### Milestone 3.2: File Summaries

| ID | Task | Status | Bead | Notes |
|----|------|--------|------|-------|
| 3.2.1 | Create template for file summary | ⬜ | dendrite-9xe | templates/file_summary.md.hbs |
| 3.2.2 | Include file metadata in summary | ⬜ | dendrite-51h | Path, layer, depth, loc |
| 3.2.3 | Include doc comment if present | ⬜ | dendrite-13n | Module-level documentation |
| 3.2.4 | Include imports list with line numbers | ⬜ | dendrite-97z | What this file depends on |
| 3.2.5 | Include dependents list | ⬜ | dendrite-llm | What depends on this file |
| 3.2.6 | Include public exports list | ⬜ | dendrite-1jp | API surface |
| 3.2.7 | Mirror directory structure in summaries/ | ⬜ | dendrite-6h4 | summaries/kernel/scheduler.zig.md |
| 3.2.8 | Generate all file summaries | ⬜ | dendrite-k19 | Iterate nodes, write files |
| 3.2.9 | Add `--summaries` flag to CLI | ⬜ | dendrite-y3x | Generate only if requested |
| 3.2.10 | Write test: file summary content | ⬜ | dendrite-6f5 | Verify all sections present |

### Milestone 3.3: Metrics JSON

| ID | Task | Status | Bead | Notes |
|----|------|--------|------|-------|
| 3.3.1 | Define `Metrics` struct | ⬜ | dendrite-3o8 | Numerical analysis data |
| 3.3.2 | Include file count by layer | ⬜ | dendrite-rw6 | layer_counts: HashMap<Layer, usize> |
| 3.3.3 | Include depth histogram | ⬜ | dendrite-qbg | How many files at each depth |
| 3.3.4 | Include fan-in/fan-out distributions | ⬜ | dendrite-d1f | Average, median, max |
| 3.3.5 | Include cycle count and sizes | ⬜ | dendrite-wo6 | Quick health check |
| 3.3.6 | Include timestamp | ⬜ | dendrite-831 | When analysis was run |
| 3.3.7 | Write metrics.json to output | ⬜ | dendrite-lra | .dendrite/metrics.json |

### Milestone 3.4: Output Orchestration

| ID | Task | Status | Bead | Notes |
|----|------|--------|------|-------|
| 3.4.1 | Create output directory if missing | ⬜ | dendrite-cu8 | mkdir -p .dendrite |
| 3.4.2 | Implement `--all` flag | ⬜ | dendrite-zup | JSON + markdown + summaries |
| 3.4.3 | Implement `--markdown` flag | ⬜ | dendrite-zsa | CODEBASE.md only |
| 3.4.4 | Add `--clean` flag | ⬜ | dendrite-f2y | Remove old output before generating |
| 3.4.5 | Print output file paths to stdout | ⬜ | dendrite-oe1 | Confirm what was written |
| 3.4.6 | End-to-end test: all outputs | ⬜ | dendrite-5r9 | Verify all files created |

**Phase 3 Exit Criteria:**
- `dendrite --all` generates complete .dendrite/ directory
- CODEBASE.md is readable by humans and AI agents
- File summaries provide useful per-file context

---

## Phase 4: Configuration System

**Goal:** User-configurable layers, rules, and thresholds.

### Milestone 4.1: Config File Parsing

| ID | Task | Status | Bead | Notes |
|----|------|--------|------|-------|
| 4.1.1 | Add `toml` dependency | ⬜ | dendrite-r9p | For config parsing |
| 4.1.2 | Define `Config` struct | ⬜ | dendrite-75l | All configurable options |
| 4.1.3 | Define `ProjectConfig` section | ⬜ | dendrite-3c0 | name, src, exclude |
| 4.1.4 | Define `LayerConfig` section | ⬜ | dendrite-w29 | Layer name → glob patterns |
| 4.1.5 | Define `RulesConfig` section | ⬜ | dendrite-vnh | allow, deny lists |
| 4.1.6 | Define `ThresholdsConfig` section | ⬜ | dendrite-hh8 | max_depth, max_fan_out, etc. |
| 4.1.7 | Define `OutputConfig` section | ⬜ | dendrite-8x0 | summary_dir, include_std |
| 4.1.8 | Implement config file discovery | ⬜ | dendrite-7us | Look for dendrite.toml |
| 4.1.9 | Implement config parsing | ⬜ | dendrite-xui | toml::from_str |
| 4.1.10 | Implement default config | ⬜ | dendrite-wy1 | Sensible defaults if no file |
| 4.1.11 | Merge CLI args with config file | ⬜ | dendrite-zhx | CLI overrides config |
| 4.1.12 | Write tests for config parsing | ⬜ | dendrite-vnm | Valid config, missing sections, invalid |

### Milestone 4.2: Config Validation

| ID | Task | Status | Bead | Notes |
|----|------|--------|------|-------|
| 4.2.1 | Validate glob patterns are valid | ⬜ | dendrite-3a8 | Catch syntax errors early |
| 4.2.2 | Validate layer names are unique | ⬜ | dendrite-gy8 | No duplicate definitions |
| 4.2.3 | Validate rules reference valid layers | ⬜ | dendrite-k2p | "driver -> invalid" is error |
| 4.2.4 | Validate thresholds are positive | ⬜ | dendrite-heq | max_depth > 0 |
| 4.2.5 | Generate helpful error messages | ⬜ | dendrite-6m5 | Point to line in config |
| 4.2.6 | Implement `--validate-config` command | ⬜ | dendrite-19x | Check config without running |

### Milestone 4.3: Init Command

| ID | Task | Status | Bead | Notes |
|----|------|--------|------|-------|
| 4.3.1 | Implement `dendrite init` subcommand | ⬜ | dendrite-s7i | Generate starter config |
| 4.3.2 | Auto-detect project structure | ⬜ | dendrite-4kk | Scan for src/, lib/, etc. |
| 4.3.3 | Suggest layers based on directory names | ⬜ | dendrite-xtn | drivers/ → driver layer |
| 4.3.4 | Write dendrite.toml with comments | ⬜ | dendrite-28w | Explain each option |
| 4.3.5 | Add `--force` to overwrite existing | ⬜ | dendrite-iob | Don't clobber by default |

**Phase 4 Exit Criteria:**
- `dendrite init` creates useful starter config
- Custom layers and rules work correctly
- Config errors have helpful messages

---

## Phase 5: Terminal UI (Ratatui)

**Goal:** Interactive graph exploration in the terminal.

### Milestone 5.1: TUI Framework Setup

| ID | Task | Status | Bead | Notes |
|----|------|--------|------|-------|
| 5.1.1 | Add ratatui and crossterm dependencies | ⬜ | dendrite-wmf | TUI framework |
| 5.1.2 | Create `tui/` module structure | ⬜ | dendrite-zqp | app.rs, ui.rs, events.rs |
| 5.1.3 | Implement basic app state struct | ⬜ | dendrite-bwf | graph, selected_node, mode |
| 5.1.4 | Implement terminal setup/teardown | ⬜ | dendrite-t99 | Raw mode, alternate screen |
| 5.1.5 | Implement main event loop | ⬜ | dendrite-c9t | Poll events, update state, render |
| 5.1.6 | Implement graceful exit (q key) | ⬜ | dendrite-b8f | Restore terminal properly |
| 5.1.7 | Handle terminal resize | ⬜ | dendrite-xr8 | Redraw on SIGWINCH |
| 5.1.8 | Add panic hook to restore terminal | ⬜ | dendrite-9hk | Don't leave terminal broken |

### Milestone 5.2: Layout System

| ID | Task | Status | Bead | Notes |
|----|------|--------|------|-------|
| 5.2.1 | Design three-panel layout | ⬜ | dendrite-hos | Graph (main), Details (side), Status (bottom) |
| 5.2.2 | Implement responsive layout | ⬜ | dendrite-vr0k | Adjust panels to terminal size |
| 5.2.3 | Implement panel borders and titles | ⬜ | dendrite-h1ny | Visual separation |
| 5.2.4 | Implement tab bar for panel switching | ⬜ | dendrite-ewlw | [G]raph [F]iles [A]lerts |
| 5.2.5 | Implement status bar | ⬜ | dendrite-yg0n | File count, alerts, help hint |

### Milestone 5.3: Graph View (ASCII)

| ID | Task | Status | Bead | Notes |
|----|------|--------|------|-------|
| 5.3.1 | Implement hierarchical layout algorithm | ⬜ | dendrite-o44c | Assign x,y to each node |
| 5.3.2 | Use Sugiyama-style layer assignment | ⬜ | dendrite-mtuw | Proper left-to-right layout |
| 5.3.3 | Implement edge routing | ⬜ | dendrite-ibgg | Avoid overlapping lines |
| 5.3.4 | Render nodes as boxes with filenames | ⬜ | dendrite-g33g | Unicode box drawing |
| 5.3.5 | Render edges with arrows | ⬜ | dendrite-dc92 | ─, │, ┌, └, →, etc. |
| 5.3.6 | Implement layer coloring | ⬜ | dendrite-sz8v | Different colors per layer |
| 5.3.7 | Highlight selected node | ⬜ | dendrite-mgnc | Reverse video or bright |
| 5.3.8 | Highlight cycles in red | ⬜ | dendrite-kjkt | Error color for cycle nodes |
| 5.3.9 | Implement viewport scrolling | ⬜ | dendrite-g47p | Pan for large graphs |
| 5.3.10 | Show truncated filenames for space | ⬜ | dendrite-mkxq | scheduler.zi… |

### Milestone 5.4: File Details Panel

| ID | Task | Status | Bead | Notes |
|----|------|--------|------|-------|
| 5.4.1 | Display selected file metadata | ⬜ | dendrite-gfg6 | Path, layer, depth, loc |
| 5.4.2 | Display imports list | ⬜ | dendrite-n3ml | Scrollable if long |
| 5.4.3 | Display dependents list | ⬜ | dendrite-xjw9 | Who imports this file |
| 5.4.4 | Display exports list | ⬜ | dendrite-w8dz | Public API |
| 5.4.5 | Display doc comment/summary | ⬜ | dendrite-ezs6 | If available |
| 5.4.6 | Highlight if file is in a cycle | ⬜ | dendrite-yi41 | Warning banner |

### Milestone 5.5: Alerts Panel

| ID | Task | Status | Bead | Notes |
|----|------|--------|------|-------|
| 5.5.1 | List all cycles with file names | ⬜ | dendrite-osn0 | Clickable to jump to graph |
| 5.5.2 | List all layer violations | ⬜ | dendrite-y7c2 | With explanations |
| 5.5.3 | List threshold warnings | ⬜ | dendrite-8wu0 | High fan-out, etc. |
| 5.5.4 | Show fix suggestions | ⬜ | dendrite-pnho | Inline help |
| 5.5.5 | Navigate alerts with j/k | ⬜ | dendrite-q6uv | Vim-style |
| 5.5.6 | Press Enter to jump to file | ⬜ | dendrite-jxtd | Switch to graph, select node |

### Milestone 5.6: Navigation

| ID | Task | Status | Bead | Notes |
|----|------|--------|------|-------|
| 5.6.1 | Implement j/k and arrow key movement | ⬜ | dendrite-h1wx | Navigate nodes |
| 5.6.2 | Implement Tab to switch panels | ⬜ | dendrite-j0w5 | Cycle through panels |
| 5.6.3 | Implement Enter to select/expand | ⬜ | dendrite-3ni7 | Context-dependent action |
| 5.6.4 | Implement 'd' for dependents mode | ⬜ | dendrite-37mg | Highlight what imports this |
| 5.6.5 | Implement 'i' for imports mode | ⬜ | dendrite-euna | Highlight what this imports |
| 5.6.6 | Implement 'p' for path trace | ⬜ | dendrite-mok0 | Show path to entry point |
| 5.6.7 | Implement 'c' to jump to next cycle | ⬜ | dendrite-rgsi | Quick cycle navigation |
| 5.6.8 | Implement 'l' to toggle layer colors | ⬜ | dendrite-lmmf | On/off for accessibility |
| 5.6.9 | Implement '/' for search | ⬜ | dendrite-peht | Filter files by name |
| 5.6.10 | Implement '?' for help overlay | ⬜ | dendrite-32rq | Keybinding reference |

### Milestone 5.7: Search

| ID | Task | Status | Bead | Notes |
|----|------|--------|------|-------|
| 5.7.1 | Implement search input field | ⬜ | dendrite-frw3 | Text input at bottom |
| 5.7.2 | Implement fuzzy matching | ⬜ | dendrite-ndey | Match partial filenames |
| 5.7.3 | Show search results as filtered list | ⬜ | dendrite-nji3 | Update as you type |
| 5.7.4 | Jump to selected result | ⬜ | dendrite-7cwk | Enter selects, Esc cancels |
| 5.7.5 | Highlight matches in graph | ⬜ | dendrite-o93b | Dim non-matching nodes |

### Milestone 5.8: Polish

| ID | Task | Status | Bead | Notes |
|----|------|--------|------|-------|
| 5.8.1 | Add loading indicator for large codebases | ⬜ | dendrite-tids | Spinner while parsing |
| 5.8.2 | Add error display for parse failures | ⬜ | dendrite-9tuj | Show in status bar |
| 5.8.3 | Test on small terminals (80x24) | ⬜ | dendrite-8u34 | Ensure usable minimum |
| 5.8.4 | Test on large terminals | ⬜ | dendrite-sqbr | Use space well |
| 5.8.5 | Add mouse support (optional) | ⬜ | dendrite-9gwk | Click to select nodes |
| 5.8.6 | Profile and optimize rendering | ⬜ | dendrite-fxwd | 60fps target |

**Phase 5 Exit Criteria:**
- `dendrite --tui` launches interactive interface
- Can navigate full graph, view file details, see alerts
- Responsive to terminal size, graceful error handling

---

## Phase 6: Additional Parsers

**Goal:** Support assembly and prepare for other languages.

### Milestone 6.1: Assembly Parser (GAS)

| ID | Task | Status | Bead | Notes |
|----|------|--------|------|-------|
| 6.1.1 | Create `parser/asm.rs` module | ⬜ | dendrite-2v0t | GNU assembler syntax |
| 6.1.2 | Parse `.include "file"` directives | ⬜ | dendrite-2hhi | File includes |
| 6.1.3 | Parse `.extern symbol` directives | ⬜ | dendrite-moy1 | External dependencies |
| 6.1.4 | Parse `.global symbol` declarations | ⬜ | dendrite-slk6 | Exports |
| 6.1.5 | Handle comments (# and /* */) | ⬜ | dendrite-uizl | Don't match in comments |
| 6.1.6 | Write tests for assembly parsing | ⬜ | dendrite-93ly | ARM, x86 syntax variations |
| 6.1.7 | Integrate into file discovery | ⬜ | dendrite-sws0 | .s and .S extensions |
| 6.1.8 | Connect assembly exports to Zig imports | ⬜ | dendrite-nk2c | Cross-language edges |

### Milestone 6.2: Parser Abstraction

| ID | Task | Status | Bead | Notes |
|----|------|--------|------|-------|
| 6.2.1 | Define `Parser` trait | ⬜ | dendrite-y6sj | parse_file() → ParseResult |
| 6.2.2 | Define `ParseResult` struct | ⬜ | dendrite-wfvq | imports, exports, doc, loc |
| 6.2.3 | Implement trait for ZigParser | ⬜ | dendrite-fmti | Wrap existing code |
| 6.2.4 | Implement trait for AsmParser | ⬜ | dendrite-aezu | Wrap assembly parser |
| 6.2.5 | Create parser registry | ⬜ | dendrite-x799 | Extension → Parser mapping |
| 6.2.6 | Auto-select parser by file extension | ⬜ | dendrite-nj8o | .zig, .s, .S, .c, etc. |

### Milestone 6.3: C Parser (Stretch)

| ID | Task | Status | Bead | Notes |
|----|------|--------|------|-------|
| 6.3.1 | Create `parser/c.rs` module | 💡 | dendrite-k669 | For mixed codebases |
| 6.3.2 | Parse `#include "file.h"` | 💡 | dendrite-wb17 | Local includes only |
| 6.3.3 | Parse function declarations | 💡 | dendrite-dq7p | Exports from headers |
| 6.3.4 | Handle include guards | 💡 | dendrite-vfbe | Don't double-count |
| 6.3.5 | Write tests for C parsing | 💡 | dendrite-bj68 | Headers and sources |

**Phase 6 Exit Criteria:**
- Assembly files included in dependency graph
- Parser system extensible for new languages

---

## Phase 7: Advanced Features

**Goal:** Watch mode, queries, and developer experience.

### Milestone 7.1: Watch Mode

| ID | Task | Status | Bead | Notes |
|----|------|--------|------|-------|
| 7.1.1 | Add `notify` dependency | ⬜ | dendrite-2bpb | File system watcher |
| 7.1.2 | Implement `--watch` flag | ⬜ | dendrite-slrj | Re-analyze on changes |
| 7.1.3 | Debounce rapid changes | ⬜ | dendrite-hna9 | Don't re-run for every keystroke |
| 7.1.4 | Incremental update (optional) | 💡 | dendrite-j40b | Only re-parse changed files |
| 7.1.5 | Update TUI in watch mode | ⬜ | dendrite-pspv | Live refresh |
| 7.1.6 | Print timestamp on each update | ⬜ | dendrite-qjx5 | Confirm it's working |

### Milestone 7.2: Query Commands

| ID | Task | Status | Bead | Notes |
|----|------|--------|------|-------|
| 7.2.1 | Implement `dendrite deps <file>` | ⬜ | dendrite-bv8q | List imports |
| 7.2.2 | Implement `dendrite rdeps <file>` | ⬜ | dendrite-od2c | List dependents |
| 7.2.3 | Implement `dendrite path <from> <to>` | ⬜ | dendrite-wkqf | Find connection path |
| 7.2.4 | Implement `dendrite list --layer X` | ⬜ | dendrite-e4nv | Files in layer |
| 7.2.5 | Implement `dendrite cycles` | ⬜ | dendrite-hudf | List all cycles |
| 7.2.6 | Implement `dendrite stats` | ⬜ | dendrite-7jwv | Quick metrics summary |
| 7.2.7 | Support JSON output for queries | ⬜ | dendrite-wlto | `--json` on all queries |

### Milestone 7.3: Editor Integration

| ID | Task | Status | Bead | Notes |
|----|------|--------|------|-------|
| 7.3.1 | Output errors in file:line:col format | ⬜ | dendrite-hamf | For editor jump-to |
| 7.3.2 | Support `--format=gcc` | ⬜ | dendrite-kfno | Standard error format |
| 7.3.3 | Support `--format=json` | ⬜ | dendrite-r0yb | For IDE plugins |
| 7.3.4 | Document VS Code integration | 💡 | dendrite-xdi7 | Problem matcher config |
| 7.3.5 | Document Neovim integration | 💡 | dendrite-vt10 | Quickfix list |

**Phase 7 Exit Criteria:**
- Watch mode works reliably
- Query commands provide quick answers
- Errors parseable by editors

---

## Phase 8: Distribution

**Goal:** Easy installation across all platforms.

### Milestone 8.1: Cross-Compilation

| ID | Task | Status | Bead | Notes |
|----|------|--------|------|-------|
| 8.1.1 | Set up GitHub Actions workflow | ⬜ | dendrite-912v | .github/workflows/release.yml |
| 8.1.2 | Build for x86_64-unknown-linux-gnu | ⬜ | dendrite-y9ua | Linux x64 |
| 8.1.3 | Build for aarch64-unknown-linux-gnu | ⬜ | dendrite-x3jg | Linux ARM64 |
| 8.1.4 | Build for x86_64-apple-darwin | ⬜ | dendrite-di89 | macOS x64 |
| 8.1.5 | Build for aarch64-apple-darwin | ⬜ | dendrite-tccj | macOS ARM64 |
| 8.1.6 | Build for x86_64-pc-windows-msvc | ⬜ | dendrite-kydn | Windows x64 |
| 8.1.7 | Strip binaries for size | ⬜ | dendrite-sy1r | strip command |
| 8.1.8 | Compress with UPX (optional) | 💡 | dendrite-xlw0 | Smaller downloads |

### Milestone 8.2: Release Automation

| ID | Task | Status | Bead | Notes |
|----|------|--------|------|-------|
| 8.2.1 | Trigger build on git tag push | ⬜ | dendrite-nsth | v* pattern |
| 8.2.2 | Generate checksums for binaries | ⬜ | dendrite-lwm5 | SHA256 sums |
| 8.2.3 | Create GitHub release automatically | ⬜ | dendrite-88ad | With changelog |
| 8.2.4 | Upload all binaries to release | ⬜ | dendrite-eq78 | Artifacts |
| 8.2.5 | Generate changelog from commits | 💡 | dendrite-3sg2 | Conventional commits |

### Milestone 8.3: Installation Methods

| ID | Task | Status | Bead | Notes |
|----|------|--------|------|-------|
| 8.3.1 | Document curl install script | ⬜ | dendrite-u8ez | One-liner in README |
| 8.3.2 | Publish to crates.io | ⬜ | dendrite-1vce | `cargo install dendrite` |
| 8.3.3 | Create Homebrew formula | 💡 | dendrite-pv06 | macOS package manager |
| 8.3.4 | Create AUR package | 💡 | dendrite-r3ff | Arch Linux |
| 8.3.5 | Create Nix flake | 💡 | dendrite-1mqn | NixOS |

**Phase 8 Exit Criteria:**
- Single command installation on all platforms
- Automated releases on git tag
- Binary size < 10MB

---

## Phase 9: Stretch Goals

### Milestone 9.1: Iced GUI

| ID | Task | Status | Bead | Notes |
|----|------|--------|------|-------|
| 9.1.1 | Create `gui/` module with Iced | 💡 | dendrite-e50n | GPU-accelerated UI |
| 9.1.2 | Implement graph canvas with pan/zoom | 💡 | dendrite-8xuy | Smooth navigation |
| 9.1.3 | Implement node rendering with icons | 💡 | dendrite-i4zo | File type icons |
| 9.1.4 | Implement edge rendering with curves | 💡 | dendrite-l8q8 | Bezier curves |
| 9.1.5 | Implement sidebar with file details | 💡 | dendrite-fz8i | Same data as TUI |
| 9.1.6 | Implement search with dropdown | 💡 | dendrite-vk94 | Fuzzy matching |
| 9.1.7 | Add dark/light theme toggle | 💡 | dendrite-g12k | User preference |
| 9.1.8 | Export graph as SVG | 💡 | dendrite-tzi1 | For documentation |

### Milestone 9.2: sqlite-vec Integration

| ID | Task | Status | Bead | Notes |
|----|------|--------|------|-------|
| 9.2.1 | Add sqlite and sqlite-vec dependencies | 💡 | dendrite-f47e | Vector search |
| 9.2.2 | Create embeddings schema | 💡 | dendrite-b5di | file_id, embedding, metadata |
| 9.2.3 | Integrate with OpenRouter API | 💡 | dendrite-167c | text-embedding-3-small |
| 9.2.4 | Generate embeddings for file summaries | 💡 | dendrite-l9it | Async batch processing |
| 9.2.5 | Implement semantic search query | 💡 | dendrite-fhhe | "find files about interrupts" |
| 9.2.6 | Cache embeddings, update on change | 💡 | dendrite-7h3d | Incremental updates |
| 9.2.7 | Add `dendrite search "query"` command | 💡 | dendrite-3611 | Natural language queries |

### Milestone 9.3: LSP Integration

| ID | Task | Status | Bead | Notes |
|----|------|--------|------|-------|
| 9.3.1 | Implement LSP server skeleton | 💡 | dendrite-5r2k | tower-lsp crate |
| 9.3.2 | Provide diagnostics for cycles | 💡 | dendrite-1o4z | Inline warnings |
| 9.3.3 | Provide code lens for import counts | 💡 | dendrite-npjp | "5 files import this" |
| 9.3.4 | Provide hover info for imports | 💡 | dendrite-2t4f | Show target file summary |
| 9.3.5 | Document VS Code extension | 💡 | dendrite-cte9 | User setup guide |

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
| 0.1.0 | 1 | Basic parsing and JSON output |  |
| 0.2.0 | 1-2 | Analysis engine, CI mode |  |
| 0.3.0 | 1-3 | Markdown generation |  |
| 0.4.0 | 1-4 | Configuration system |  |
| 0.5.0 | 1-5 | Full TUI |  |
| 0.6.0 | 1-6 | Assembly support |  |
| 0.7.0 | 1-7 | Watch mode, queries |  |
| 1.0.0 | 1-8 | Full release, all platforms |  |
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
