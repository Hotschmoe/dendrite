//! JSON serialization for codebase maps.
//!
//! Provides CodebaseMap structure and conversion from DepGraph to JSON format.
//! Output format is designed for:
//! - AI agent consumption (LLM-friendly structure)
//! - Graph visualization tools (nodes and edges)
//! - Programmatic analysis (well-defined schema)

use crate::graph::{DepGraph, FileNode, Import, Layer};
use petgraph::graph::NodeIndex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Top-level JSON structure for codebase dependency maps.
///
/// Represents a complete snapshot of a codebase's import graph with:
/// - All files as nodes with metadata
/// - All imports as edges with line numbers
/// - Summary metadata about the graph
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CodebaseMap {
    /// Schema version (currently "1.0")
    pub version: String,
    /// ISO 8601 timestamp when this map was generated
    pub generated_at: String,
    /// Absolute path to project root
    pub root_path: String,
    /// All file nodes in the dependency graph
    pub nodes: Vec<NodeInfo>,
    /// All import edges between nodes
    pub edges: Vec<EdgeInfo>,
    /// Summary statistics about the graph
    pub metadata: Metadata,
}

/// Node metadata for a single source file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NodeInfo {
    /// Unique node ID (petgraph NodeIndex)
    pub id: usize,
    /// Absolute path to the file
    pub path: String,
    /// Path relative to project root
    pub relative_path: String,
    /// Architectural layer classification
    pub layer: String,
    /// Distance from entry point (0 = entry)
    pub depth: usize,
    /// Optional summary of file's purpose
    pub summary: Option<String>,
    /// Public exports from this file
    pub exports: Vec<String>,
    /// Lines of code
    pub loc: usize,
}

/// Edge representing an import relationship.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EdgeInfo {
    /// Source node ID (file doing the importing)
    pub from: usize,
    /// Target node ID (file being imported)
    pub to: usize,
    /// Line number where the import occurs
    pub line: usize,
}

/// Summary metadata about the entire codebase graph.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Metadata {
    /// Total number of nodes in the graph
    pub node_count: usize,
    /// Total number of import edges
    pub edge_count: usize,
    /// Maximum depth from entry point
    pub max_depth: usize,
    /// List of all layers present in the graph
    pub layers: Vec<String>,
}

impl CodebaseMap {
    /// Convert a DepGraph into a CodebaseMap for JSON serialization.
    ///
    /// # Arguments
    /// * `graph` - The dependency graph to convert
    /// * `root_path` - Absolute path to the project root
    ///
    /// # Returns
    /// A CodebaseMap ready for JSON serialization
    pub fn from_graph(graph: &DepGraph, root_path: &str) -> Self {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut layer_set = std::collections::HashSet::new();
        let mut max_depth = 0;

        // Convert all nodes
        for node_idx in graph.node_indices() {
            let node = &graph[node_idx];
            let layer_str = layer_to_string(node.layer);

            layer_set.insert(layer_str.clone());
            max_depth = max_depth.max(node.depth);

            nodes.push(NodeInfo {
                id: node_idx.index(),
                path: node.path.display().to_string(),
                relative_path: node.relative_path.clone(),
                layer: layer_str,
                depth: node.depth,
                summary: node.summary.clone(),
                exports: node.exports.clone(),
                loc: node.loc,
            });
        }

        // Convert all edges
        for edge in graph.raw_edges() {
            edges.push(EdgeInfo {
                from: edge.source().index(),
                to: edge.target().index(),
                line: edge.weight.line,
            });
        }

        // Sort layers for consistent output
        let mut layers: Vec<String> = layer_set.into_iter().collect();
        layers.sort();

        Self {
            version: "1.0".to_string(),
            generated_at: chrono::Utc::now().to_rfc3339(),
            root_path: root_path.to_string(),
            nodes,
            edges,
            metadata: Metadata {
                node_count: graph.node_count(),
                edge_count: graph.edge_count(),
                max_depth,
                layers,
            },
        }
    }

