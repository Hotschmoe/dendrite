---
name: verify
description: Full build verification before merge/release
agent: build-verifier
---

# /verify - Full Build Verification

Comprehensive build validation for all targets before merge or release.

## Usage

```
/verify
```

## What It Does

1. **Format check**: `cargo fmt --check`
2. **Clippy**: `cargo clippy --all-targets -- -D warnings`
3. **Debug build**: `cargo build --all-targets`
4. **Release build**: `cargo build --release --all-targets`
5. **Run tests**: `cargo test --all-targets`

## Output

```
VERIFICATION REPORT
===================

[PASS] Format check
[PASS] Clippy (0 warnings)
[PASS] Debug build (12.3s)
[PASS] Release build (45.2s)
[PASS] Tests (36 passed, 0 failed)

RESULT: ALL CHECKS PASS - Ready to merge
```

## When to Use

- Before merging PRs
- After major refactoring
- Before releases
- After updating dependencies

## Failure Handling

If any check fails:
1. The specific error is reported
2. Verification continues to find all issues
3. Final summary shows all failures
4. Exit code is non-zero

## Implementation

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo build --all-targets
cargo build --release --all-targets
cargo test --all-targets
```

## Exit Codes

- 0: All checks pass
- 1: Build, test, or lint failure
