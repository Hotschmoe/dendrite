//! Language-specific parsers for extracting imports and exports.

mod zig;

pub use zig::{
    extract_doc_comment, extract_imports, extract_public_decls, parse_file, ZigFile, ZigImport,
};

/// Result of parsing a source file.
#[derive(Debug, Clone)]
pub struct ParseResult {
    pub imports: Vec<String>,
    pub exports: Vec<String>,
    pub doc_comment: Option<String>,
    pub loc: usize,
}
