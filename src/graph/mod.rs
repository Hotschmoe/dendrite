//! Dependency graph construction and analysis.

pub mod analysis;

use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Architectural layer classification for files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Layer {
    /// Entry point (main.zig)
    Entry,
    /// Application layer (shell/*, init.zig)
    App,
    /// Core kernel functionality (kernel/*, memory/*)
    Core,
    /// Platform abstraction (platform/*, exceptions.zig)
    Platform,
    /// Device drivers (drivers/*)
    Driver,
    /// Architecture-specific code (arch/*)
    Arch,
    /// Unclassified files
    Unknown,
}

impl std::fmt::Display for Layer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Layer::Entry => "Entry",
            Layer::App => "App",
            Layer::Core => "Core",
            Layer::Platform => "Platform",
            Layer::Driver => "Driver",
            Layer::Arch => "Arch",
            Layer::Unknown => "Unknown",
        };
        write!(f, "{}", name)
    }
}

impl Layer {
    /// Classify a file based on its relative path.
    ///
    /// Uses pattern matching against common directory structures:
    /// - main.zig -> Entry
    /// - shell/*, init.zig -> App
    /// - kernel/*, memory/* -> Core
    /// - platform/*, exceptions.zig -> Platform
    /// - drivers/* -> Driver
    /// - arch/* -> Arch
    pub fn from_path(relative_path: &str) -> Self {
        let normalized = relative_path.replace('\\', "/");

        if normalized == "main.zig" {
            Layer::Entry
        } else if normalized.starts_with("shell/") || normalized == "init.zig" {
            Layer::App
        } else if normalized.starts_with("kernel/") || normalized.starts_with("memory/") {
            Layer::Core
        } else if normalized.starts_with("platform/") || normalized == "exceptions.zig" {
            Layer::Platform
        } else if normalized.starts_with("drivers/") {
            Layer::Driver
        } else if normalized.starts_with("arch/") {
            Layer::Arch
        } else {
            Layer::Unknown
        }
    }
}

/// A node in the dependency graph representing a source file.
#[derive(Debug, Clone)]
pub struct FileNode {
    /// Absolute path to the file
    pub path: PathBuf,
    /// Path relative to project root
    pub relative_path: String,
    /// Architectural layer this file belongs to
    pub layer: Layer,
    /// Distance from entry point
    pub depth: usize,
    /// Optional summary of the file's purpose
    pub summary: Option<String>,
    /// Public exports from this file
    pub exports: Vec<String>,
    /// Lines of code
    pub loc: usize,
}

/// Edge metadata representing an import relationship.
#[derive(Debug, Clone)]
pub struct Import {
    /// Line number where the import occurs
    pub line: usize,
}

/// Dependency graph using petgraph DiGraph.
pub type DepGraph = DiGraph<FileNode, Import>;

/// Builder for constructing dependency graphs from parsed files.
///
/// Handles:
/// - Adding file nodes with layer classification
/// - Resolving import paths (relative and absolute)
/// - Creating edges between nodes
/// - Handling missing targets with warnings
/// - Filtering standard library imports (optional)
pub struct GraphBuilder {
    graph: DepGraph,
    path_to_node: HashMap<PathBuf, NodeIndex>,
    project_root: PathBuf,
    include_std: bool,
    unresolved_imports: Vec<UnresolvedImport>,
}

/// An import that could not be resolved to a file node.
#[derive(Debug, Clone)]
pub struct UnresolvedImport {
    pub source_file: String,
    pub target: String,
    pub line: usize,
}

impl GraphBuilder {
    /// Create a new GraphBuilder.
    ///
    /// # Arguments
    /// * `project_root` - Root directory of the project (used for relative path resolution)
    /// * `include_std` - If true, include edges for standard library imports
    pub fn new(project_root: PathBuf, include_std: bool) -> Self {
        Self {
            graph: DiGraph::new(),
            path_to_node: HashMap::new(),
            project_root,
            include_std,
            unresolved_imports: Vec::new(),
        }
    }

    /// Add a file node to the graph.
    ///
    /// The layer is automatically determined from the relative path.
    /// Returns the NodeIndex for the added node.
    pub fn add_file(&mut self, node: FileNode) -> NodeIndex {
        let path = node.path.clone();
        let idx = self.graph.add_node(node);
        self.path_to_node.insert(path, idx);
        idx
    }

