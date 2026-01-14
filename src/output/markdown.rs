//! Markdown generation for CODEBASE.md and file summaries.
//!
//! Generates human and AI-readable documentation from dependency graphs:
//! - CODEBASE.md: Overview with Mermaid diagrams, file index, alerts
//! - File summaries: Per-file metadata, imports, dependents
//!
//! Uses Handlebars templates for flexible output formatting.

use crate::graph::analysis::AnalysisResult;
use crate::graph::{DepGraph, Layer};
use crate::output::json::CodebaseMap;
use handlebars::Handlebars;
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use petgraph::Direction;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

const MAX_MERMAID_NODES: usize = 30;

/// Template data for CODEBASE.md generation.
#[derive(Debug, Serialize)]
pub struct CodebaseTemplate {
    /// Project name (derived from root path)
    pub project_name: String,
    /// Generation timestamp
    pub generated_at: String,
    /// Overview statistics
    pub overview: Overview,
    /// Mermaid diagram source
    pub mermaid_diagram: String,
    /// File index entries
    pub files: Vec<FileEntry>,
    /// Alert entries (cycles and violations)
    pub alerts: Vec<Alert>,
    /// Layer definitions with descriptions
    pub layers: Vec<LayerInfo>,
    /// Whether there are any alerts
    pub has_alerts: bool,
}

/// Overview statistics for the codebase.
#[derive(Debug, Serialize)]
pub struct Overview {
    /// Total file count
    pub file_count: usize,
    /// Total lines of code
    pub total_loc: usize,
    /// Maximum depth from entry points
    pub max_depth: usize,
    /// Number of entry points
    pub entry_point_count: usize,
    /// Entry point file names
    pub entry_points: Vec<String>,
    /// Number of cycles detected
    pub cycle_count: usize,
    /// Number of layer violations
    pub violation_count: usize,
    /// Files per layer
    pub layer_counts: Vec<LayerCount>,
}

/// Count of files in a layer.
#[derive(Debug, Serialize)]
pub struct LayerCount {
    pub layer: String,
    pub count: usize,
}

/// File entry for the index table.
#[derive(Debug, Serialize)]
pub struct FileEntry {
    pub path: String,
    pub layer: String,
    pub depth: usize,
    pub loc: usize,
    pub imports: usize,
    pub imported_by: usize,
    pub in_cycle: bool,
}

/// Alert for cycles or violations.
#[derive(Debug, Clone, Serialize)]
pub struct Alert {
    /// Alert severity: "error" or "warning"
    pub severity: String,
    /// Alert type: "cycle" or "violation"
    pub alert_type: String,
    /// Human-readable message
    pub message: String,
    /// Affected files with line numbers
    pub details: Vec<String>,
    /// Fix suggestion
    pub fix: String,
}

/// Layer definition with description.
#[derive(Debug, Serialize)]
pub struct LayerInfo {
    pub name: String,
    pub description: String,
    pub precedence: u8,
}

impl CodebaseTemplate {
    /// Create template data from graph and analysis results.
    pub fn from_graph(
        graph: &DepGraph,
        analysis: &AnalysisResult,
        codemap: &CodebaseMap,
    ) -> Self {
        let project_name = extract_project_name(&codemap.root_path);
        let overview = build_overview(graph, analysis, codemap);
        let mermaid_diagram = generate_mermaid(graph, analysis, MAX_MERMAID_NODES);
        let files = build_file_entries(graph, analysis);
        let alerts = build_alerts(analysis);
        let layers = build_layer_info();
        let has_alerts = !alerts.is_empty();

        Self {
            project_name,
            generated_at: codemap.generated_at.clone(),
            overview,
            mermaid_diagram,
            files,
            alerts,
            layers,
            has_alerts,
        }
    }
}