    /// Serialize this CodebaseMap to a pretty-printed JSON string.
    ///
    /// Uses 2-space indentation for readability.
    ///
    /// # Errors
    /// Returns error if serialization fails (very rare for well-formed data).
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }

    /// Write this CodebaseMap to a JSON file.
    ///
    /// Creates the output directory if it doesn't exist.
    ///
    /// # Arguments
    /// * `output_path` - Path to write the JSON file (e.g., "codebase_map.json")
    ///
    /// # Errors
    /// Returns error if:
    /// - Directory creation fails
    /// - JSON serialization fails
    /// - File write fails
    pub fn write_to_file(&self, output_path: &Path) -> anyhow::Result<()> {
        // Create parent directory if needed
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let json = self.to_json()?;
        fs::write(output_path, json)?;

        Ok(())
    }

    /// Read a CodebaseMap from a JSON file.
    ///
    /// Used for round-trip testing and loading saved maps.
    ///
    /// # Arguments
    /// * `input_path` - Path to the JSON file to load
    ///
    /// # Errors
    /// Returns error if:
    /// - File read fails
    /// - JSON deserialization fails
    pub fn read_from_file(input_path: &Path) -> anyhow::Result<Self> {
        let json = fs::read_to_string(input_path)?;
        let map = serde_json::from_str(&json)?;
        Ok(map)
    }
}

