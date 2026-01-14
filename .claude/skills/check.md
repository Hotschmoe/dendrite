---
name: check
description: Run cargo check and clippy on all targets
---

# /check - Validate Rust Code

Run fast compile checks and lints without full build.

## Usage

```
/check [--fix] [--pedantic]
```

## Options

| Option | Description |
|--------|-------------|
| `--fix` | Auto-fix clippy warnings where possible |
| `--pedantic` | Enable additional pedantic lints |

## Examples

- `/check` - Standard check + clippy
- `/check --fix` - Auto-fix warnings
- `/check --pedantic` - Strict linting mode

## What It Does

1. Runs `cargo check --all-targets` for fast compile verification
2. Runs `cargo clippy --all-targets -- -D warnings`
3. Reports any errors or warnings
4. Optionally auto-fixes issues

## When to Use

- Quick feedback during development (faster than full build)
- Before committing changes
- To catch common mistakes and non-idiomatic code
- CI validation

## Implementation

```bash
# Standard check
cargo check --all-targets
cargo clippy --all-targets -- -D warnings

# With auto-fix
cargo clippy --all-targets --fix --allow-dirty -- -D warnings

# Pedantic mode
cargo clippy --all-targets -- -D warnings -W clippy::pedantic
```

## Common Clippy Categories

| Category | What it catches |
|----------|-----------------|
| `correctness` | Likely bugs (enabled by default) |
| `suspicious` | Code that looks wrong |
| `style` | Non-idiomatic patterns |
| `complexity` | Unnecessarily complex code |
| `perf` | Performance anti-patterns |
| `pedantic` | Very strict (optional) |