    /// Add edges for all imports in the graph.
    ///
    /// For each node, attempts to resolve its imports and create edges.
    /// Logs warnings for unresolved imports but does not create edges for them.
    pub fn add_edges(&mut self, import_map: &HashMap<PathBuf, Vec<(String, usize)>>) {
        for (source_path, imports) in import_map {
            let source_idx = match self.path_to_node.get(source_path) {
                Some(&idx) => idx,
                None => continue,
            };

            for (import_target, line) in imports {
                // Skip standard library imports if not included
                if !self.include_std && is_std_import(import_target) {
                    continue;
                }

                // Try to resolve the import target to an absolute path
                if let Some(target_path) = self.resolve_import(source_path, import_target) {
                    if let Some(&target_idx) = self.path_to_node.get(&target_path) {
                        // Create edge from source to target
                        self.graph.add_edge(source_idx, target_idx, Import { line: *line });
                    } else {
                        // Target file not in graph
                        self.unresolved_imports.push(UnresolvedImport {
                            source_file: source_path.display().to_string(),
                            target: import_target.clone(),
                            line: *line,
                        });
                    }
                } else {
                    // Could not resolve import path
                    self.unresolved_imports.push(UnresolvedImport {
                        source_file: source_path.display().to_string(),
                        target: import_target.clone(),
                        line: *line,
                    });
                }
            }
        }
    }

    /// Resolve an import path relative to the source file.
    ///
    /// Handles:
    /// - Relative imports: @import("../foo.zig"), @import("./bar.zig")
    /// - Same-directory imports: @import("foo.zig")
    /// - Absolute imports from project root: @import("kernel/memory.zig")
    ///
    /// Returns None for standard library imports or unresolvable paths.
    fn resolve_import(&self, source_path: &Path, import_target: &str) -> Option<PathBuf> {
        // Skip standard library imports
        if is_std_import(import_target) {
            return None;
        }

        // Try relative to source file's directory
        if let Some(source_dir) = source_path.parent() {
            let candidate = source_dir.join(import_target);

            // First check if this path is already in our graph (for testing)
            if self.path_to_node.contains_key(&candidate) {
                return Some(candidate);
            }

            // Then check if it exists on disk (for real usage)
            if candidate.exists() {
                return candidate.canonicalize().ok();
            }
        }

        // Try relative to project root
        let candidate = self.project_root.join(import_target);

        // First check if this path is already in our graph (for testing)
        if self.path_to_node.contains_key(&candidate) {
            return Some(candidate);
        }

        // Then check if it exists on disk (for real usage)
        if candidate.exists() {
            return candidate.canonicalize().ok();
        }

        None
    }

    /// Consume the builder and return the constructed graph.
    ///
    /// Also prints warnings for any unresolved imports.
    pub fn build(self) -> DepGraph {
        if !self.unresolved_imports.is_empty() {
            eprintln!("Warning: {} unresolved imports", self.unresolved_imports.len());
            for unresolved in &self.unresolved_imports {
                eprintln!(
                    "  {} imports {} at line {} (file not found)",
                    unresolved.source_file, unresolved.target, unresolved.line
                );
            }
        }
        self.graph
    }

    /// Get the list of unresolved imports without consuming the builder.
    pub fn unresolved_imports(&self) -> &[UnresolvedImport] {
        &self.unresolved_imports
    }
}

