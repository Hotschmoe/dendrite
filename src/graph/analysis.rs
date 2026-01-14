//! Graph analysis algorithms for cycle detection and metrics.
//!
//! This module provides:
//! - Cycle detection using Kosaraju's strongly connected components
//! - Depth metrics (entry points, max depth, deepest path)
//! - Fan-in/fan-out metrics
//! - Layer violation detection

use super::{DepGraph, FileNode, Import, Layer};
use petgraph::algo::kosaraju_scc;
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use petgraph::Direction;
use std::collections::{HashMap, HashSet};

/// A cycle in the dependency graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cycle {
    /// File paths in the cycle (in order of dependency)
    pub nodes: Vec<String>,
    /// Edges forming the cycle: (from_path, to_path, line_number)
    pub edges: Vec<(String, String, usize)>,
}

/// An edge that participates in a cycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CycleEdge {
    /// Source file path
    pub from: String,
    /// Target file path
    pub to: String,
    /// Line number where the import occurs
    pub line: usize,
}

/// Result of cycle detection analysis.
#[derive(Debug, Clone)]
pub struct CycleAnalysis {
    /// All cycles found in the graph
    pub cycles: Vec<Cycle>,
    /// All edges that participate in cycles
    pub cycle_edges: Vec<CycleEdge>,
    /// Node indices that are part of at least one cycle
    pub nodes_in_cycles: HashSet<NodeIndex>,
}

/// Complete analysis result for a dependency graph.
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    /// Cycle detection results
    pub cycles: Vec<Cycle>,
    /// All edges participating in cycles
    pub cycle_edges: Vec<CycleEdge>,
    /// Maximum depth from entry points
    pub max_depth: usize,
    /// Path from entry to deepest node
    pub deepest_path: Vec<String>,
    /// Files with high fan-out (many imports): (path, count)
    pub high_fan_out: Vec<(String, usize)>,
    /// Files with high fan-in (imported by many): (path, count)
    pub high_fan_in: Vec<(String, usize)>,
    /// Orphan files (zero fan-in AND zero fan-out)
    pub orphans: Vec<String>,
    /// Entry points (files with zero in-degree)
    pub entry_points: Vec<String>,
}

impl AnalysisResult {
    /// Check if there are any critical errors (cycles).
    pub fn has_errors(&self) -> bool {
        !self.cycles.is_empty()
    }

    /// Get the total number of issues found.
    pub fn issue_count(&self) -> usize {
        self.cycles.len()
    }
}

/// Find all cycles in the dependency graph.
///
/// Uses Kosaraju's algorithm to find strongly connected components (SCCs).
/// An SCC with more than one node indicates a cycle.
///
/// # Arguments
/// * `graph` - The dependency graph to analyze
///
/// # Returns
/// CycleAnalysis containing all cycles, cycle edges, and affected nodes
pub fn find_cycles(graph: &DepGraph) -> CycleAnalysis {
    let sccs = kosaraju_scc(graph);

    let mut cycles = Vec::new();
    let mut cycle_edges = Vec::new();
    let mut nodes_in_cycles = HashSet::new();

    for scc in sccs {
        // Skip single-node SCCs (not cycles)
        if scc.len() <= 1 {
            continue;
        }

        // Mark all nodes in this SCC as part of a cycle
        for &node_idx in &scc {
            nodes_in_cycles.insert(node_idx);
        }

        // Extract cycle path and edges
        let cycle = extract_cycle_from_scc(graph, &scc);
        cycles.push(cycle.0);
        cycle_edges.extend(cycle.1);
    }

    CycleAnalysis {
        cycles,
        cycle_edges,
        nodes_in_cycles,
    }
}

