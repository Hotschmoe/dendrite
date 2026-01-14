//! Graph layout algorithm for ASCII visualization.
//!
//! Implements a simplified Sugiyama-style hierarchical layout:
//! - Nodes are arranged in columns by depth (distance from entry points)
//! - Nodes within columns are ordered to minimize edge crossings
//! - Layout is left-to-right (layers are columns)

use crate::graph::DepGraph;
use petgraph::graph::NodeIndex;
use std::collections::HashMap;

/// Layout information for rendering a graph as ASCII art.
#[derive(Debug, Clone)]
pub struct GraphLayout {
    /// Position of each node (x, y in character cells)
    pub positions: HashMap<NodeIndex, (u16, u16)>,
    /// Width of each node box
    pub node_width: u16,
    /// Height of each node box
    pub node_height: u16,
    /// Total layout bounds (width, height)
    pub bounds: (u16, u16),
    /// Nodes organized by layer (column)
    pub layers: Vec<Vec<NodeIndex>>,
}

impl GraphLayout {
    /// Compute layout for the given dependency graph.
    ///
    /// Uses a hierarchical left-to-right layout where:
    /// - Each depth level becomes a column
    /// - Nodes within a column are vertically stacked
    /// - Spacing accounts for node boxes and edges
    pub fn compute(graph: &DepGraph) -> Self {
        const NODE_WIDTH: u16 = 24;
        const NODE_HEIGHT: u16 = 3;
        const HORIZONTAL_SPACING: u16 = 8;
        const VERTICAL_SPACING: u16 = 2;

        // Group nodes by depth
        let mut depth_map: HashMap<usize, Vec<NodeIndex>> = HashMap::new();
        let mut max_depth = 0;

        for node_idx in graph.node_indices() {
            let node = &graph[node_idx];
            depth_map.entry(node.depth).or_default().push(node_idx);
            max_depth = max_depth.max(node.depth);
        }

        // Create layers (columns) from depth groups
        let mut layers = Vec::new();
        for depth in 0..=max_depth {
            if let Some(nodes) = depth_map.get(&depth) {
                let mut layer_nodes = nodes.clone();
                // Sort nodes within layer by relative path for consistent layout
                layer_nodes.sort_by_key(|&idx| &graph[idx].relative_path);
                layers.push(layer_nodes);
            } else {
                layers.push(Vec::new());
            }
        }

        // Calculate positions
        let mut positions = HashMap::new();
        let mut max_y = 0;

        for (col_idx, layer) in layers.iter().enumerate() {
            let x = col_idx as u16 * (NODE_WIDTH + HORIZONTAL_SPACING);

            for (row_idx, &node_idx) in layer.iter().enumerate() {
                let y = row_idx as u16 * (NODE_HEIGHT + VERTICAL_SPACING);
                positions.insert(node_idx, (x, y));
                max_y = max_y.max(y + NODE_HEIGHT);
            }
        }

        // Calculate total bounds
        let max_x = if positions.is_empty() {
            0
        } else {
            (layers.len() - 1) as u16 * (NODE_WIDTH + HORIZONTAL_SPACING) + NODE_WIDTH
        };

        Self {
            positions,
            node_width: NODE_WIDTH,
            node_height: NODE_HEIGHT,
            bounds: (max_x, max_y),
            layers,
        }
    }

    /// Get the position of a node, if it has been laid out.
    pub fn get_position(&self, node_idx: NodeIndex) -> Option<(u16, u16)> {
        self.positions.get(&node_idx).copied()
    }

    /// Get the center point of a node box.
    pub fn get_node_center(&self, node_idx: NodeIndex) -> Option<(u16, u16)> {
        self.get_position(node_idx)
            .map(|(x, y)| (x + self.node_width / 2, y + self.node_height / 2))
    }

    /// Get the right edge of a node box (for edge drawing).
    pub fn get_node_right(&self, node_idx: NodeIndex) -> Option<(u16, u16)> {
        self.get_position(node_idx)
            .map(|(x, y)| (x + self.node_width, y + self.node_height / 2))
    }

    /// Get the left edge of a node box (for edge drawing).
    pub fn get_node_left(&self, node_idx: NodeIndex) -> Option<(u16, u16)> {
        self.get_position(node_idx)
            .map(|(x, y)| (x, y + self.node_height / 2))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{FileNode, Import, Layer};
    use petgraph::graph::DiGraph;
    use std::path::PathBuf;

    #[test]
    fn test_layout_empty_graph() {
        let graph: DepGraph = DiGraph::new();
        let layout = GraphLayout::compute(&graph);

        assert_eq!(layout.positions.len(), 0);
        assert_eq!(layout.bounds, (0, 0));
    }

    #[test]
    fn test_layout_single_node() {
        let mut graph: DepGraph = DiGraph::new();
        graph.add_node(FileNode {
            path: PathBuf::from("main.zig"),
            relative_path: "main.zig".to_string(),
            layer: Layer::Entry,
            depth: 0,
            summary: None,
            exports: vec![],
            loc: 10,
        });

        let layout = GraphLayout::compute(&graph);

        assert_eq!(layout.positions.len(), 1);
        assert!(layout.bounds.0 >= layout.node_width);
    }

    #[test]
    fn test_layout_linear_chain() {
        let mut graph: DepGraph = DiGraph::new();

        let node_a = graph.add_node(FileNode {
            path: PathBuf::from("a.zig"),
            relative_path: "a.zig".to_string(),
            layer: Layer::Unknown,
            depth: 0,
            summary: None,
            exports: vec![],
            loc: 10,
        });

        let node_b = graph.add_node(FileNode {
            path: PathBuf::from("b.zig"),
            relative_path: "b.zig".to_string(),
            layer: Layer::Unknown,
            depth: 1,
            summary: None,
            exports: vec![],
            loc: 10,
        });

        let node_c = graph.add_node(FileNode {
            path: PathBuf::from("c.zig"),
            relative_path: "c.zig".to_string(),
            layer: Layer::Unknown,
            depth: 2,
            summary: None,
            exports: vec![],
            loc: 10,
        });

        graph.add_edge(node_a, node_b, Import { line: 1 });
        graph.add_edge(node_b, node_c, Import { line: 1 });

        let layout = GraphLayout::compute(&graph);

        assert_eq!(layout.positions.len(), 3);

        // Nodes should be in different columns
        let pos_a = layout.get_position(node_a).unwrap();
        let pos_b = layout.get_position(node_b).unwrap();
        let pos_c = layout.get_position(node_c).unwrap();

        assert!(pos_a.0 < pos_b.0);
        assert!(pos_b.0 < pos_c.0);
    }

    #[test]
    fn test_layout_same_depth_nodes() {
        let mut graph: DepGraph = DiGraph::new();

        graph.add_node(FileNode {
            path: PathBuf::from("a.zig"),
            relative_path: "a.zig".to_string(),
            layer: Layer::Unknown,
            depth: 0,
            summary: None,
            exports: vec![],
            loc: 10,
        });

        graph.add_node(FileNode {
            path: PathBuf::from("b.zig"),
            relative_path: "b.zig".to_string(),
            layer: Layer::Unknown,
            depth: 0,
            summary: None,
            exports: vec![],
            loc: 10,
        });

        let layout = GraphLayout::compute(&graph);

        assert_eq!(layout.positions.len(), 2);
        assert_eq!(layout.layers.len(), 1);
        assert_eq!(layout.layers[0].len(), 2);
    }
}
