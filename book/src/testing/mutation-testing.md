# Mutation Testing

## Overview

Mutation testing verifies test effectiveness by introducing small changes (mutations) to the code and checking if tests catch them.

## Setup

```bash
cargo install cargo-mutants
```

## Running Mutation Tests

```bash
# Run mutation testing
cargo mutants

# With timeout (for faster feedback)
cargo mutants --timeout 60

# On specific module
cargo mutants -- --package ald-cookbook --lib
```

## Understanding Results

```
Mutation testing:
  Total: 150
  Caught: 127 (85%)
  Missed: 18 (12%)
  Timeout: 5 (3%)
```

### Caught Mutations

The test suite detected and failed on these mutations:

```rust
// Original
fn add(a: i32, b: i32) -> i32 { a + b }

// Mutation (+ → -)
fn add(a: i32, b: i32) -> i32 { a - b }
// ✓ Test caught this mutation
```

### Missed Mutations

Tests didn't catch these mutations - indicates weak tests:

```rust
// Original
fn validate(x: i32) -> bool { x > 0 && x < 100 }

// Mutation (> → >=)
fn validate(x: i32) -> bool { x >= 0 && x < 100 }
// ✗ No test covers x = 0 boundary
```

## Quality Threshold

**Minimum mutation score: 80%**

| Score | Interpretation |
|-------|----------------|
| 90%+ | Excellent test suite |
| 80-90% | Good, meets threshold |
| 70-80% | Needs improvement |
| <70% | Weak test suite |

## Improving Mutation Score

### 1. Add Boundary Tests

```rust
#[test]
fn test_boundary_values() {
    assert!(!validate(0));   // Catches >= mutation
    assert!(validate(1));
    assert!(validate(99));
    assert!(!validate(100)); // Catches < mutation
}
```

### 2. Test Error Paths

```rust
#[test]
fn test_error_handling() {
    let result = load("nonexistent.ald");
    assert!(result.is_err());
}
```

### 3. Add Property Tests

```rust
proptest! {
    #[test]
    fn filter_preserves_valid_rows(rows in 1..1000usize) {
        // Property ensures all mutations to filter logic are caught
    }
}
```

## CI Integration

```yaml
mutation-testing:
  runs-on: ubuntu-latest
  steps:
    - run: cargo install cargo-mutants
    - run: cargo mutants --timeout 120
```

## Mutation Types

| Type | Example | Catches |
|------|---------|---------|
| Arithmetic | `+` → `-` | Calculation errors |
| Comparison | `>` → `>=` | Boundary errors |
| Logical | `&&` → `\|\|` | Logic errors |
| Return | `return x` → `return 0` | Missing assertions |
| Negation | `!x` → `x` | Boolean inversions |