/// Extract a cycle representation from a strongly connected component.
///
/// This function reconstructs the cycle path by following edges within the SCC.
fn extract_cycle_from_scc(graph: &DepGraph, scc: &[NodeIndex]) -> (Cycle, Vec<CycleEdge>) {
    let scc_set: HashSet<NodeIndex> = scc.iter().copied().collect();
    let mut cycle_edges = Vec::new();

    // Get file paths for all nodes in the SCC
    let node_paths: Vec<String> = scc
        .iter()
        .map(|&idx| graph[idx].relative_path.clone())
        .collect();

    // Find all edges within the SCC
    let mut edges: Vec<(String, String, usize)> = Vec::new();

    for &source_idx in scc {
        for edge in graph.edges_directed(source_idx, Direction::Outgoing) {
            let target_idx = edge.target();
            if scc_set.contains(&target_idx) {
                let from = graph[source_idx].relative_path.clone();
                let to = graph[target_idx].relative_path.clone();
                let line = edge.weight().line;

                edges.push((from.clone(), to.clone(), line));
                cycle_edges.push(CycleEdge { from, to, line });
            }
        }
    }

    // Try to reconstruct an ordered cycle path
    let ordered_path = reconstruct_cycle_path(&edges, &node_paths);

    (
        Cycle {
            nodes: ordered_path,
            edges,
        },
        cycle_edges,
    )
}

