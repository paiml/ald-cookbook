# Recipe QA Checklist

Every recipe must pass this 10-point verification checklist before being considered complete.

## The Checklist

| # | Check | Description | Verification Command |
|---|-------|-------------|---------------------|
| 1 | **Exit Code 0** | Recipe runs successfully | `cargo run --example <name>; echo $?` |
| 2 | **Tests Pass** | All tests pass | `cargo test --example <name>` |
| 3 | **Deterministic** | Same output on repeated runs | Run twice, compare output |
| 4 | **No Temp Leaks** | No files left in temp directories | Check `/tmp` before/after |
| 5 | **Memory Stable** | No memory leaks | `valgrind` or watch RSS |
| 6 | **Platform Independent** | Works on Linux, macOS | CI matrix |
| 7 | **Clippy Clean** | Zero clippy warnings | `cargo clippy -- -D warnings` |
| 8 | **Rustfmt Standard** | Properly formatted | `cargo fmt --check` |
| 9 | **No unwrap()** | All errors handled | `grep -r "unwrap()" examples/<name>` |
| 10 | **Property Tests** | 50+ proptest cases pass | `cargo test --test proptest_*` |

## Detailed Verification

### 1. Exit Code 0

```bash
cargo run --example create_ald_from_arrow
echo "Exit code: $?"
# Expected: Exit code: 0
```

### 2. Tests Pass

```bash
cargo test --example create_ald_from_arrow
# Expected: test result: ok
```

### 3. Deterministic Output

```bash
cargo run --example create_ald_from_arrow > run1.txt 2>&1
cargo run --example create_ald_from_arrow > run2.txt 2>&1
diff run1.txt run2.txt
# Expected: No differences
```

### 4. No Temp File Leaks

```bash
# Count temp files before
BEFORE=$(ls /tmp | wc -l)

cargo run --example create_ald_from_arrow

# Count after
AFTER=$(ls /tmp | wc -l)

# Should be equal (or very close due to system activity)
echo "Before: $BEFORE, After: $AFTER"
```

### 5. Memory Stable

```bash
# Monitor RSS during execution
/usr/bin/time -v cargo run --example create_ald_from_arrow 2>&1 | grep "Maximum resident"
# Or use valgrind for leak detection
valgrind --leak-check=full cargo run --example create_ald_from_arrow
```

### 6. Platform Independent

Verified by CI matrix:

```yaml
strategy:
  matrix:
    os: [ubuntu-latest, macos-latest]
    rust: [1.75, stable]
```

### 7. Clippy Clean

```bash
cargo clippy --example create_ald_from_arrow -- -D warnings
# Expected: No warnings or errors
```

### 8. Rustfmt Standard

```bash
cargo fmt --check
# Expected: No formatting issues
```

### 9. No unwrap()

```bash
grep -r "\.unwrap()" examples/dataset_creation/create_ald_from_arrow.rs
# Expected: No matches (or only in test code)
```

### 10. Property Tests Pass

```bash
cargo test --test proptest_format -- --nocapture
cargo test --test proptest_transforms -- --nocapture
# Expected: All properties pass with 50+ cases
```

## Checklist Template

Copy this template for each recipe:

```markdown
## QA Checklist: [Recipe Name]

| # | Check | Status | Notes |
|---|-------|--------|-------|
| 1 | Exit Code 0 | [ ] | |
| 2 | Tests Pass | [ ] | |
| 3 | Deterministic | [ ] | |
| 4 | No Temp Leaks | [ ] | |
| 5 | Memory Stable | [ ] | |
| 6 | Platform Independent | [ ] | |
| 7 | Clippy Clean | [ ] | |
| 8 | Rustfmt Standard | [ ] | |
| 9 | No unwrap() | [ ] | |
| 10 | Property Tests | [ ] | |

**Verified by**: [Name]
**Date**: [Date]
**Version**: [Commit SHA]
```

## Automated Verification

The Makefile provides automated checking:

```bash
# Run all quality checks
make validate

# Quick checks during development
make quick-validate

# Full verification including mutation testing
make verify-all
```
