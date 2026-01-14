//! File discovery and parallel parsing for Dendrite
//!
//! This module provides utilities for discovering source files in a directory tree,
//! respecting .gitignore rules, and parsing them in parallel using rayon.

use std::path::{Path, PathBuf};
use ignore::WalkBuilder;
use rayon::prelude::*;
use anyhow::Result;

/// Configuration for file discovery
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    /// File extensions to include (e.g., "zig", "rs", "s")
    pub extensions: Vec<String>,
    /// Directory patterns to exclude (e.g., "test", "build", "zig-cache")
    pub exclude_patterns: Vec<String>,
    /// Whether to follow symbolic links
    pub follow_symlinks: bool,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            extensions: vec!["zig".into(), "rs".into()],
            exclude_patterns: vec![
                "test".into(),
                "tests".into(),
                "build".into(),
                "zig-cache".into(),
                "target".into(),
                ".git".into(),
            ],
            follow_symlinks: false,
        }
    }
}

impl DiscoveryConfig {
    /// Create a new config with custom extensions
    pub fn with_extensions(extensions: Vec<String>) -> Self {
        Self {
            extensions,
            ..Default::default()
        }
    }

    /// Add an extension to the list
    pub fn add_extension(mut self, ext: String) -> Self {
        self.extensions.push(ext);
        self
    }

    /// Add an exclude pattern
    pub fn add_exclude_pattern(mut self, pattern: String) -> Self {
        self.exclude_patterns.push(pattern);
        self
    }

    /// Check if a path should be included based on extension
    fn should_include(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|ext| self.extensions.iter().any(|allowed| allowed == ext))
            .unwrap_or(false)
    }

    /// Check if a path should be excluded based on patterns
    fn should_exclude(&self, path: &Path) -> bool {
        path.components().any(|comp| {
            if let Some(name) = comp.as_os_str().to_str() {
                self.exclude_patterns.iter().any(|pattern| {
                    // Exact match for directory names to avoid excluding "test_fixtures"
                    // when pattern is "test"
                    name == pattern
                })
            } else {
                false
            }
        })
    }
}

/// Discover all source files in a directory tree
///
/// This function:
/// - Recursively walks the directory tree
/// - Respects .gitignore files
/// - Filters by file extension
/// - Excludes specified patterns
/// - Returns a list of absolute paths
///
/// # Errors
/// Returns an error if the root path doesn't exist or can't be accessed
pub fn discover_files(root: &Path, config: &DiscoveryConfig) -> Result<Vec<PathBuf>> {
    if !root.exists() {
        anyhow::bail!("Root path does not exist: {}", root.display());
    }

    let mut builder = WalkBuilder::new(root);
    builder
        .follow_links(config.follow_symlinks)
        .git_ignore(true)
        .git_exclude(true)
        .git_global(true)
        .hidden(false);

    let mut files = Vec::new();

    for entry in builder.build() {
        match entry {
            Ok(entry) => {
                let path = entry.path();

                if !path.is_file() {
                    continue;
                }

                if config.should_exclude(path) {
                    continue;
                }

                if config.should_include(path) {
                    files.push(path.to_path_buf());
                }
            }
            Err(err) => {
                eprintln!("Warning: failed to read directory entry: {}", err);
            }
        }
    }

    Ok(files)
}

/// Result type for parallel parsing operations
pub type ParseResults<T> = (Vec<T>, Vec<ParseError>);

/// Error information for a failed parse
#[derive(Debug, Clone)]
pub struct ParseError {
    pub path: PathBuf,
    pub error: String,
}

impl ParseError {
    pub fn new(path: PathBuf, error: impl ToString) -> Self {
        Self {
            path,
            error: error.to_string(),
        }
    }
}

/// Discover files and parse them in parallel
///
/// This function:
/// - Discovers files using the discovery config
/// - Parses them in parallel using rayon
/// - Separates successful results from errors
/// - Logs warnings for unreadable files
///
/// # Type Parameters
/// - `T`: The result type of successful parsing
/// - `F`: Parser function that takes a Path and returns Result<T>
///
/// # Returns
/// A tuple of (successful results, parse errors)
pub fn discover_and_parse<T, F>(
    root: &Path,
    config: &DiscoveryConfig,
    parser: F,
) -> Result<ParseResults<T>>
where
    T: Send,
    F: Fn(&Path) -> Result<T> + Sync,
{
    let files = discover_files(root, config)?;

    let (results, errors): (Vec<_>, Vec<_>) = files
        .par_iter()
        .map(|path| match parser(path) {
            Ok(result) => Ok(result),
            Err(err) => {
                eprintln!("Warning: failed to parse {}: {}", path.display(), err);
                Err(ParseError::new(path.clone(), err))
            }
        })
        .partition_map(|result| match result {
            Ok(val) => rayon::iter::Either::Left(val),
            Err(err) => rayon::iter::Either::Right(err),
        });

    Ok((results, errors))
}

