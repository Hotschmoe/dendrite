//! Zig source file parser for extracting @import statements.

use once_cell::sync::Lazy;
use regex::Regex;
use std::path::PathBuf;

/// A single @import statement from a Zig file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZigImport {
    /// Import target: "scheduler.zig" or "std"
    pub target: String,
    /// Line number (1-indexed) for error messages
    pub line: usize,
    /// true for @import("std") and other standard library imports
    pub is_std: bool,
}

/// Parsed Zig file with imports, exports, and metadata.
#[derive(Debug, Clone)]
pub struct ZigFile {
    /// Absolute path to the source file
    pub path: PathBuf,
    /// All @import statements found in the file
    pub imports: Vec<ZigImport>,
    /// Top-level documentation comment (//! ...)
    pub doc_comment: Option<String>,
    /// Public declarations (pub fn, pub const, etc.)
    pub exports: Vec<String>,
    /// Lines of code (non-comment, non-blank)
    pub loc: usize,
}

// Regex pattern for @import("...") statements
static IMPORT_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"@import\("([^"]+)"\)"#).expect("Invalid import regex")
});

// Regex pattern for pub fn, pub const, pub var declarations
// Captures the identifier name following the pub keyword
static PUBLIC_DECL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?m)^\s*pub\s+(fn|const|var)\s+([a-zA-Z_][a-zA-Z0-9_]*)").unwrap()
});

/// Extract all @import statements from Zig source code.
///
/// This function:
/// - Strips line comments (//) and block comments (/* */)
/// - Avoids matching imports inside string literals
/// - Tracks line numbers for each import (1-indexed)
/// - Detects standard library imports (@import("std"))
///
/// # Arguments
/// * `source` - Raw Zig source code
///
/// # Returns
/// Vector of ZigImport with target path, line number, and std detection
///
/// # Examples
///
/// ```
/// # use dendrite::parser::zig::extract_imports;
/// let source = r#"
/// const std = @import("std");
/// const lib = @import("lib.zig");
/// "#;
/// let imports = extract_imports(source);
/// assert_eq!(imports.len(), 2);
/// assert_eq!(imports[0].target, "std");
/// assert!(imports[0].is_std);
/// assert_eq!(imports[1].target, "lib.zig");
/// assert!(!imports[1].is_std);
/// ```
pub fn extract_imports(source: &str) -> Vec<ZigImport> {
    let cleaned = remove_comments_and_strings(source);
    let mut imports = Vec::new();

    // Track line numbers by counting newlines up to each match
    for cap in IMPORT_REGEX.captures_iter(&cleaned) {
        let match_pos = cap.get(0).unwrap().start();
        // Count lines in original source up to this position
        let line = source[..match_pos].lines().count();
        let target = cap[1].to_string();
        let is_std = target == "std" || target.starts_with("std.");

        imports.push(ZigImport {
            target,
            line,
            is_std,
        });
    }

    imports
}

/// Extract all public declarations from Zig source code.
///
/// Finds pub fn, pub const, and pub var declarations and returns their identifier names.
/// Does NOT match:
/// - Private declarations (without pub)
/// - Declarations inside comments
/// - Declarations inside strings
///
/// # Examples
///
/// ```
/// # use dendrite::parser::zig::extract_public_decls;
/// let source = r#"
/// pub fn add(a: i32, b: i32) i32 { return a + b; }
/// pub const MAX_SIZE: usize = 1024;
/// pub var counter: u32 = 0;
/// fn private_fn() void {}
/// "#;
/// let decls = extract_public_decls(source);
/// assert_eq!(decls, vec!["add", "MAX_SIZE", "counter"]);
/// ```
pub fn extract_public_decls(source: &str) -> Vec<String> {
    // First, remove comments and string literals to avoid false matches
    let cleaned = remove_comments_and_strings(source);

    PUBLIC_DECL_REGEX
        .captures_iter(&cleaned)
        .map(|cap| cap[2].to_string())
        .collect()
}

/// Remove comments and string literals from source code to avoid false matches.
///
/// This is a simplified approach that:
/// - Removes // line comments
/// - Removes /* block comments */
/// - Removes "string literals"
/// - Removes 'c' character literals
fn remove_comments_and_strings(source: &str) -> String {
    let mut result = String::with_capacity(source.len());
    let mut chars = source.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            // Line comment
            '/' if chars.peek() == Some(&'/') => {
                chars.next(); // consume second '/'
                // Skip until end of line
                while let Some(c) = chars.next() {
                    if c == '\n' {
                        result.push('\n'); // preserve newlines for line-based regex
                        break;
                    }
                }
            }
            // Block comment
            '/' if chars.peek() == Some(&'*') => {
                chars.next(); // consume '*'
                // Skip until */
                while let Some(c) = chars.next() {
                    if c == '*' && chars.peek() == Some(&'/') {
                        chars.next(); // consume '/'
                        break;
                    }
                    if c == '\n' {
                        result.push('\n'); // preserve newlines
                    }
                }
            }
            // String literal
            '"' => {
                result.push(' '); // replace with space
                let mut escaped = false;
                while let Some(c) = chars.next() {
                    if escaped {
                        escaped = false;
                    } else if c == '\\' {
                        escaped = true;
                    } else if c == '"' {
                        break;
                    } else if c == '\n' {
                        result.push('\n'); // preserve newlines
                    }
                }
            }
            // Character literal
            '\'' => {
                result.push(' '); // replace with space
                let mut escaped = false;
                while let Some(c) = chars.next() {
                    if escaped {
                        escaped = false;
                    } else if c == '\\' {
                        escaped = true;
                    } else if c == '\'' {
                        break;
                    }
                }
            }
            // Regular character
            _ => result.push(ch),
        }
    }

    result
}

