# Property-Based Testing

Property-based testing verifies that code satisfies invariants across thousands of randomly generated inputs.

## Why Property-Based Testing?

Traditional unit tests check specific examples:

```rust
#[test]
fn test_roundtrip() {
    let data = vec![1, 2, 3];
    assert_eq!(decode(encode(&data)), data);
}
```

Property tests check invariants across all possible inputs:

```rust
proptest! {
    #[test]
    fn roundtrip_preserves_data(data: Vec<i32>) {
        assert_eq!(decode(encode(&data)), data);
    }
}
```

## Core Properties

### Roundtrip Properties

```rust
proptest! {
    #[test]
    fn save_load_roundtrip(
        values in prop::collection::vec(any::<f64>(), 1..1000)
    ) {
        let batch = create_batch(&values);
        let path = temp_path();

        save(&batch, &path, SaveOptions::default())?;
        let (loaded, _) = load(&path)?;

        assert_eq!(batch.num_rows(), loaded.num_rows());
    }
}
```

### Invariant Properties

```rust
proptest! {
    #[test]
    fn filter_reduces_or_maintains_count(
        rows in 1..1000usize,
        threshold in 0.0..1.0f64
    ) {
        let batch = generate_batch(rows);
        let filtered = filter(&batch, |row| row.value > threshold)?;

        prop_assert!(filtered.num_rows() <= batch.num_rows());
    }
}
```

### Idempotency Properties

```rust
proptest! {
    #[test]
    fn normalize_is_idempotent(values in prop::collection::vec(1.0..100.0f64, 2..100)) {
        let normalized = normalize(&values);
        let double_normalized = normalize(&normalized);

        for (a, b) in normalized.iter().zip(double_normalized.iter()) {
            prop_assert!((a - b).abs() < 1e-10);
        }
    }
}
```

## Testing Strategies

### Value Strategies

```rust
use proptest::prelude::*;

// Simple values
let int_strategy = any::<i64>();
let float_strategy = -1e6..1e6f64;
let string_strategy = "[a-zA-Z0-9]{1,100}";

// Collections
let vec_strategy = prop::collection::vec(any::<i32>(), 0..1000);
let hashmap_strategy = prop::collection::hash_map(any::<String>(), any::<i32>(), 0..100);
```

### Custom Strategies

```rust
fn dataset_type_strategy() -> impl Strategy<Value = DatasetType> {
    prop_oneof![
        Just(DatasetType::Tabular),
        Just(DatasetType::TimeSeries),
        Just(DatasetType::Text),
        Just(DatasetType::Image),
    ]
}

fn metadata_strategy() -> impl Strategy<Value = Metadata> {
    (
        "[a-z]{3,20}",           // name
        "[a-zA-Z ]{0,100}",      // description
        1u64..1_000_000,         // row_count
        dataset_type_strategy(), // type
    ).prop_map(|(name, desc, rows, dtype)| {
        Metadata::new(name, desc, rows, dtype)
    })
}
```

### Composite Strategies

```rust
fn record_batch_strategy() -> impl Strategy<Value = RecordBatch> {
    (1usize..100, 1usize..10).prop_flat_map(|(rows, cols)| {
        prop::collection::vec(
            prop::collection::vec(any::<f64>(), rows),
            cols
        ).prop_map(move |data| create_batch_from_vecs(&data))
    })
}
```

## ALD-Specific Properties

### Format Properties

```rust
proptest! {
    // Header always 34 bytes
    #[test]
    fn header_size_constant(
        name in "[a-z]{1,50}",
        rows in 1..10000u64
    ) {
        let header = Header::new(&name, rows);
        let bytes = header.to_bytes();
        prop_assert_eq!(bytes.len(), 34);
    }

    // Magic bytes preserved
    #[test]
    fn magic_preserved(metadata in metadata_strategy()) {
        let header = Header::from_metadata(&metadata);
        prop_assert_eq!(&header.magic, b"ALDF");
    }
}
```

### Transform Properties

```rust
proptest! {
    // Shuffle preserves count
    #[test]
    fn shuffle_preserves_count(rows in 1..1000usize) {
        let batch = generate_batch(rows);
        let shuffled = shuffle(&batch, 42)?;
        prop_assert_eq!(batch.num_rows(), shuffled.num_rows());
    }

    // Sample respects ratio
    #[test]
    fn sample_respects_ratio(
        rows in 100..1000usize,
        ratio in 0.1..0.9f64
    ) {
        let batch = generate_batch(rows);
        let sampled = sample(&batch, ratio, 42)?;
        let expected = (rows as f64 * ratio) as usize;
        let tolerance = (expected as f64 * 0.1) as usize;

        prop_assert!((sampled.num_rows() as i64 - expected as i64).abs() <= tolerance as i64);
    }
}
```

### Drift Properties

```rust
proptest! {
    // Identical distributions have KS statistic near 0
    #[test]
    fn identical_distributions_low_ks(values in prop::collection::vec(0.0..1.0f64, 100..1000)) {
        let result = ks_test(&values, &values)?;
        prop_assert!(result.statistic < 0.1);
    }

    // PSI is non-negative
    #[test]
    fn psi_non_negative(
        baseline in prop::collection::vec(0.0..1.0f64, 100..1000),
        current in prop::collection::vec(0.0..1.0f64, 100..1000)
    ) {
        let result = calculate_psi(&baseline, &current, 10)?;
        prop_assert!(result >= 0.0);
    }
}
```

## Configuration

### Default Configuration

```rust
proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn fast_property_test(x: i32) {
        // Runs 50 cases
    }
}
```

### Comprehensive Configuration

```rust
proptest! {
    #![proptest_config(ProptestConfig {
        cases: 500,
        max_shrink_iters: 10000,
        ..ProptestConfig::default()
    })]

    #[test]
    fn thorough_property_test(x: i32) {
        // Runs 500 cases with extensive shrinking
    }
}
```

## Shrinking

When a property fails, proptest automatically finds the minimal failing case:

```
test proptest_format::roundtrip_preserves_data ... FAILED
  Input: values = [0.0, NaN, 1.0]
         ^^^ Minimal failing case found after shrinking
  Original input: values = [3.14159, -2.71828, NaN, 42.0, 0.0, 1.0, ...]
```

## Integration with CI

```yaml
test-property:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - run: cargo test --test proptest_format -- --nocapture
    - run: cargo test --test proptest_transforms -- --nocapture
    - run: cargo test --test proptest_drift -- --nocapture
```

## Test Organization

```
tests/
├── proptest_format.rs      # Format module properties
│   ├── header_invariants
│   ├── metadata_roundtrip
│   └── checksum_integrity
├── proptest_transforms.rs  # Transform module properties
│   ├── filter_properties
│   ├── shuffle_properties
│   └── sample_properties
└── proptest_drift.rs       # Drift module properties
    ├── ks_test_properties
    └── psi_properties
```

## Best Practices

1. **Name properties clearly**: `{operation}_{invariant}`
2. **Keep strategies focused**: One logical concept per strategy
3. **Use appropriate case counts**: 50 for fast, 500 for thorough
4. **Test edge cases explicitly**: Empty inputs, single elements
5. **Document expected failures**: Some properties should fail

## Next Steps

- [Mutation Testing](../testing/mutation-testing.md) - Test effectiveness
- [Coverage Requirements](../testing/coverage.md) - Coverage goals
