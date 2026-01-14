---
name: test
description: Run Rust tests with optional filtering
---

# /test - Run Rust Tests

Run cargo test with smart defaults for quick verification.

## Usage

```
/test [filter] [--nocapture]
```

## Options

| Option | Values | Default | Description |
|--------|--------|---------|-------------|
| `filter` | test name pattern | (all) | Filter tests by name |
| `--nocapture` | flag | false | Show test output (println!, etc.) |

## Examples

- `/test` - Run all tests
- `/test parser` - Run tests matching "parser"
- `/test cycle --nocapture` - Run "cycle" tests with output
- `/test graph::` - Run tests in graph module

## What It Does

1. Executes: `cargo test [filter] [-- --nocapture]`
2. Reports pass/fail counts
3. Shows failure details if any tests fail
4. Returns exit code 0 on success, 1 on failure

## When to Use

- After making code changes
- Before committing
- Quick regression check
- Debugging test failures (with --nocapture)

## Implementation

```bash
# Default (all tests)
cargo test

# With filter
cargo test extract_imports

# With output capture disabled
cargo test -- --nocapture

# Specific module
cargo test parser::zig::
```
