//! App state management for the TUI.
//!
//! Defines the core App struct that holds all TUI state, including the graph,
//! analysis results, selected node, view mode, and active panel.

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
}

impl App {
    /// Create a new App instance with the given graph and analysis results.
    pub fn new(graph: DepGraph, analysis: AnalysisResult) -> Self {
        // Select the first node by default (if any)
        let selected_node = graph.node_indices().next();

        Self {
            graph,
            analysis,
            selected_node,
            mode: ViewMode::Normal,
            panel: ActivePanel::Graph,
            should_quit: false,
        }
    }

    /// Mark the application for quitting.
    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}