/// Extract project name from root path.
fn extract_project_name(root_path: &str) -> String {
    Path::new(root_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project")
        .to_string()
}

/// Build overview statistics.
fn build_overview(
    graph: &DepGraph,
    analysis: &AnalysisResult,
    codemap: &CodebaseMap,
) -> Overview {
    let total_loc: usize = graph.node_indices().map(|idx| graph[idx].loc).sum();

    let mut layer_map: HashMap<String, usize> = HashMap::new();
    for idx in graph.node_indices() {
        let layer = graph[idx].layer.to_string();
        *layer_map.entry(layer).or_insert(0) += 1;
    }

    let mut layer_counts: Vec<LayerCount> = layer_map
        .into_iter()
        .map(|(layer, count)| LayerCount { layer, count })
        .collect();
    layer_counts.sort_by(|a, b| a.layer.cmp(&b.layer));

    Overview {
        file_count: codemap.metadata.node_count,
        total_loc,
        max_depth: analysis.max_depth,
        entry_point_count: analysis.entry_points.len(),
        entry_points: analysis.entry_points.clone(),
        cycle_count: analysis.cycles.len(),
        violation_count: analysis.violations.len(),
        layer_counts,
    }
}

/// Build file entries for the index table.
fn build_file_entries(graph: &DepGraph, analysis: &AnalysisResult) -> Vec<FileEntry> {
    let cycle_files: HashSet<&str> = analysis
        .cycles
        .iter()
        .flat_map(|c| c.nodes.iter().map(|s| s.as_str()))
        .collect();

    let mut entries: Vec<FileEntry> = graph
        .node_indices()
        .map(|idx| {
            let node = &graph[idx];
            let imports = graph.edges_directed(idx, Direction::Outgoing).count();
            let imported_by = graph.edges_directed(idx, Direction::Incoming).count();
            let in_cycle = cycle_files.contains(node.relative_path.as_str());

            FileEntry {
                path: node.relative_path.clone(),
                layer: node.layer.to_string(),
                depth: node.depth,
                loc: node.loc,
                imports,
                imported_by,
                in_cycle,
            }
        })
        .collect();

    entries.sort_by(|a, b| a.path.cmp(&b.path));
    entries
}

/// Build alerts from analysis results.
fn build_alerts(analysis: &AnalysisResult) -> Vec<Alert> {
    let mut alerts = Vec::new();

    for cycle in &analysis.cycles {
        let details: Vec<String> = cycle
            .edges
            .iter()
            .map(|(from, to, line)| format!("{}:{} imports {}", from, line, to))
            .collect();

        let fix = if !cycle.edges.is_empty() {
            let (from, to, line) = &cycle.edges[0];
            format!(
                "Break cycle by removing import at {}:{} (imports {})",
                from, line, to
            )
        } else {
            "Extract shared types to a common module".to_string()
        };

        alerts.push(Alert {
            severity: "error".to_string(),
            alert_type: "cycle".to_string(),
            message: format!("Circular dependency: {}", cycle.nodes.join(" -> ")),
            details,
            fix,
        });
    }

    for violation in &analysis.violations {
        alerts.push(Alert {
            severity: "warning".to_string(),
            alert_type: "violation".to_string(),
            message: format!(
                "Layer violation: {} ({}) imports {} ({})",
                violation.file, violation.from_layer, violation.imports, violation.to_layer
            ),
            details: vec![format!("{}:{}", violation.file, violation.line)],
            fix: violation.fix_suggestion(),
        });
    }

    alerts
}

const LAYER_DEFINITIONS: &[(&str, &str, u8)] = &[
    ("Entry", "Application entry point (main.zig)", 0),
    ("App", "Application layer (shell/*, init.zig)", 1),
    ("Core", "Core functionality (kernel/*, memory/*)", 2),
    ("Platform", "Platform abstraction (platform/*, exceptions.zig)", 3),
    ("Driver", "Device drivers (drivers/*)", 4),
    ("Arch", "Architecture-specific code (arch/*)", 5),
];

/// Build layer definitions.
fn build_layer_info() -> Vec<LayerInfo> {
    LAYER_DEFINITIONS
        .iter()
        .map(|(name, description, precedence)| LayerInfo {
            name: (*name).to_string(),
            description: (*description).to_string(),
            precedence: *precedence,
        })
        .collect()
}

/// Generate Mermaid flowchart diagram from dependency graph.
///
/// Limits output to top N most important nodes (highest fan-in + fan-out).
/// Highlights cycle nodes with red styling.
pub fn generate_mermaid(graph: &DepGraph, analysis: &AnalysisResult, max_nodes: usize) -> String {
    if graph.node_count() == 0 {
        return "```mermaid\nflowchart LR\n  empty[No files found]\n```".to_string();
    }

    let cycle_files: HashSet<&str> = analysis
        .cycles
        .iter()
        .flat_map(|c| c.nodes.iter().map(|s| s.as_str()))
        .collect();

    // Score nodes by importance (fan-in + fan-out)
    let mut node_scores: Vec<(NodeIndex, usize)> = graph
        .node_indices()
        .map(|idx| {
            let fan_in = graph.edges_directed(idx, Direction::Incoming).count();
            let fan_out = graph.edges_directed(idx, Direction::Outgoing).count();
            (idx, fan_in + fan_out)
        })
        .collect();

    node_scores.sort_by(|a, b| b.1.cmp(&a.1));

    // Select top N nodes, but always include cycle nodes
    let mut selected: HashSet<NodeIndex> = HashSet::new();

    // First, add all cycle nodes
    for idx in graph.node_indices() {
        if cycle_files.contains(graph[idx].relative_path.as_str()) {
            selected.insert(idx);
        }
    }

    // Then add top nodes by importance up to max
    for (idx, _) in node_scores {
        if selected.len() >= max_nodes {
            break;
        }
        selected.insert(idx);
    }

    let mut lines = vec!["```mermaid".to_string(), "flowchart LR".to_string()];

    // Create node ID mapping
    let node_ids: HashMap<NodeIndex, String> = selected
        .iter()
        .map(|&idx| {
            let safe_id = sanitize_mermaid_id(&graph[idx].relative_path);
            (idx, safe_id)
        })
        .collect();

    // Add style classes
    lines.push("  classDef entry fill:#90EE90,stroke:#006400".to_string());
    lines.push("  classDef app fill:#87CEEB,stroke:#4169E1".to_string());
    lines.push("  classDef core fill:#FFD700,stroke:#B8860B".to_string());
    lines.push("  classDef platform fill:#DDA0DD,stroke:#8B008B".to_string());
    lines.push("  classDef driver fill:#FFA07A,stroke:#FF4500".to_string());
    lines.push("  classDef arch fill:#D3D3D3,stroke:#696969".to_string());
    lines.push("  classDef cycle fill:#FF6B6B,stroke:#CC0000,stroke-width:3px".to_string());

    // Add nodes
    for &idx in &selected {
        let node = &graph[idx];
        let id = &node_ids[&idx];
        let label = truncate_filename(&node.relative_path, 20);
        let in_cycle = cycle_files.contains(node.relative_path.as_str());

        let class = if in_cycle {
            "cycle"
        } else {
            layer_to_class(node.layer)
        };

        lines.push(format!("  {}[{}]:::{}", id, label, class));
    }

    // Add edges between selected nodes
    for edge in graph.raw_edges() {
        let source = edge.source();
        let target = edge.target();

        if selected.contains(&source) && selected.contains(&target) {
            let source_id = &node_ids[&source];
            let target_id = &node_ids[&target];
            lines.push(format!("  {} --> {}", source_id, target_id));
        }
    }

    lines.push("```".to_string());

    // Add truncation notice if needed
    if graph.node_count() > max_nodes {
        lines.push(String::new());
        lines.push(format!(
            "_Showing {} of {} files (top by connectivity)_",
            selected.len(),
            graph.node_count()
        ));
    }

    lines.join("\n")
}

/// Sanitize a file path for use as a Mermaid node ID.
fn sanitize_mermaid_id(path: &str) -> String {
    path.replace(['/', '\\', '.', '-', ' '], "_")
}

/// Truncate filename for display, preserving extension.
fn truncate_filename(path: &str, max_len: usize) -> String {
    if path.len() <= max_len {
        return path.to_string();
    }

    let basename = Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(path);

    if basename.len() <= max_len {
        return basename.to_string();
    }

    let truncated = &basename[..max_len - 3];
    format!("{}...", truncated)
}

/// Map Layer to Mermaid style class.
fn layer_to_class(layer: Layer) -> &'static str {
    match layer {
        Layer::Entry => "entry",
        Layer::App => "app",
        Layer::Core => "core",
        Layer::Platform => "platform",
        Layer::Driver => "driver",
        Layer::Arch => "arch",
        Layer::Unknown => "arch",
    }
}