/// Convert Layer enum to string for JSON serialization.
fn layer_to_string(layer: Layer) -> String {
    match layer {
        Layer::Entry => "Entry".to_string(),
        Layer::App => "App".to_string(),
        Layer::Core => "Core".to_string(),
        Layer::Platform => "Platform".to_string(),
        Layer::Driver => "Driver".to_string(),
        Layer::Arch => "Arch".to_string(),
        Layer::Unknown => "Unknown".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{FileNode, GraphBuilder};
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_layer_to_string() {
        assert_eq!(layer_to_string(Layer::Entry), "Entry");
        assert_eq!(layer_to_string(Layer::App), "App");
        assert_eq!(layer_to_string(Layer::Core), "Core");
        assert_eq!(layer_to_string(Layer::Platform), "Platform");
        assert_eq!(layer_to_string(Layer::Driver), "Driver");
        assert_eq!(layer_to_string(Layer::Arch), "Arch");
        assert_eq!(layer_to_string(Layer::Unknown), "Unknown");
    }

    #[test]
    fn test_codemap_from_graph_empty() {
        let graph = DepGraph::new();
        let map = CodebaseMap::from_graph(&graph, "/project");

        assert_eq!(map.version, "1.0");
        assert_eq!(map.root_path, "/project");
        assert_eq!(map.nodes.len(), 0);
        assert_eq!(map.edges.len(), 0);
        assert_eq!(map.metadata.node_count, 0);
        assert_eq!(map.metadata.edge_count, 0);
        assert_eq!(map.metadata.max_depth, 0);
        assert_eq!(map.metadata.layers.len(), 0);
    }

    #[test]
    fn test_codemap_from_graph_single_node() {
        let mut builder = GraphBuilder::new(PathBuf::from("/project"), false);

        let node = FileNode {
            path: PathBuf::from("/project/main.zig"),
            relative_path: "main.zig".to_string(),
            layer: Layer::Entry,
            depth: 0,
            summary: Some("Entry point".to_string()),
            exports: vec!["main".to_string()],
            loc: 42,
        };

        builder.add_file(node);
        let graph = builder.build();
        let map = CodebaseMap::from_graph(&graph, "/project");

        assert_eq!(map.nodes.len(), 1);
        assert_eq!(map.edges.len(), 0);
        assert_eq!(map.metadata.node_count, 1);
        assert_eq!(map.metadata.edge_count, 0);
        assert_eq!(map.metadata.max_depth, 0);
        assert_eq!(map.metadata.layers, vec!["Entry"]);

        let node_info = &map.nodes[0];
        assert_eq!(node_info.id, 0);
        assert_eq!(node_info.relative_path, "main.zig");
        assert_eq!(node_info.layer, "Entry");
        assert_eq!(node_info.depth, 0);
        assert_eq!(node_info.summary, Some("Entry point".to_string()));
        assert_eq!(node_info.exports, vec!["main"]);
        assert_eq!(node_info.loc, 42);
    }

    #[test]
    fn test_codemap_from_graph_with_edges() {
        let mut builder = GraphBuilder::new(PathBuf::from("/project"), false);

        // Create two nodes: main.zig -> lib.zig
        let main_node = FileNode {
            path: PathBuf::from("/project/main.zig"),
            relative_path: "main.zig".to_string(),
            layer: Layer::Entry,
            depth: 0,
            summary: None,
            exports: vec![],
            loc: 10,
        };

        let lib_node = FileNode {
            path: PathBuf::from("/project/lib.zig"),
            relative_path: "lib.zig".to_string(),
            layer: Layer::Unknown,
            depth: 1,
            summary: None,
            exports: vec![],
            loc: 20,
        };

        builder.add_file(main_node);
        builder.add_file(lib_node);

        let mut import_map = HashMap::new();
        import_map.insert(
            PathBuf::from("/project/main.zig"),
            vec![("lib.zig".to_string(), 5)],
        );

        builder.add_edges(&import_map);
        let graph = builder.build();
        let map = CodebaseMap::from_graph(&graph, "/project");

        assert_eq!(map.nodes.len(), 2);
        assert_eq!(map.edges.len(), 1);
        assert_eq!(map.metadata.node_count, 2);
        assert_eq!(map.metadata.edge_count, 1);
        assert_eq!(map.metadata.max_depth, 1);

        let edge = &map.edges[0];
        assert_eq!(edge.from, 0);
        assert_eq!(edge.to, 1);
        assert_eq!(edge.line, 5);
    }

    #[test]
    fn test_codemap_to_json() {
        let graph = DepGraph::new();
        let map = CodebaseMap::from_graph(&graph, "/project");

        let json = map.to_json().expect("JSON serialization should succeed");

        assert!(json.contains("\"version\": \"1.0\""));
        assert!(json.contains("\"root_path\": \"/project\""));
        assert!(json.contains("\"nodes\": []"));
        assert!(json.contains("\"edges\": []"));
    }

    #[test]
    fn test_codemap_write_and_read_roundtrip() {
        let mut builder = GraphBuilder::new(PathBuf::from("/project"), false);

        let node = FileNode {
            path: PathBuf::from("/project/main.zig"),
            relative_path: "main.zig".to_string(),
            layer: Layer::Entry,
            depth: 0,
            summary: Some("Test file".to_string()),
            exports: vec!["main".to_string(), "init".to_string()],
            loc: 100,
        };

        builder.add_file(node);
        let graph = builder.build();
        let original_map = CodebaseMap::from_graph(&graph, "/project");

        // Write to temp file
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let output_path = temp_dir.path().join("codebase_map.json");

        original_map
            .write_to_file(&output_path)
            .expect("Failed to write JSON file");

        // Read it back
        let loaded_map =
            CodebaseMap::read_from_file(&output_path).expect("Failed to read JSON file");

        // Verify round-trip equality (excluding timestamp)
        assert_eq!(loaded_map.version, original_map.version);
        assert_eq!(loaded_map.root_path, original_map.root_path);
        assert_eq!(loaded_map.nodes, original_map.nodes);
        assert_eq!(loaded_map.edges, original_map.edges);
        assert_eq!(loaded_map.metadata, original_map.metadata);
    }

    #[test]
    fn test_codemap_multiple_layers() {
        let mut builder = GraphBuilder::new(PathBuf::from("/project"), false);

        // Create nodes in different layers
        let nodes = vec![
            (Layer::Entry, "main.zig", 0),
            (Layer::App, "shell/main.zig", 1),
            (Layer::Core, "kernel/scheduler.zig", 2),
            (Layer::Driver, "drivers/uart.zig", 3),
        ];

        for (layer, path, depth) in nodes {
            let node = FileNode {
                path: PathBuf::from(format!("/project/{}", path)),
                relative_path: path.to_string(),
                layer,
                depth,
                summary: None,
                exports: vec![],
                loc: 10,
            };
            builder.add_file(node);
        }

        let graph = builder.build();
        let map = CodebaseMap::from_graph(&graph, "/project");

        assert_eq!(map.metadata.max_depth, 3);
        assert_eq!(map.metadata.layers.len(), 4);
        assert!(map.metadata.layers.contains(&"Entry".to_string()));
        assert!(map.metadata.layers.contains(&"App".to_string()));
        assert!(map.metadata.layers.contains(&"Core".to_string()));
        assert!(map.metadata.layers.contains(&"Driver".to_string()));
    }

    #[test]
    fn test_codemap_preserves_node_ids() {
        let mut builder = GraphBuilder::new(PathBuf::from("/project"), false);

        // Create nodes and verify their IDs are preserved
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
            depth: 1,
            summary: None,
            exports: vec![],
            loc: 10,
        };

        let idx_a = builder.add_file(node_a);
        let idx_b = builder.add_file(node_b);

        let mut import_map = HashMap::new();
        import_map.insert(
            PathBuf::from("/project/a.zig"),
            vec![("b.zig".to_string(), 1)],
        );

        builder.add_edges(&import_map);
        let graph = builder.build();
        let map = CodebaseMap::from_graph(&graph, "/project");

        // Verify node IDs match petgraph indices
        assert_eq!(map.nodes[0].id, idx_a.index());
        assert_eq!(map.nodes[1].id, idx_b.index());

        // Verify edge references correct IDs
        assert_eq!(map.edges[0].from, idx_a.index());
        assert_eq!(map.edges[0].to, idx_b.index());
    }
}
