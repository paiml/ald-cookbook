# Property-Based Testing

## Overview

Property-based testing verifies code correctness by testing invariants across thousands of automatically generated inputs.

## Setup

Add to `Cargo.toml`:

```toml
[dev-dependencies]
proptest = "1.5"
```

## Basic Property Test

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn addition_is_commutative(a: i32, b: i32) {
        prop_assert_eq!(a + b, b + a);
    }
}
```

## Strategies for ALD

### Value Strategies

```rust
// Dataset name strategy
fn dataset_name() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{2,30}".prop_map(|s| s.to_string())
}

// Row count strategy
fn row_count() -> impl Strategy<Value = usize> {
    1usize..100_000
}

// Float values (avoiding NaN/Inf)
fn valid_float() -> impl Strategy<Value = f64> {
    prop::num::f64::NORMAL
}
```

### Composite Strategies

```rust
fn metadata_strategy() -> impl Strategy<Value = Metadata> {
    (
        dataset_name(),
        "[A-Za-z ]{0,200}",  // description
        row_count(),
    ).prop_map(|(name, desc, rows)| {
        Metadata::new(&name, &desc, rows)
    })
}
```

## Key Properties

### Roundtrip Properties

```rust
proptest! {
    #[test]
    fn save_load_roundtrip(metadata in metadata_strategy()) {
        let bytes = metadata.to_msgpack()?;
        let loaded = Metadata::from_msgpack(&bytes)?;
        prop_assert_eq!(metadata.name, loaded.name);
        prop_assert_eq!(metadata.row_count, loaded.row_count);
    }
}
```

### Invariant Properties

```rust
proptest! {
    #[test]
    fn checksum_detects_corruption(data in prop::collection::vec(any::<u8>(), 100..10000)) {
        let checksum = crc32fast::hash(&data);

        // Flip one bit
        let mut corrupted = data.clone();
        corrupted[0] ^= 1;

        let corrupted_checksum = crc32fast::hash(&corrupted);
        prop_assert_ne!(checksum, corrupted_checksum);
    }
}
```

### Monotonicity Properties

```rust
proptest! {
    #[test]
    fn filter_reduces_count(
        rows in 100..1000usize,
        threshold in 0.0..1.0f64
    ) {
        let batch = generate_batch(rows);
        let filtered = filter(&batch, threshold)?;
        prop_assert!(filtered.num_rows() <= batch.num_rows());
    }
}
```

## Configuration

### Fast Tests (Development)

```rust
proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn fast_property(x: i32) { ... }
}
```

### Thorough Tests (CI)

```rust
proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    #[test]
    fn thorough_property(x: i32) { ... }
}
```

## Running Property Tests

```bash
# All property tests
cargo test --test proptest_format
cargo test --test proptest_transforms
cargo test --test proptest_drift

# With specific seed for reproduction
PROPTEST_SEED=12345 cargo test
```
