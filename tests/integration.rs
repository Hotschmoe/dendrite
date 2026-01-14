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
#[ignore]
fn test_cli_json_output() {
    // TODO: Run dendrite --path ./test_fixtures/simple --json
    // Verify it produces valid JSON output
    //
    // Expected flow:
    // 1. Build dendrite binary
    // 2. Execute: dendrite --path ./test_fixtures/simple --json
    // 3. Capture stdout
    // 4. Parse as JSON
    // 5. Verify structure contains:
    //    - "files" array
    //    - "dependencies" array
    //    - Valid paths
}

#[test]
#[ignore]
fn test_parse_simple_fixture() {
    // TODO: Parse test_fixtures/simple and verify graph structure
    //
    // Expected flow:
    // 1. Call parser API on test_fixtures/simple directory
    // 2. Verify ZigFile results:
    //    - main.zig found
    //    - lib.zig found
    //    - Imports correctly extracted
    // 3. Verify graph structure:
    //    - 2 nodes (main, lib)
    //    - 1 edge (main -> lib)
    //    - No cycles
}