/// Default Handlebars template for CODEBASE.md.
pub const CODEBASE_TEMPLATE: &str = r#"# {{ project_name }} - Codebase Map

> Generated: {{ generated_at }}

## Overview

| Metric | Value |
|--------|-------|
| Files | {{ overview.file_count }} |
| Lines of Code | {{ overview.total_loc }} |
| Max Depth | {{ overview.max_depth }} |
| Entry Points | {{ overview.entry_point_count }} |
| Cycles | {{ overview.cycle_count }} |
| Violations | {{ overview.violation_count }} |

### Entry Points

{{#each overview.entry_points}}
- `{{ this }}`
{{/each}}

### Files by Layer

| Layer | Count |
|-------|-------|
{{#each overview.layer_counts}}
| {{ layer }} | {{ count }} |
{{/each}}

## Dependency Graph

{{{ mermaid_diagram }}}

## File Index

| File | Layer | Depth | LOC | Imports | Imported By | Cycle |
|------|-------|-------|-----|---------|-------------|-------|
{{#each files}}
| `{{ path }}` | {{ layer }} | {{ depth }} | {{ loc }} | {{ imports }} | {{ imported_by }} | {{#if in_cycle}}:warning:{{/if}} |
{{/each}}

{{#if has_alerts}}
## Alerts

{{#each alerts}}
### {{#if (eq severity "error")}}:x:{{else}}:warning:{{/if}} {{ alert_type }}: {{{ message }}}

**Details:**
{{#each details}}
- `{{ this }}`
{{/each}}

**Fix:** {{ fix }}

{{/each}}
{{/if}}

## Layer Architecture

The codebase follows a layered architecture. Each layer can only import from layers at the same level or below.

```
Entry (0) -> App (1) -> Core (2) -> Platform (3) -> Driver (4) -> Arch (5)
```

| Layer | Description | Precedence |
|-------|-------------|------------|
{{#each layers}}
| {{ name }} | {{ description }} | {{ precedence }} |
{{/each}}

---

_Generated by [dendrite](https://github.com/hotschmoe/dendrite)_
"#;

/// Render CODEBASE.md using the template and data.
pub fn render_codebase_md(template_data: &CodebaseTemplate) -> anyhow::Result<String> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(true);

    // Register equality helper
    handlebars.register_helper(
        "eq",
        Box::new(
            |h: &handlebars::Helper,
             _: &Handlebars,
             _: &handlebars::Context,
             _: &mut handlebars::RenderContext,
             out: &mut dyn handlebars::Output|
             -> handlebars::HelperResult {
                let param1 = h
                    .param(0)
                    .and_then(|v| v.value().as_str())
                    .unwrap_or("");
                let param2 = h
                    .param(1)
                    .and_then(|v| v.value().as_str())
                    .unwrap_or("");
                out.write(if param1 == param2 { "true" } else { "" })?;
                Ok(())
            },
        ),
    );

    handlebars.register_template_string("codebase", CODEBASE_TEMPLATE)?;

    let rendered = handlebars.render("codebase", template_data)?;
    Ok(rendered)
}

/// Write CODEBASE.md to the output directory.
pub fn write_codebase_md(output_dir: &Path, content: &str) -> anyhow::Result<()> {
    fs::create_dir_all(output_dir)?;
    let output_path = output_dir.join("CODEBASE.md");
    fs::write(&output_path, content)?;
    Ok(())
}

/// Generate and write CODEBASE.md from graph and analysis.
///
/// This is the main entry point for markdown generation.
pub fn generate_codebase_md(
    graph: &DepGraph,
    analysis: &AnalysisResult,
    codemap: &CodebaseMap,
    output_dir: &Path,
) -> anyhow::Result<()> {
    let template_data = CodebaseTemplate::from_graph(graph, analysis, codemap);
    let content = render_codebase_md(&template_data)?;
    write_codebase_md(output_dir, &content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{FileNode, GraphBuilder};
    use std::path::PathBuf;

    fn create_test_graph() -> (DepGraph, AnalysisResult, CodebaseMap) {
        let mut builder = GraphBuilder::new(PathBuf::from("/project"), false);

        let main_node = FileNode {
            path: PathBuf::from("/project/main.zig"),
            relative_path: "main.zig".to_string(),
            layer: Layer::Entry,
            depth: 0,
            summary: Some("Entry point".to_string()),
            exports: vec!["main".to_string()],
            loc: 50,
        };

        let lib_node = FileNode {
            path: PathBuf::from("/project/kernel/scheduler.zig"),
            relative_path: "kernel/scheduler.zig".to_string(),
            layer: Layer::Core,
            depth: 1,
            summary: None,
            exports: vec!["schedule".to_string()],
            loc: 200,
        };

        builder.add_file(main_node);
        builder.add_file(lib_node);

        let mut import_map = HashMap::new();
        import_map.insert(
            PathBuf::from("/project/main.zig"),
            vec![("kernel/scheduler.zig".to_string(), 5)],
        );

        builder.add_edges(&import_map);
        let graph = builder.build();

        let analysis = crate::graph::analysis::analyze(&graph, 3);
        let codemap = CodebaseMap::from_graph(&graph, "/project");

        (graph, analysis, codemap)
    }

    #[test]
    fn test_extract_project_name() {
        assert_eq!(extract_project_name("/home/user/myproject"), "myproject");
        assert_eq!(extract_project_name("C:\\Users\\dev\\myapp"), "myapp");
        assert_eq!(extract_project_name("/project"), "project");
    }

    #[test]
    fn test_sanitize_mermaid_id() {
        assert_eq!(sanitize_mermaid_id("main.zig"), "main_zig");
        assert_eq!(
            sanitize_mermaid_id("kernel/scheduler.zig"),
            "kernel_scheduler_zig"
        );
        assert_eq!(
            sanitize_mermaid_id("my-file.zig"),
            "my_file_zig"
        );
    }

    #[test]
    fn test_truncate_filename() {
        assert_eq!(truncate_filename("main.zig", 20), "main.zig");
        assert_eq!(
            truncate_filename("very_long_filename_here.zig", 15),
            "very_long_fi..."
        );
    }

    #[test]
    fn test_layer_to_class() {
        assert_eq!(layer_to_class(Layer::Entry), "entry");
        assert_eq!(layer_to_class(Layer::Core), "core");
        assert_eq!(layer_to_class(Layer::Unknown), "arch");
    }

    #[test]
    fn test_generate_mermaid_empty() {
        let graph = DepGraph::new();
        let analysis = crate::graph::analysis::analyze(&graph, 3);
        let mermaid = generate_mermaid(&graph, &analysis, 30);
        assert!(mermaid.contains("No files found"));
    }

    #[test]
    fn test_generate_mermaid_simple() {
        let (graph, analysis, _) = create_test_graph();
        let mermaid = generate_mermaid(&graph, &analysis, 30);

        assert!(mermaid.contains("```mermaid"));
        assert!(mermaid.contains("flowchart LR"));
        assert!(mermaid.contains("main_zig"));
        assert!(mermaid.contains("kernel_scheduler_zig"));
        assert!(mermaid.contains(":::entry"));
        assert!(mermaid.contains(":::core"));
    }

    #[test]
    fn test_build_overview() {
        let (graph, analysis, codemap) = create_test_graph();
        let overview = build_overview(&graph, &analysis, &codemap);

        assert_eq!(overview.file_count, 2);
        assert_eq!(overview.total_loc, 250);
        assert_eq!(overview.entry_point_count, 1);
        assert!(overview.entry_points.contains(&"main.zig".to_string()));
    }

    #[test]
    fn test_build_file_entries() {
        let (graph, analysis, _) = create_test_graph();
        let entries = build_file_entries(&graph, &analysis);

        assert_eq!(entries.len(), 2);

        let main_entry = entries.iter().find(|e| e.path == "main.zig").unwrap();
        assert_eq!(main_entry.layer, "Entry");
        assert_eq!(main_entry.imports, 1);
        assert_eq!(main_entry.imported_by, 0);
    }

    #[test]
    fn test_build_layer_info() {
        let layers = build_layer_info();
        assert_eq!(layers.len(), 6);
        assert_eq!(layers[0].name, "Entry");
        assert_eq!(layers[0].precedence, 0);
    }

    #[test]
    fn test_render_codebase_md() {
        let (graph, analysis, codemap) = create_test_graph();
        let template_data = CodebaseTemplate::from_graph(&graph, &analysis, &codemap);
        let result = render_codebase_md(&template_data);

        assert!(result.is_ok());
        let content = result.unwrap();
        assert!(content.contains("# project - Codebase Map"));
        assert!(content.contains("| Files | 2 |"));
        assert!(content.contains("```mermaid"));
        assert!(content.contains("main.zig"));
    }

    #[test]
    fn test_codebase_template_from_graph() {
        let (graph, analysis, codemap) = create_test_graph();
        let template = CodebaseTemplate::from_graph(&graph, &analysis, &codemap);

        assert_eq!(template.project_name, "project");
        assert_eq!(template.overview.file_count, 2);
        assert!(!template.mermaid_diagram.is_empty());
        assert_eq!(template.files.len(), 2);
        assert!(!template.has_alerts);
    }
}