/// Reconstruct an ordered cycle path from edges.
///
/// Starting from the first node, follows edges to construct A -> B -> C -> A.
fn reconstruct_cycle_path(edges: &[(String, String, usize)], nodes: &[String]) -> Vec<String> {
    if nodes.is_empty() {
        return vec![];
    }

    // Build adjacency map
    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
    for (from, to, _) in edges {
        adj.entry(from.as_str()).or_default().push(to.as_str());
    }

    // Start from first node and follow the path
    let mut path = Vec::new();
    let mut visited = HashSet::new();
    let start = &nodes[0];
    let mut current = start.as_str();

    loop {
        if visited.contains(current) {
            break;
        }
        path.push(current.to_string());
        visited.insert(current);

        if let Some(neighbors) = adj.get(current) {
            // Prefer unvisited neighbors, then any neighbor in the SCC
            if let Some(&next) = neighbors.iter().find(|&&n| !visited.contains(n)) {
                current = next;
            } else if let Some(&next) = neighbors.first() {
                // Complete the cycle
                if !path.contains(&next.to_string()) {
                    path.push(next.to_string());
                }
                break;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    // If we didn't get all nodes, just return them in original order
    if path.len() < nodes.len() {
        return nodes.to_vec();
    }

    path
}

/// Identify entry points (files with zero in-degree).
///
/// Entry points are files that are not imported by any other file.
/// These are typically the starting points of the dependency tree.
pub fn find_entry_points(graph: &DepGraph) -> Vec<String> {
    graph
        .node_indices()
        .filter(|&idx| graph.edges_directed(idx, Direction::Incoming).count() == 0)
        .map(|idx| graph[idx].relative_path.clone())
        .collect()
}

/// Calculate depth for each node from entry points.
///
/// Depth is the longest path from any entry point to the node.
/// Returns a map of node index to depth.
pub fn calculate_depths(graph: &DepGraph) -> HashMap<NodeIndex, usize> {
    let mut depths: HashMap<NodeIndex, usize> = HashMap::new();
    let entry_points: Vec<NodeIndex> = graph
        .node_indices()
        .filter(|&idx| graph.edges_directed(idx, Direction::Incoming).count() == 0)
        .collect();

    // Initialize entry points with depth 0
    for &entry in &entry_points {
        depths.insert(entry, 0);
    }

    // BFS to propagate depths
    let mut changed = true;
    while changed {
        changed = false;
        for node_idx in graph.node_indices() {
            let current_depth = depths.get(&node_idx).copied();

            for edge in graph.edges_directed(node_idx, Direction::Outgoing) {
                let target = edge.target();
                let new_depth = current_depth.unwrap_or(0) + 1;

                if depths.get(&target).is_none_or(|&d| d < new_depth) {
                    depths.insert(target, new_depth);
                    changed = true;
                }
            }
        }
    }

    depths
}

/// Find the deepest path from an entry point to a leaf.
pub fn find_deepest_path(graph: &DepGraph) -> Vec<String> {
    let depths = calculate_depths(graph);

    // Find the node with maximum depth
    let max_node = depths.iter().max_by_key(|(_, &d)| d);

    if let Some((&deepest_idx, _)) = max_node {
        // Reconstruct path from entry to this node
        reconstruct_path_to_node(graph, deepest_idx, &depths)
    } else {
        vec![]
    }
}

/// Reconstruct path from an entry point to a target node.
fn reconstruct_path_to_node(
    graph: &DepGraph,
    target: NodeIndex,
    depths: &HashMap<NodeIndex, usize>,
) -> Vec<String> {
    let mut path = vec![graph[target].relative_path.clone()];
    let mut current = target;

    // Walk backwards to entry
    while let Some(&current_depth) = depths.get(&current) {
        if current_depth == 0 {
            break;
        }

        // Find an incoming edge from a node with depth = current_depth - 1
        let parent = graph
            .edges_directed(current, Direction::Incoming)
            .find(|e| depths.get(&e.source()).copied() == Some(current_depth - 1))
            .map(|e| e.source());

        if let Some(parent_idx) = parent {
            path.push(graph[parent_idx].relative_path.clone());
            current = parent_idx;
        } else {
            break;
        }
    }

    path.reverse();
    path
}

/// Calculate fan-out (number of imports) for each node.
pub fn calculate_fan_out(graph: &DepGraph) -> Vec<(String, usize)> {
    let mut results: Vec<(String, usize)> = graph
        .node_indices()
        .map(|idx| {
            let path = graph[idx].relative_path.clone();
            let count = graph.edges_directed(idx, Direction::Outgoing).count();
            (path, count)
        })
        .collect();

    results.sort_by(|a, b| b.1.cmp(&a.1));
    results
}

/// Calculate fan-in (number of files importing this) for each node.
pub fn calculate_fan_in(graph: &DepGraph) -> Vec<(String, usize)> {
    let mut results: Vec<(String, usize)> = graph
        .node_indices()
        .map(|idx| {
            let path = graph[idx].relative_path.clone();
            let count = graph.edges_directed(idx, Direction::Incoming).count();
            (path, count)
        })
        .collect();

    results.sort_by(|a, b| b.1.cmp(&a.1));
    results
}

/// Find orphan files (zero fan-in AND zero fan-out).
pub fn find_orphans(graph: &DepGraph) -> Vec<String> {
    graph
        .node_indices()
        .filter(|&idx| {
            graph.edges_directed(idx, Direction::Incoming).count() == 0
                && graph.edges_directed(idx, Direction::Outgoing).count() == 0
        })
        .map(|idx| graph[idx].relative_path.clone())
        .collect()
}

/// Perform complete analysis of a dependency graph.
///
/// This is the main entry point for analysis, running all analyses and
/// combining results into a single AnalysisResult.
pub fn analyze(graph: &DepGraph, fan_threshold: usize) -> AnalysisResult {
    let cycle_analysis = find_cycles(graph);
    let depths = calculate_depths(graph);
    let max_depth = depths.values().copied().max().unwrap_or(0);
    let deepest_path = find_deepest_path(graph);
    let entry_points = find_entry_points(graph);
    let orphans = find_orphans(graph);

    let all_fan_out = calculate_fan_out(graph);
    let all_fan_in = calculate_fan_in(graph);

    let high_fan_out: Vec<(String, usize)> = all_fan_out
        .into_iter()
        .filter(|(_, count)| *count >= fan_threshold)
        .collect();

    let high_fan_in: Vec<(String, usize)> = all_fan_in
        .into_iter()
        .filter(|(_, count)| *count >= fan_threshold)
        .collect();

    AnalysisResult {
        cycles: cycle_analysis.cycles,
        cycle_edges: cycle_analysis.cycle_edges,
        max_depth,
        deepest_path,
        high_fan_out,
        high_fan_in,
        orphans,
        entry_points,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::GraphBuilder;
    use std::path::PathBuf;

    fn create_test_node(builder: &mut GraphBuilder, name: &str) {
        let node = FileNode {
            path: PathBuf::from(format!("/project/{}", name)),
            relative_path: name.to_string(),
            layer: Layer::Unknown,
            depth: 0,
            summary: None,
            exports: vec![],
            loc: 10,
        };
        builder.add_file(node);
    }

    // Test 2.1.5: DAG should return no cycles
    #[test]
    fn test_no_cycles_in_dag() {
        let mut builder = GraphBuilder::new(PathBuf::from("/project"), false);

        create_test_node(&mut builder, "a.zig");
        create_test_node(&mut builder, "b.zig");
        create_test_node(&mut builder, "c.zig");

        // Linear chain: A -> B -> C (no cycles)
        let mut import_map = std::collections::HashMap::new();
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
        let analysis = find_cycles(&graph);

        assert!(analysis.cycles.is_empty(), "DAG should have no cycles");
        assert!(analysis.cycle_edges.is_empty());
        assert!(analysis.nodes_in_cycles.is_empty());
    }

    // Test 2.1.6: Simple two-node cycle (A <-> B)
    #[test]
    fn test_simple_cycle() {
        let mut builder = GraphBuilder::new(PathBuf::from("/project"), false);

        create_test_node(&mut builder, "a.zig");
        create_test_node(&mut builder, "b.zig");

        // A -> B and B -> A
        let mut import_map = std::collections::HashMap::new();
        import_map.insert(
            PathBuf::from("/project/a.zig"),
            vec![("b.zig".to_string(), 5)],
        );
        import_map.insert(
            PathBuf::from("/project/b.zig"),
            vec![("a.zig".to_string(), 3)],
        );

        builder.add_edges(&import_map);
        let graph = builder.build();
        let analysis = find_cycles(&graph);

        assert_eq!(analysis.cycles.len(), 1, "Should find exactly 1 cycle");
        assert_eq!(analysis.nodes_in_cycles.len(), 2, "Both nodes should be in cycle");

        let cycle = &analysis.cycles[0];
        assert!(cycle.nodes.contains(&"a.zig".to_string()));
        assert!(cycle.nodes.contains(&"b.zig".to_string()));
        assert_eq!(cycle.edges.len(), 2, "Cycle should have 2 edges");
    }

    // Test 2.1.7: Complex cycle (A -> B -> C -> A)
    #[test]
    fn test_complex_cycle() {
        let mut builder = GraphBuilder::new(PathBuf::from("/project"), false);

        create_test_node(&mut builder, "a.zig");
        create_test_node(&mut builder, "b.zig");
        create_test_node(&mut builder, "c.zig");

        // A -> B -> C -> A
        let mut import_map = std::collections::HashMap::new();
        import_map.insert(
            PathBuf::from("/project/a.zig"),
            vec![("b.zig".to_string(), 1)],
        );
        import_map.insert(
            PathBuf::from("/project/b.zig"),
            vec![("c.zig".to_string(), 1)],
        );
        import_map.insert(
            PathBuf::from("/project/c.zig"),
            vec![("a.zig".to_string(), 1)],
        );

        builder.add_edges(&import_map);
        let graph = builder.build();
        let analysis = find_cycles(&graph);

        assert_eq!(analysis.cycles.len(), 1, "Should find exactly 1 cycle");
        assert_eq!(analysis.nodes_in_cycles.len(), 3, "All 3 nodes should be in cycle");

        let cycle = &analysis.cycles[0];
        assert_eq!(cycle.nodes.len(), 3, "Cycle should have 3 nodes");
        assert_eq!(cycle.edges.len(), 3, "Cycle should have 3 edges");
    }

    // Test 2.1.8: Multiple independent cycles
    #[test]
    fn test_multiple_cycles() {
        let mut builder = GraphBuilder::new(PathBuf::from("/project"), false);

        // First cycle: a <-> b
        create_test_node(&mut builder, "a.zig");
        create_test_node(&mut builder, "b.zig");

        // Second cycle: c <-> d
        create_test_node(&mut builder, "c.zig");
        create_test_node(&mut builder, "d.zig");

        let mut import_map = std::collections::HashMap::new();
        import_map.insert(
            PathBuf::from("/project/a.zig"),
            vec![("b.zig".to_string(), 1)],
        );
        import_map.insert(
            PathBuf::from("/project/b.zig"),
            vec![("a.zig".to_string(), 1)],
        );
        import_map.insert(
            PathBuf::from("/project/c.zig"),
            vec![("d.zig".to_string(), 1)],
        );
        import_map.insert(
            PathBuf::from("/project/d.zig"),
            vec![("c.zig".to_string(), 1)],
        );

        builder.add_edges(&import_map);
        let graph = builder.build();
        let analysis = find_cycles(&graph);

        assert_eq!(analysis.cycles.len(), 2, "Should find 2 independent cycles");
        assert_eq!(analysis.nodes_in_cycles.len(), 4, "All 4 nodes should be in cycles");
    }

    #[test]
    fn test_entry_points() {
        let mut builder = GraphBuilder::new(PathBuf::from("/project"), false);

        create_test_node(&mut builder, "main.zig");
        create_test_node(&mut builder, "lib.zig");
        create_test_node(&mut builder, "util.zig");

        // main -> lib -> util
        let mut import_map = std::collections::HashMap::new();
        import_map.insert(
            PathBuf::from("/project/main.zig"),
            vec![("lib.zig".to_string(), 1)],
        );
        import_map.insert(
            PathBuf::from("/project/lib.zig"),
            vec![("util.zig".to_string(), 1)],
        );

        builder.add_edges(&import_map);
        let graph = builder.build();
        let entry_points = find_entry_points(&graph);

        assert_eq!(entry_points.len(), 1, "Should have 1 entry point");
        assert_eq!(entry_points[0], "main.zig");
    }

    #[test]
    fn test_depth_calculation() {
        let mut builder = GraphBuilder::new(PathBuf::from("/project"), false);

        create_test_node(&mut builder, "main.zig");
        create_test_node(&mut builder, "lib.zig");
        create_test_node(&mut builder, "util.zig");

        // main -> lib -> util
        let mut import_map = std::collections::HashMap::new();
        import_map.insert(
            PathBuf::from("/project/main.zig"),
            vec![("lib.zig".to_string(), 1)],
        );
        import_map.insert(
            PathBuf::from("/project/lib.zig"),
            vec![("util.zig".to_string(), 1)],
        );

        builder.add_edges(&import_map);
        let graph = builder.build();
        let depths = calculate_depths(&graph);

        assert_eq!(depths.values().copied().max(), Some(2));
    }

    #[test]
    fn test_fan_metrics() {
        let mut builder = GraphBuilder::new(PathBuf::from("/project"), false);

        create_test_node(&mut builder, "main.zig");
        create_test_node(&mut builder, "a.zig");
        create_test_node(&mut builder, "b.zig");
        create_test_node(&mut builder, "c.zig");
        create_test_node(&mut builder, "common.zig");

        // main imports a, b, c (high fan-out)
        // a, b, c all import common (high fan-in for common)
        let mut import_map = std::collections::HashMap::new();
        import_map.insert(
            PathBuf::from("/project/main.zig"),
            vec![
                ("a.zig".to_string(), 1),
                ("b.zig".to_string(), 2),
                ("c.zig".to_string(), 3),
            ],
        );
        import_map.insert(
            PathBuf::from("/project/a.zig"),
            vec![("common.zig".to_string(), 1)],
        );
        import_map.insert(
            PathBuf::from("/project/b.zig"),
            vec![("common.zig".to_string(), 1)],
        );
        import_map.insert(
            PathBuf::from("/project/c.zig"),
            vec![("common.zig".to_string(), 1)],
        );

        builder.add_edges(&import_map);
        let graph = builder.build();

        let fan_out = calculate_fan_out(&graph);
        let fan_in = calculate_fan_in(&graph);

        // main.zig should have highest fan-out (3)
        assert_eq!(fan_out[0].0, "main.zig");
        assert_eq!(fan_out[0].1, 3);

        // common.zig should have highest fan-in (3)
        assert_eq!(fan_in[0].0, "common.zig");
        assert_eq!(fan_in[0].1, 3);
    }

    #[test]
    fn test_orphan_detection() {
        let mut builder = GraphBuilder::new(PathBuf::from("/project"), false);

        create_test_node(&mut builder, "main.zig");
        create_test_node(&mut builder, "lib.zig");
        create_test_node(&mut builder, "orphan.zig");

        // main -> lib, orphan has no connections
        let mut import_map = std::collections::HashMap::new();
        import_map.insert(
            PathBuf::from("/project/main.zig"),
            vec![("lib.zig".to_string(), 1)],
        );

        builder.add_edges(&import_map);
        let graph = builder.build();
        let orphans = find_orphans(&graph);

        assert_eq!(orphans.len(), 1, "Should find 1 orphan");
        assert_eq!(orphans[0], "orphan.zig");
    }

    #[test]
    fn test_full_analysis() {
        let mut builder = GraphBuilder::new(PathBuf::from("/project"), false);

        create_test_node(&mut builder, "main.zig");
        create_test_node(&mut builder, "lib.zig");

        let mut import_map = std::collections::HashMap::new();
        import_map.insert(
            PathBuf::from("/project/main.zig"),
            vec![("lib.zig".to_string(), 1)],
        );

        builder.add_edges(&import_map);
        let graph = builder.build();
        let result = analyze(&graph, 3);

        assert!(result.cycles.is_empty());
        assert!(!result.entry_points.is_empty());
        assert!(!result.has_errors());
    }
}