/// Extracts module-level documentation from //! comments at the start of the file.
///
/// Collects consecutive //! lines into a single string, stripping the //! prefix.
/// Stops at the first non-//! line (including blank lines or regular // comments).
///
/// # Arguments
/// * `source` - The source code to parse
///
/// # Returns
/// * `Some(String)` - The concatenated doc comments with newlines preserved
/// * `None` - If no module doc comment exists at the start of the file
///
/// # Examples
/// ```
/// # use dendrite::parser::zig::extract_doc_comment;
/// let source = "//! Module docs.\n//! Second line.\n\nconst std = @import(\"std\");";
/// let doc = extract_doc_comment(source);
/// assert_eq!(doc, Some("Module docs.\nSecond line.".to_string()));
/// ```
pub fn extract_doc_comment(source: &str) -> Option<String> {
    let mut doc_lines = Vec::new();

    for line in source.lines() {
        let trimmed = line.trim_start();

        if let Some(content) = trimmed.strip_prefix("//!") {
            // Strip the prefix and any single leading space after //!
            let cleaned = content.strip_prefix(' ').unwrap_or(content);
            doc_lines.push(cleaned.to_string());
        } else if trimmed.is_empty() && doc_lines.is_empty() {
            // Skip leading blank lines before doc comments start
            continue;
        } else {
            // Stop at first non-//! line
            break;
        }
    }

    if doc_lines.is_empty() {
        None
    } else {
        Some(doc_lines.join("\n"))
    }
}

// TODO: Implement parse_file function that reads from PathBuf and combines all extractors
// TODO: Add integration tests with test fixtures in test_fixtures/

#[cfg(test)]
mod tests {
    use super::*;

