//! Dendrite - Codebase mapping and dependency analysis
//!
//! This library provides tools for parsing source files, building dependency
//! graphs, and generating documentation for codebases.

#![allow(dead_code)]
#![allow(unused_imports)]

pub mod parser;
pub mod graph;
pub mod output;
pub mod config;
pub mod discovery;

// Re-export main types for convenience
pub use graph::{DepGraph, FileNode, Layer};
pub use parser::ParseResult;
pub use discovery::{DiscoveryConfig, discover_files, discover_and_parse};
