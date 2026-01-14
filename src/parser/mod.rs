//! Language-specific parsers for extracting imports and exports.

mod zig;
mod rust;

pub use zig::{
    extract_doc_comment, extract_imports, extract_public_decls, parse_file, ZigFile, ZigImport,
};

pub use rust::{
    extract_extern_crates, extract_mod_declarations, extract_pub_items,
    extract_rust_doc_comment, extract_use_statements, parse_rust_file, ImportKind, RustFile,
    RustImport,
};

/// Result of parsing a source file.
#[derive(Debug, Clone)]
pub struct ParseResult {
    pub imports: Vec<String>,
    pub exports: Vec<String>,
    pub doc_comment: Option<String>,
    pub loc: usize,
}
