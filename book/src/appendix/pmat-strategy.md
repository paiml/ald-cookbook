# PMAT Testing Strategy

**PMAT** (Property-based testing, Mutation testing, and Adversarial Testing) is our comprehensive testing methodology.

## Overview

```
┌─────────────────────────────────────────────────┐
│              PMAT Testing Strategy               │
├─────────────────────────────────────────────────┤
│  P: Property-Based Testing                       │
│     └── Verify invariants across random inputs   │
├─────────────────────────────────────────────────┤
│  M: Mutation Testing                             │
│     └── Verify tests catch code changes          │
├─────────────────────────────────────────────────┤
│  A: Adversarial Testing                          │
│     └── Test edge cases and malicious input      │
├─────────────────────────────────────────────────┤
│  T: Traditional Testing                          │
│     └── Unit tests, integration tests, E2E       │
└─────────────────────────────────────────────────┘
```

## P: Property-Based Testing

### Purpose
Discover edge cases automatically by testing invariants across thousands of generated inputs.

### Implementation

```rust
proptest! {
    // Roundtrip property
    #[test]
    fn save_load_roundtrip(rows in 1..10000usize) {
        let batch = generate_batch(rows);
        save(&batch, &path, options)?;
        let (loaded, _) = load(&path)?;
        prop_assert_eq!(batch.num_rows(), loaded.num_rows());
    }

    // Monotonicity property
    #[test]
    fn filter_reduces_count(rows in 100..1000usize) {
        let batch = generate_batch(rows);
        let filtered = filter(&batch, predicate)?;
        prop_assert!(filtered.num_rows() <= batch.num_rows());
    }

    // Idempotency property
    #[test]
    fn normalize_idempotent(values in vec(1.0..100.0f64, 2..100)) {
        let normalized = normalize(&values);
        let double = normalize(&normalized);
        prop_assert_eq!(normalized, double);
    }
}
```

### Metrics
- Minimum 3 properties per module
- 50 cases for fast tests
- 500 cases for comprehensive tests

## M: Mutation Testing

### Purpose
Verify that tests actually catch bugs by introducing mutations and checking if tests fail.

### Implementation

```bash
# Run mutation testing
cargo mutants --timeout 60

# Results
Mutation testing:
  Total: 150
  Caught: 127 (85%)
  Missed: 18 (12%)
  Timeout: 5 (3%)
```

### Mutation Types

| Type | Example | Test Requirement |
|------|---------|------------------|
| Arithmetic | `+` → `-` | Test with known values |
| Comparison | `>` → `>=` | Test boundaries |
| Logical | `&&` → `||` | Test both branches |
| Return | `return x` → `return 0` | Assert return values |

### Metrics
- Minimum 80% mutation score
- All critical paths covered

## A: Adversarial Testing

### Purpose
Test handling of malicious, malformed, or unexpected inputs.

### Implementation

```rust
#[test]
fn test_corrupted_magic_bytes() {
    let mut bytes = valid_ald_bytes();
    bytes[0..4].copy_from_slice(b"XXXX");  // Corrupt magic

    let result = load_from_bytes(&bytes);
    assert!(matches!(result, Err(Error::InvalidFormat(_))));
}

#[test]
fn test_truncated_file() {
    let bytes = valid_ald_bytes();
    let truncated = &bytes[..bytes.len() / 2];

    let result = load_from_bytes(truncated);
    assert!(result.is_err());
}

#[test]
fn test_oversized_metadata() {
    let mut header = valid_header();
    header.metadata_len = u32::MAX;  // Impossibly large

    let result = parse_header(&header.to_bytes());
    assert!(result.is_err());
}
```

### Test Categories

1. **Malformed Input**: Corrupted headers, invalid checksums
2. **Boundary Values**: Empty files, maximum sizes
3. **Type Confusion**: Wrong data types, schema mismatches
4. **Resource Exhaustion**: Large allocations, deep recursion

## T: Traditional Testing

### Unit Tests

```rust
#[test]
fn test_header_magic_bytes() {
    let header = Header::new("test", 100);
    assert_eq!(&header.magic, b"ALDF");
}
```

### Integration Tests

```rust
#[test]
fn test_full_pipeline() {
    // Create → Save → Load → Transform → Save
    let batch = create_batch();
    save(&batch, "test.ald", options)?;
    let (loaded, _) = load("test.ald")?;
    let filtered = filter(&loaded, predicate)?;
    save(&filtered, "output.ald", options)?;
}
```

### End-to-End Tests

```bash
# Verify full recipe execution
cargo run --example create_ald_from_arrow
# Verify idempotency
cargo run --example create_ald_from_arrow > run1.txt
cargo run --example create_ald_from_arrow > run2.txt
diff run1.txt run2.txt
```

## Quality Matrix

| Strategy | Coverage | Purpose |
|----------|----------|---------|
| Property | Invariants | Find edge cases |
| Mutation | Test quality | Verify test effectiveness |
| Adversarial | Security | Handle malicious input |
| Traditional | Functionality | Verify behavior |

## CI Integration

```yaml
jobs:
  pmat:
    steps:
      # Traditional tests
      - run: cargo test --all-features

      # Property tests
      - run: cargo test --test proptest_format
      - run: cargo test --test proptest_transforms

      # Mutation testing (on schedule)
      - run: cargo mutants --timeout 120

      # Adversarial tests
      - run: cargo test adversarial::
```

## Metrics Summary

| Metric | Target | Enforcement |
|--------|--------|-------------|
| Line Coverage | 95% | CI gate |
| Mutation Score | 80% | Weekly check |
| Property Cases | 50+ | Per test file |
| Adversarial Tests | 10+ | Per module |
