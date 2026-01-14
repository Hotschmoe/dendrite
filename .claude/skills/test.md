---
name: test
description: Run Rust tests with optional filtering
---

# /test - Run Rust Tests

Run cargo test with smart defaults for quick verification.

## Usage

```
/test [filter] [--package=<crate>] [--nocapture]
```

## Options

| Option | Values | Default | Description |
|--------|--------|---------|-------------|
| `filter` | test name pattern | (all) | Filter tests by name |
| `--package` | calc_core, calc_gui, calc_cli | (all) | Run tests for specific crate |
| `--nocapture` | flag | false | Show test output (println!, etc.) |

## Examples

- `/test` - Run all tests
- `/test beam` - Run tests matching "beam"
- `/test --package=calc_core` - Run only calc_core tests
- `/test moment --nocapture` - Run "moment" tests with output

## What It Does

1. Executes: `cargo test [filter] [-p package] [-- --nocapture]`
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
cargo test beam_moment

# Specific package
cargo test -p calc_core

# With output capture disabled
cargo test -- --nocapture
```
