//! App state management for the TUI.
//!
//! Defines the core App struct that holds all TUI state, including the graph,
//! analysis results, selected node, view mode, and active panel.

use super::layout::GraphLayout;
use crate::graph::analysis::AnalysisResult;
use crate::graph::DepGraph;
use petgraph::graph::NodeIndex;

/// View mode for graph visualization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    /// Normal view - show full graph
    Normal,
    /// Highlight what the selected file imports
    Imports,
    /// Highlight what imports the selected file
    Dependents,
    /// Search/filter mode
    Search,
}

/// Active panel in the TUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivePanel {
    /// Graph view panel
    Graph,
    /// File details panel
    Details,
    /// Alerts (cycles/violations) panel
    Alerts,
}

/// Main application state for the TUI.
#[derive(Debug)]
pub struct App {
    /// Dependency graph
    pub graph: DepGraph,
    /// Analysis results (cycles, violations, metrics)
    pub analysis: AnalysisResult,
    /// Currently selected node in the graph
    pub selected_node: Option<NodeIndex>,
    /// Current view mode
    pub mode: ViewMode,
    /// Currently active panel
    pub panel: ActivePanel,
    /// Whether the application should quit
    pub should_quit: bool,
    /// Graph layout for visualization
    pub layout: GraphLayout,
    /// Viewport scroll offset (x, y)
    pub viewport_offset: (i16, i16),
    /// Details panel scroll offset
    pub details_scroll: usize,
    /// Selected alert index in alerts panel
    pub selected_alert: usize,
}

impl App {
    /// Create a new App instance with the given graph and analysis results.
    pub fn new(graph: DepGraph, analysis: AnalysisResult) -> Self {
        // Select the first node by default (if any)
        let selected_node = graph.node_indices().next();

        // Compute layout
        let layout = GraphLayout::compute(&graph);

        Self {
            graph,
            analysis,
            selected_node,
            mode: ViewMode::Normal,
            panel: ActivePanel::Graph,
            should_quit: false,
            layout,
            viewport_offset: (0, 0),
            details_scroll: 0,
            selected_alert: 0,
        }
    }

    /// Mark the application for quitting.
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Navigate to the next node in the current layer.
    pub fn select_next_in_layer(&mut self) {
        if let Some(current_idx) = self.selected_node {
            let current_node = &self.graph[current_idx];
            let current_depth = current_node.depth;

            // Find nodes at the same depth
            let mut same_depth_nodes: Vec<_> = self
                .graph
                .node_indices()
                .filter(|&idx| self.graph[idx].depth == current_depth)
                .collect();

            // Sort by relative path for consistent ordering
            same_depth_nodes.sort_by_key(|&idx| &self.graph[idx].relative_path);

            // Find current position and move to next
            if let Some(pos) = same_depth_nodes.iter().position(|&idx| idx == current_idx) {
                let next_pos = (pos + 1) % same_depth_nodes.len();
                self.selected_node = Some(same_depth_nodes[next_pos]);
            }
        }
    }

    /// Navigate to the previous node in the current layer.
    pub fn select_prev_in_layer(&mut self) {
        if let Some(current_idx) = self.selected_node {
            let current_node = &self.graph[current_idx];
            let current_depth = current_node.depth;

            // Find nodes at the same depth
            let mut same_depth_nodes: Vec<_> = self
                .graph
                .node_indices()
                .filter(|&idx| self.graph[idx].depth == current_depth)
                .collect();

            // Sort by relative path for consistent ordering
            same_depth_nodes.sort_by_key(|&idx| &self.graph[idx].relative_path);

            // Find current position and move to previous
            if let Some(pos) = same_depth_nodes.iter().position(|&idx| idx == current_idx) {
                let prev_pos = if pos == 0 {
                    same_depth_nodes.len() - 1
                } else {
                    pos - 1
                };
                self.selected_node = Some(same_depth_nodes[prev_pos]);
            }
        }
    }

    /// Navigate to a node in the next layer (deeper).
    pub fn select_next_layer(&mut self) {
        if let Some(current_idx) = self.selected_node {
            let current_depth = self.graph[current_idx].depth;

            // Find first node at next depth level
            if let Some(next_node) = self
                .graph
                .node_indices()
                .find(|&idx| self.graph[idx].depth == current_depth + 1)
            {
                self.selected_node = Some(next_node);
            }
        }
    }

    /// Navigate to a node in the previous layer (shallower).
    pub fn select_prev_layer(&mut self) {
        if let Some(current_idx) = self.selected_node {
            let current_depth = self.graph[current_idx].depth;

            if current_depth > 0 {
                // Find first node at previous depth level
                if let Some(prev_node) = self
                    .graph
                    .node_indices()
                    .find(|&idx| self.graph[idx].depth == current_depth - 1)
                {
                    self.selected_node = Some(prev_node);
                }
            }
        }
    }

    /// Scroll the viewport.
    pub fn scroll_viewport(&mut self, dx: i16, dy: i16) {
        self.viewport_offset.0 = (self.viewport_offset.0 + dx).clamp(-100, 100);
        self.viewport_offset.1 = (self.viewport_offset.1 + dy).clamp(-100, 100);
    }

    /// Scroll the details panel down.
    pub fn scroll_details_down(&mut self) {
        self.details_scroll = self.details_scroll.saturating_add(1);
    }

    /// Scroll the details panel up.
    pub fn scroll_details_up(&mut self) {
        self.details_scroll = self.details_scroll.saturating_sub(1);
    }

    /// Move to the next alert in the alerts panel.
    pub fn next_alert(&mut self) {
        let total_alerts = self.analysis.cycles.len() + self.analysis.violations.len();
        if total_alerts > 0 {
            self.selected_alert = (self.selected_alert + 1) % total_alerts;
        }
    }

    /// Move to the previous alert in the alerts panel.
    pub fn prev_alert(&mut self) {
        let total_alerts = self.analysis.cycles.len() + self.analysis.violations.len();
        if total_alerts > 0 {
            self.selected_alert = if self.selected_alert == 0 {
                total_alerts - 1
            } else {
                self.selected_alert - 1
            };
        }
    }

    /// Jump to the file node associated with the currently selected alert.
    pub fn jump_to_selected_alert(&mut self) {
        let cycles_count = self.analysis.cycles.len();

        if self.selected_alert < cycles_count {
            // Selected alert is a cycle
            let cycle = &self.analysis.cycles[self.selected_alert];
            if let Some(first_file) = cycle.nodes.first() {
                // Find the node index for this file
                if let Some(idx) = self.graph.node_indices()
                    .find(|&idx| &self.graph[idx].relative_path == first_file) {
                    self.selected_node = Some(idx);
                    self.panel = ActivePanel::Graph;
                }
            }
        } else {
            // Selected alert is a violation
            let violation_idx = self.selected_alert - cycles_count;
            if let Some(violation) = self.analysis.violations.get(violation_idx) {
                // Find the node index for the violating file
                if let Some(idx) = self.graph.node_indices()
                    .find(|&idx| self.graph[idx].relative_path == violation.file) {
                    self.selected_node = Some(idx);
                    self.panel = ActivePanel::Graph;
                }
            }
        }
    }
}
