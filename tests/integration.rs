use std::path::PathBuf;

/// Helper function to get path to test fixtures directory.
///
/// Returns the absolute path to the test_fixtures directory
/// at the project root.
fn fixtures_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_fixtures")
}

#[test]
fn test_fixtures_exist() {
    let path = fixtures_path().join("simple");
    assert!(path.exists(), "test_fixtures/simple should exist");
    assert!(path.join("main.zig").exists());
    assert!(path.join("lib.zig").exists());
}

#[test]
fn test_cli_json_output() {
    use std::process::Command;
    use std::fs;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join(".dendrite");
    let fixtures = fixtures_path().join("simple");

    let output = Command::new(env!("CARGO_BIN_EXE_dendrite"))
        .args([
            "--path",
            fixtures.to_str().unwrap(),
            "--json",
            "--output",
            output_dir.to_str().unwrap(),
            "--quiet",
        ])
        .output()
        .expect("Failed to execute dendrite");

    assert!(
        output.status.success(),
        "dendrite should exit successfully. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let json_path = output_dir.join("graph.json");
    assert!(json_path.exists(), "graph.json should be created");

    let json_content = fs::read_to_string(&json_path).expect("Failed to read graph.json");
    let parsed: serde_json::Value =
        serde_json::from_str(&json_content).expect("JSON should be valid");

    assert!(parsed.get("nodes").is_some(), "JSON should have 'nodes'");
    assert!(parsed.get("edges").is_some(), "JSON should have 'edges'");
    assert!(parsed.get("metadata").is_some(), "JSON should have 'metadata'");

    let nodes = parsed["nodes"].as_array().expect("nodes should be array");
    assert!(nodes.len() >= 2, "Should have at least 2 nodes (main.zig, lib.zig)");

    let metadata = &parsed["metadata"];
    assert!(metadata["node_count"].is_number(), "metadata should have node_count");
    assert!(metadata["edge_count"].is_number(), "metadata should have edge_count");
}

#[test]
fn test_parse_simple_fixture() {
    use dendrite::discovery::{discover_files, DiscoveryConfig};
    use dendrite::graph::{FileNode, GraphBuilder, Layer};
    use dendrite::parser::parse_file;
    use std::collections::HashMap;

    let fixtures = fixtures_path().join("simple");
    let config = DiscoveryConfig::default();

    let files = discover_files(&fixtures, &config).expect("Failed to discover files");

    assert!(files.len() >= 2, "Should find at least 2 Zig files");

    let mut zig_files = Vec::new();
    for file in &files {
        if file.extension().and_then(|e| e.to_str()) == Some("zig") {
            let parsed = parse_file(file).expect("Failed to parse Zig file");
            zig_files.push(parsed);
        }
    }

    assert_eq!(zig_files.len(), 2, "Should parse exactly 2 Zig files");

    let main_file = zig_files
        .iter()
        .find(|f| f.path.file_name().unwrap() == "main.zig")
        .expect("Should find main.zig");
    let lib_file = zig_files
        .iter()
        .find(|f| f.path.file_name().unwrap() == "lib.zig")
        .expect("Should find lib.zig");

    assert!(
        !main_file.imports.is_empty(),
        "main.zig should have imports"
    );
    assert!(
        !lib_file.exports.is_empty(),
        "lib.zig should have exports"
    );

    let mut builder = GraphBuilder::new(fixtures.clone(), false);
    let mut import_map = HashMap::new();

    for zig_file in &zig_files {
        let relative_path = zig_file
            .path
            .strip_prefix(&fixtures)
            .unwrap_or(&zig_file.path)
            .to_string_lossy()
            .to_string();

        let node = FileNode {
            path: zig_file.path.clone(),
            relative_path: relative_path.clone(),
            layer: Layer::from_path(&relative_path),
            depth: 0,
            summary: zig_file.doc_comment.clone(),
            exports: zig_file.exports.clone(),
            loc: zig_file.loc,
        };

        builder.add_file(node);

        let imports: Vec<(String, usize)> = zig_file
            .imports
            .iter()
            .filter(|i| !i.is_std)
            .map(|i| (i.target.clone(), i.line))
            .collect();

        if !imports.is_empty() {
            import_map.insert(zig_file.path.clone(), imports);
        }
    }

    builder.add_edges(&import_map);
    let graph = builder.build();

    assert_eq!(graph.node_count(), 2, "Graph should have 2 nodes");
    assert!(graph.edge_count() >= 1, "Graph should have at least 1 edge");
}

// Task 2.7.5: CI pass case - clean codebase exits 0
#[test]
fn test_ci_pass_case() {
    use std::process::Command;

    let fixtures = fixtures_path().join("simple");

    let output = Command::new(env!("CARGO_BIN_EXE_dendrite"))
        .args([
            "--path",
            fixtures.to_str().unwrap(),
            "--check",
            "--quiet",
        ])
        .output()
        .expect("Failed to execute dendrite");

    assert!(
        output.status.success(),
        "Clean codebase should exit 0 with --check. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

// Task 2.7.6: CI fail case - cycle present exits 1
#[test]
fn test_ci_fail_case_with_cycle() {
    use std::process::Command;

    let fixtures = fixtures_path().join("with_cycle");

    // Make sure the test fixture exists
    assert!(fixtures.exists(), "with_cycle fixture should exist");
    assert!(fixtures.join("a.zig").exists(), "a.zig should exist");
    assert!(fixtures.join("b.zig").exists(), "b.zig should exist");

    let output = Command::new(env!("CARGO_BIN_EXE_dendrite"))
        .args([
            "--path",
            fixtures.to_str().unwrap(),
            "--check",
            "--quiet",
        ])
        .output()
        .expect("Failed to execute dendrite");

    assert!(
        !output.status.success(),
        "Codebase with cycle should exit 1 with --check. exit code: {:?}",
        output.status.code()
    );

    // The exit code should be 1
    assert_eq!(
        output.status.code(),
        Some(1),
        "Exit code should be 1 for cycle detection"
    );
}

// Task 2.7.4: --check --json combo
#[test]
fn test_ci_check_json_combo() {
    use std::process::Command;

    let fixtures = fixtures_path().join("simple");

    let output = Command::new(env!("CARGO_BIN_EXE_dendrite"))
        .args([
            "--path",
            fixtures.to_str().unwrap(),
            "--check",
            "--json",
            "--quiet",
        ])
        .output()
        .expect("Failed to execute dendrite");

    assert!(output.status.success(), "Clean codebase should pass");

    // The stdout should contain valid JSON with analysis results
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout)
        .expect("--check --json should output valid JSON");

    assert!(parsed.get("has_errors").is_some(), "Should have has_errors field");
    assert!(parsed.get("cycles").is_some(), "Should have cycles field");
    assert!(parsed.get("violations").is_some(), "Should have violations field");
    assert_eq!(parsed["has_errors"], false, "Clean codebase should have no errors");
}