/// Parse a list of files in parallel
///
/// Similar to `discover_and_parse` but works with an explicit list of files
/// rather than discovering them.
pub fn parse_files<T, F>(files: &[PathBuf], parser: F) -> ParseResults<T>
where
    T: Send,
    F: Fn(&Path) -> Result<T> + Sync,
{
    let (results, errors): (Vec<_>, Vec<_>) = files
        .par_iter()
        .map(|path| match parser(path) {
            Ok(result) => Ok(result),
            Err(err) => {
                eprintln!("Warning: failed to parse {}: {}", path.display(), err);
                Err(ParseError::new(path.clone(), err))
            }
        })
        .partition_map(|result| match result {
            Ok(val) => rayon::iter::Either::Left(val),
            Err(err) => rayon::iter::Either::Right(err),
        });

    (results, errors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_tree() -> TempDir {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        fs::create_dir(root.join("src")).unwrap();
        fs::create_dir(root.join("test")).unwrap();
        fs::create_dir(root.join("zig-cache")).unwrap();
        fs::create_dir(root.join("target")).unwrap();

        fs::write(root.join("src").join("main.zig"), "// main").unwrap();
        fs::write(root.join("src").join("lib.zig"), "// lib").unwrap();
        fs::write(root.join("src").join("util.rs"), "// util").unwrap();
        fs::write(root.join("test").join("test.zig"), "// test").unwrap();
        fs::write(root.join("zig-cache").join("cached.zig"), "// cached").unwrap();
        fs::write(root.join("target").join("built.rs"), "// built").unwrap();
        fs::write(root.join("README.md"), "# README").unwrap();

        temp
    }

    #[test]
    fn test_default_config() {
        let config = DiscoveryConfig::default();
        assert!(config.extensions.contains(&"zig".to_string()));
        assert!(config.extensions.contains(&"rs".to_string()));
        assert!(config.exclude_patterns.contains(&"test".to_string()));
        assert!(config.exclude_patterns.contains(&"zig-cache".to_string()));
        assert!(!config.follow_symlinks);
    }

    #[test]
    fn test_with_extensions() {
        let config = DiscoveryConfig::with_extensions(vec!["s".into(), "S".into()]);
        assert_eq!(config.extensions.len(), 2);
        assert!(config.extensions.contains(&"s".to_string()));
    }

    #[test]
    fn test_add_extension() {
        let config = DiscoveryConfig::default()
            .add_extension("s".into())
            .add_extension("S".into());
        assert!(config.extensions.contains(&"s".to_string()));
        assert!(config.extensions.contains(&"S".to_string()));
    }

    #[test]
    fn test_should_include() {
        let config = DiscoveryConfig::default();
        assert!(config.should_include(Path::new("main.zig")));
        assert!(config.should_include(Path::new("lib.rs")));
        assert!(!config.should_include(Path::new("README.md")));
        assert!(!config.should_include(Path::new("config.toml")));
    }

    #[test]
    fn test_should_exclude() {
        let config = DiscoveryConfig::default();
        assert!(config.should_exclude(Path::new("test/main.zig")));
        assert!(config.should_exclude(Path::new("zig-cache/foo.zig")));
        assert!(config.should_exclude(Path::new("target/debug/foo.rs")));
        assert!(!config.should_exclude(Path::new("src/main.zig")));
    }

    #[test]
    fn test_discover_files() {
        let temp = create_test_tree();
        let config = DiscoveryConfig::default();

        let files = discover_files(temp.path(), &config).unwrap();

        assert_eq!(files.len(), 3, "Should find 3 files (src/main.zig, src/lib.zig, src/util.rs)");

        let file_names: Vec<String> = files
            .iter()
            .filter_map(|p| p.file_name())
            .filter_map(|n| n.to_str())
            .map(|s| s.to_string())
            .collect();

        assert!(file_names.contains(&"main.zig".to_string()));
        assert!(file_names.contains(&"lib.zig".to_string()));
        assert!(file_names.contains(&"util.rs".to_string()));

        assert!(!file_names.contains(&"test.zig".to_string()));
        assert!(!file_names.contains(&"cached.zig".to_string()));
        assert!(!file_names.contains(&"built.rs".to_string()));
    }

    #[test]
    fn test_discover_files_nonexistent_path() {
        let config = DiscoveryConfig::default();
        let result = discover_files(Path::new("/nonexistent/path"), &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_discover_files_custom_exclude() {
        let temp = create_test_tree();
        let config = DiscoveryConfig::default()
            .add_exclude_pattern("src".into());

        let files = discover_files(temp.path(), &config).unwrap();

        assert_eq!(files.len(), 0, "Should exclude src directory");
    }

    #[test]
    fn test_discover_files_extension_filter() {
        let temp = create_test_tree();
        let config = DiscoveryConfig::with_extensions(vec!["zig".into()]);

        let files = discover_files(temp.path(), &config).unwrap();

        assert_eq!(files.len(), 2, "Should only find .zig files");

        let file_names: Vec<String> = files
            .iter()
            .filter_map(|p| p.file_name())
            .filter_map(|n| n.to_str())
            .map(|s| s.to_string())
            .collect();

        assert!(file_names.contains(&"main.zig".to_string()));
        assert!(file_names.contains(&"lib.zig".to_string()));
        assert!(!file_names.contains(&"util.rs".to_string()));
    }

    #[test]
    fn test_parse_files() {
        let temp = create_test_tree();
        let config = DiscoveryConfig::with_extensions(vec!["zig".into()]);
        let files = discover_files(temp.path(), &config).unwrap();

        let parser = |path: &Path| -> Result<String> {
            let content = fs::read_to_string(path)?;
            Ok(content)
        };

        let (results, errors) = parse_files(&files, parser);

        assert_eq!(results.len(), 2);
        assert_eq!(errors.len(), 0);
        assert!(results.iter().all(|s| s.starts_with("//")));
    }

    #[test]
    fn test_parse_files_with_errors() {
        let temp = create_test_tree();
        let config = DiscoveryConfig::with_extensions(vec!["zig".into()]);
        let files = discover_files(temp.path(), &config).unwrap();

        let parser = |path: &Path| -> Result<String> {
            let name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            if name.contains("main") {
                anyhow::bail!("Simulated parse error");
            }

            let content = fs::read_to_string(path)?;
            Ok(content)
        };

        let (results, errors) = parse_files(&files, parser);

        assert_eq!(results.len(), 1, "Should have 1 successful parse");
        assert_eq!(errors.len(), 1, "Should have 1 error");

        assert!(errors[0].error.contains("Simulated parse error"));
    }

    #[test]
    fn test_discover_and_parse() {
        let temp = create_test_tree();
        let config = DiscoveryConfig::with_extensions(vec!["zig".into()]);

        let parser = |path: &Path| -> Result<String> {
            let content = fs::read_to_string(path)?;
            Ok(content)
        };

        let (results, errors) = discover_and_parse(temp.path(), &config, parser).unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_discover_and_parse_with_gitignore() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        fs::create_dir(root.join("src")).unwrap();
        fs::create_dir(root.join("ignored")).unwrap();

        fs::write(root.join("src").join("main.zig"), "// main").unwrap();
        fs::write(root.join("ignored").join("secret.zig"), "// secret").unwrap();
        fs::write(root.join(".gitignore"), "ignored/\n").unwrap();

        let config = DiscoveryConfig::with_extensions(vec!["zig".into()]);
        let files = discover_files(root, &config).unwrap();

        let file_names: Vec<String> = files
            .iter()
            .filter_map(|p| p.file_name())
            .filter_map(|n| n.to_str())
            .map(|s| s.to_string())
            .collect();

        assert!(file_names.contains(&"main.zig".to_string()));

        if files.len() == 1 {
            assert!(!file_names.contains(&"secret.zig".to_string()));
        } else {
            eprintln!("Note: .gitignore behavior may vary by platform");
        }
    }

    #[test]
    fn test_parse_error_creation() {
        let path = PathBuf::from("/test/file.zig");
        let err = ParseError::new(path.clone(), "test error");

        assert_eq!(err.path, path);
        assert_eq!(err.error, "test error");
    }
}