/// Check if an import target is a standard library import.
///
/// Standard library imports include:
/// - "std"
/// - "builtin"
/// - Anything starting with "std."
fn is_std_import(target: &str) -> bool {
    target == "std" || target == "builtin" || target.starts_with("std.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_from_path_entry() {
        assert_eq!(Layer::from_path("main.zig"), Layer::Entry);
    }

    #[test]
    fn test_layer_from_path_app() {
        assert_eq!(Layer::from_path("init.zig"), Layer::App);
        assert_eq!(Layer::from_path("shell/main.zig"), Layer::App);
        assert_eq!(Layer::from_path("shell/commands.zig"), Layer::App);
    }

    #[test]
    fn test_layer_from_path_core() {
        assert_eq!(Layer::from_path("kernel/scheduler.zig"), Layer::Core);
        assert_eq!(Layer::from_path("memory/allocator.zig"), Layer::Core);
    }

    #[test]
    fn test_layer_from_path_platform() {
        assert_eq!(Layer::from_path("platform/x86.zig"), Layer::Platform);
        assert_eq!(Layer::from_path("exceptions.zig"), Layer::Platform);
    }

    #[test]
    fn test_layer_from_path_driver() {
        assert_eq!(Layer::from_path("drivers/uart.zig"), Layer::Driver);
    }

    #[test]
    fn test_layer_from_path_arch() {
        assert_eq!(Layer::from_path("arch/x86_64.zig"), Layer::Arch);
    }

    #[test]
    fn test_layer_from_path_unknown() {
        assert_eq!(Layer::from_path("utils.zig"), Layer::Unknown);
        assert_eq!(Layer::from_path("lib/helper.zig"), Layer::Unknown);
    }

    #[test]
    fn test_layer_from_path_windows_separator() {
        assert_eq!(Layer::from_path("kernel\\scheduler.zig"), Layer::Core);
        assert_eq!(Layer::from_path("shell\\commands.zig"), Layer::App);
    }

    #[test]
    fn test_is_std_import() {
        assert!(is_std_import("std"));
        assert!(is_std_import("builtin"));
        assert!(is_std_import("std.debug"));
        assert!(!is_std_import("stdlib.zig"));
        assert!(!is_std_import("my_std.zig"));
        assert!(!is_std_import("kernel/std.zig"));
    }

    #[test]
    fn test_graph_builder_new() {
        let builder = GraphBuilder::new(PathBuf::from("/project"), true);
        assert_eq!(builder.project_root, PathBuf::from("/project"));
        assert!(builder.include_std);
        assert_eq!(builder.graph.node_count(), 0);
        assert_eq!(builder.graph.edge_count(), 0);
    }

    #[test]
    fn test_graph_builder_add_file() {
        let mut builder = GraphBuilder::new(PathBuf::from("/project"), false);

        let node = FileNode {
            path: PathBuf::from("/project/main.zig"),
            relative_path: "main.zig".to_string(),
            layer: Layer::Entry,
            depth: 0,
            summary: None,
            exports: vec![],
            loc: 10,
        };

        let idx = builder.add_file(node);

        assert_eq!(builder.graph.node_count(), 1);
        assert_eq!(builder.graph[idx].relative_path, "main.zig");
        assert_eq!(builder.graph[idx].layer, Layer::Entry);
    }

    #[test]
    fn test_graph_builder_linear_chain() {
        let mut builder = GraphBuilder::new(PathBuf::from("/project"), false);

        // Create a linear chain: A -> B -> C
        let node_a = FileNode {
            path: PathBuf::from("/project/a.zig"),
            relative_path: "a.zig".to_string(),
            layer: Layer::Unknown,
            depth: 0,
            summary: None,
            exports: vec![],
            loc: 10,
        };

        let node_b = FileNode {
            path: PathBuf::from("/project/b.zig"),
            relative_path: "b.zig".to_string(),
            layer: Layer::Unknown,
            depth: 0,
            summary: None,
            exports: vec![],
            loc: 10,
        };

        let node_c = FileNode {
            path: PathBuf::from("/project/c.zig"),
            relative_path: "c.zig".to_string(),
            layer: Layer::Unknown,
            depth: 0,
            summary: None,
            exports: vec![],
            loc: 10,
        };

        builder.add_file(node_a);
        builder.add_file(node_b);
        builder.add_file(node_c);

        let mut import_map = HashMap::new();
        import_map.insert(
            PathBuf::from("/project/a.zig"),
            vec![("b.zig".to_string(), 1)],
        );
        import_map.insert(
            PathBuf::from("/project/b.zig"),
            vec![("c.zig".to_string(), 1)],
        );

        builder.add_edges(&import_map);
        let graph = builder.build();

        assert_eq!(graph.node_count(), 3);
        assert_eq!(graph.edge_count(), 2);
    }

    #[test]
    fn test_graph_builder_diamond_pattern() {
        let mut builder = GraphBuilder::new(PathBuf::from("/project"), false);

        // Create a diamond: A -> B, A -> C, B -> D, C -> D
        let node_a = FileNode {
            path: PathBuf::from("/project/a.zig"),
            relative_path: "a.zig".to_string(),
            layer: Layer::Unknown,
            depth: 0,
            summary: None,
            exports: vec![],
            loc: 10,
        };

        let node_b = FileNode {
            path: PathBuf::from("/project/b.zig"),
            relative_path: "b.zig".to_string(),
            layer: Layer::Unknown,
            depth: 0,
            summary: None,
            exports: vec![],
            loc: 10,
        };

        let node_c = FileNode {
            path: PathBuf::from("/project/c.zig"),
            relative_path: "c.zig".to_string(),
            layer: Layer::Unknown,
            depth: 0,
            summary: None,
            exports: vec![],
            loc: 10,
        };

        let node_d = FileNode {
            path: PathBuf::from("/project/d.zig"),
            relative_path: "d.zig".to_string(),
            layer: Layer::Unknown,
            depth: 0,
            summary: None,
            exports: vec![],
            loc: 10,
        };

        builder.add_file(node_a);
        builder.add_file(node_b);
        builder.add_file(node_c);
        builder.add_file(node_d);

        let mut import_map = HashMap::new();
        import_map.insert(
            PathBuf::from("/project/a.zig"),
            vec![("b.zig".to_string(), 1), ("c.zig".to_string(), 2)],
        );
        import_map.insert(
            PathBuf::from("/project/b.zig"),
            vec![("d.zig".to_string(), 1)],
        );
        import_map.insert(
            PathBuf::from("/project/c.zig"),
            vec![("d.zig".to_string(), 1)],
        );

        builder.add_edges(&import_map);
        let graph = builder.build();

        assert_eq!(graph.node_count(), 4);
        assert_eq!(graph.edge_count(), 4);
    }

    #[test]
    fn test_graph_builder_disconnected_components() {
        let mut builder = GraphBuilder::new(PathBuf::from("/project"), false);

        // Create two disconnected components: A -> B and C -> D
        let node_a = FileNode {
            path: PathBuf::from("/project/a.zig"),
            relative_path: "a.zig".to_string(),
            layer: Layer::Unknown,
            depth: 0,
            summary: None,
            exports: vec![],
            loc: 10,
        };

        let node_b = FileNode {
            path: PathBuf::from("/project/b.zig"),
            relative_path: "b.zig".to_string(),
            layer: Layer::Unknown,
            depth: 0,
            summary: None,
            exports: vec![],
            loc: 10,
        };

        let node_c = FileNode {
            path: PathBuf::from("/project/c.zig"),
            relative_path: "c.zig".to_string(),
            layer: Layer::Unknown,
            depth: 0,
            summary: None,
            exports: vec![],
            loc: 10,
        };

        let node_d = FileNode {
            path: PathBuf::from("/project/d.zig"),
            relative_path: "d.zig".to_string(),
            layer: Layer::Unknown,
            depth: 0,
            summary: None,
            exports: vec![],
            loc: 10,
        };

        builder.add_file(node_a);
        builder.add_file(node_b);
        builder.add_file(node_c);
        builder.add_file(node_d);

        let mut import_map = HashMap::new();
        import_map.insert(
            PathBuf::from("/project/a.zig"),
            vec![("b.zig".to_string(), 1)],
        );
        import_map.insert(
            PathBuf::from("/project/c.zig"),
            vec![("d.zig".to_string(), 1)],
        );

        builder.add_edges(&import_map);
        let graph = builder.build();

        assert_eq!(graph.node_count(), 4);
        assert_eq!(graph.edge_count(), 2);
    }

    #[test]
    fn test_graph_builder_unresolved_imports() {
        let mut builder = GraphBuilder::new(PathBuf::from("/project"), false);

        let node_a = FileNode {
            path: PathBuf::from("/project/a.zig"),
            relative_path: "a.zig".to_string(),
            layer: Layer::Unknown,
            depth: 0,
            summary: None,
            exports: vec![],
            loc: 10,
        };

        builder.add_file(node_a);

        let mut import_map = HashMap::new();
        import_map.insert(
            PathBuf::from("/project/a.zig"),
            vec![("missing.zig".to_string(), 1)],
        );

        builder.add_edges(&import_map);

        assert_eq!(builder.unresolved_imports().len(), 1);
        assert_eq!(builder.unresolved_imports()[0].target, "missing.zig");
        assert_eq!(builder.unresolved_imports()[0].line, 1);
    }

    #[test]
    fn test_graph_builder_exclude_std() {
        let mut builder = GraphBuilder::new(PathBuf::from("/project"), false);

        let node_a = FileNode {
            path: PathBuf::from("/project/a.zig"),
            relative_path: "a.zig".to_string(),
            layer: Layer::Unknown,
            depth: 0,
            summary: None,
            exports: vec![],
            loc: 10,
        };

        builder.add_file(node_a);

        let mut import_map = HashMap::new();
        import_map.insert(
            PathBuf::from("/project/a.zig"),
            vec![("std".to_string(), 1), ("builtin".to_string(), 2)],
        );

        builder.add_edges(&import_map);
        let graph = builder.build();

        assert_eq!(graph.node_count(), 1);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_graph_builder_include_std() {
        let mut builder = GraphBuilder::new(PathBuf::from("/project"), true);

        let node_a = FileNode {
            path: PathBuf::from("/project/a.zig"),
            relative_path: "a.zig".to_string(),
            layer: Layer::Unknown,
            depth: 0,
            summary: None,
            exports: vec![],
            loc: 10,
        };

        builder.add_file(node_a);

        let mut import_map = HashMap::new();
        import_map.insert(
            PathBuf::from("/project/a.zig"),
            vec![("std".to_string(), 1)],
        );

        builder.add_edges(&import_map);

        // std is not in the graph, so it will be unresolved
        assert_eq!(builder.unresolved_imports().len(), 1);
    }
}
