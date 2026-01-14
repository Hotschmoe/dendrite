//! Dependency graph construction and analysis.

use petgraph::graph::DiGraph;
use std::path::PathBuf;

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

// TODO: Implement GraphBuilder for constructing DepGraph from parsed files
// TODO: Implement analysis functions (cycles, metrics, violations)
// TODO: Add layer classification logic
// TODO: Add tests with fixture graphs