    // Tests for extract_imports
    #[test]
    fn test_extract_imports_single_import() {
        let source = r#"const std = @import("std");"#;
        let imports = extract_imports(source);

        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].target, "std");
        assert_eq!(imports[0].line, 1);
        assert!(imports[0].is_std);
    }

    #[test]
    fn test_extract_imports_multiple_imports() {
        let source = r#"
const std = @import("std");
const scheduler = @import("scheduler.zig");
const memory = @import("kernel/memory.zig");
"#;
        let imports = extract_imports(source);

        assert_eq!(imports.len(), 3);

        assert_eq!(imports[0].target, "std");
        assert_eq!(imports[0].line, 2);
        assert!(imports[0].is_std);

        assert_eq!(imports[1].target, "scheduler.zig");
        assert_eq!(imports[1].line, 3);
        assert!(!imports[1].is_std);

        assert_eq!(imports[2].target, "kernel/memory.zig");
        assert_eq!(imports[2].line, 4);
        assert!(!imports[2].is_std);
    }

    #[test]
    fn test_extract_imports_multiline_source() {
        let source = r#"
// File header
const std = @import("std");

pub fn main() void {
    const lib = @import("lib.zig");
}
"#;
        let imports = extract_imports(source);

        assert_eq!(imports.len(), 2);
        assert_eq!(imports[0].target, "std");
        assert_eq!(imports[0].line, 3);
        assert_eq!(imports[1].target, "lib.zig");
        assert_eq!(imports[1].line, 6);
    }

    #[test]
    fn test_extract_imports_in_line_comment() {
        let source = r#"
const std = @import("std");
// const fake = @import("fake.zig");
const real = @import("real.zig");
"#;
        let imports = extract_imports(source);

        assert_eq!(imports.len(), 2);
        assert_eq!(imports[0].target, "std");
        assert_eq!(imports[1].target, "real.zig");

        // Should NOT find fake.zig
        assert!(!imports.iter().any(|i| i.target == "fake.zig"));
    }

    #[test]
    fn test_extract_imports_in_block_comment() {
        let source = r#"
const std = @import("std");
/*
 * const fake = @import("fake.zig");
 * More comments
 */
const real = @import("real.zig");
"#;
        let imports = extract_imports(source);

        assert_eq!(imports.len(), 2);
        assert_eq!(imports[0].target, "std");
        assert_eq!(imports[1].target, "real.zig");

        // Should NOT find fake.zig
        assert!(!imports.iter().any(|i| i.target == "fake.zig"));
    }

    #[test]
    fn test_extract_imports_in_string_literal() {
        let source = r#"
const std = @import("std");
const msg = "This is not an import: @import(\"fake.zig\")";
const real = @import("real.zig");
"#;
        let imports = extract_imports(source);

        assert_eq!(imports.len(), 2);
        assert_eq!(imports[0].target, "std");
        assert_eq!(imports[1].target, "real.zig");

        // Should NOT find fake.zig inside string
        assert!(!imports.iter().any(|i| i.target == "fake.zig"));
    }

    #[test]
    fn test_extract_imports_std_detection() {
        let source = r#"
const std = @import("std");
const builtin = @import("builtin");
const local = @import("local.zig");
"#;
        let imports = extract_imports(source);

        assert_eq!(imports.len(), 3);
        assert!(imports[0].is_std); // std
        assert!(!imports[1].is_std); // builtin (not prefixed with std.)
        assert!(!imports[2].is_std); // local.zig
    }

    #[test]
    fn test_extract_imports_nested_string_escapes() {
        let source = r#"
const std = @import("std");
const escaped = "String with \" and @import(\"fake.zig\")";
const real = @import("real.zig");
"#;
        let imports = extract_imports(source);

        assert_eq!(imports.len(), 2);
        assert_eq!(imports[0].target, "std");
        assert_eq!(imports[1].target, "real.zig");
    }

    #[test]
    fn test_extract_imports_empty_source() {
        let source = "";
        let imports = extract_imports(source);
        assert_eq!(imports.len(), 0);
    }

    #[test]
    fn test_extract_imports_no_imports() {
        let source = r#"
pub fn main() void {
    // No imports here
}
"#;
        let imports = extract_imports(source);
        assert_eq!(imports.len(), 0);
    }

    #[test]
    fn test_extract_imports_complex_mixed_content() {
        let source = r#"
const std = @import("std"); // Real import
// Fake: @import("comment.zig")
/* Block: @import("block.zig") */
const msg = "@import(\"string.zig\")";
const real2 = @import("real2.zig");
"#;
        let imports = extract_imports(source);

        assert_eq!(imports.len(), 2);
        assert_eq!(imports[0].target, "std");
        assert_eq!(imports[1].target, "real2.zig");
    }

    #[test]
    fn test_extract_imports_inline_comment_after_import() {
        let source = r#"const std = @import("std"); // standard library"#;
        let imports = extract_imports(source);

        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].target, "std");
    }

    // Tests for extract_public_decls (existing tests would go here)

    // Tests for extract_doc_comment
    #[test]
    fn test_extract_doc_comment_empty_file() {
        let source = "";
        assert_eq!(extract_doc_comment(source), None);
    }

    #[test]
    fn test_extract_doc_comment_no_doc_comment() {
        let source = r#"const std = @import("std");
pub fn main() void {}"#;
        assert_eq!(extract_doc_comment(source), None);
    }

    #[test]
    fn test_extract_doc_comment_single_line() {
        let source = "//! This is a single line doc comment.\nconst std = @import(\"std\");";
        assert_eq!(
            extract_doc_comment(source),
            Some("This is a single line doc comment.".to_string())
        );
    }

    #[test]
    fn test_extract_doc_comment_multiline() {
        let source = r#"//! This is the module documentation.
//! It can span multiple lines.

const std = @import("std");"#;
        assert_eq!(
            extract_doc_comment(source),
            Some("This is the module documentation.\nIt can span multiple lines.".to_string())
        );
    }

    #[test]
    fn test_extract_doc_comment_stops_at_blank_line() {
        let source = r#"//! First line.
//! Second line.

//! This should not be captured.
const std = @import("std");"#;
        assert_eq!(
            extract_doc_comment(source),
            Some("First line.\nSecond line.".to_string())
        );
    }

    #[test]
    fn test_extract_doc_comment_regular_comments_not_captured() {
        let source = r#"// Regular comment
// Another regular comment
const std = @import("std");"#;
        assert_eq!(extract_doc_comment(source), None);
    }

    #[test]
    fn test_extract_doc_comment_stops_at_regular_comment() {
        let source = r#"//! Module doc.
// Regular comment should stop the doc.
//! This should not be captured."#;
        assert_eq!(
            extract_doc_comment(source),
            Some("Module doc.".to_string())
        );
    }

    #[test]
    fn test_extract_doc_comment_leading_blank_lines() {
        let source = r#"

//! Doc comment after blank lines.
const std = @import("std");"#;
        assert_eq!(
            extract_doc_comment(source),
            Some("Doc comment after blank lines.".to_string())
        );
    }

    #[test]
    fn test_extract_doc_comment_no_space_after_prefix() {
        let source = "//!No space after prefix\n//! With space\nconst x = 1;";
        assert_eq!(
            extract_doc_comment(source),
            Some("No space after prefix\nWith space".to_string())
        );
    }

    #[test]
    fn test_extract_doc_comment_only_doc_comments() {
        let source = "//! Line 1\n//! Line 2\n//! Line 3";
        assert_eq!(
            extract_doc_comment(source),
            Some("Line 1\nLine 2\nLine 3".to_string())
        );
    }
}
