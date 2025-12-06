# Null Detection

**Category**: E (Data Quality)
**Status**: Verified
**Isolation**: Full
**Idempotency**: Guaranteed

## Overview

Detect and report null values in datasets. Essential for data quality validation before ML training.

## Run the Recipe

```bash
cargo run --example quality_null_detection
```

## Code

```rust
use ald_cookbook::prelude::*;
use ald_cookbook::quality::null_count;
use ald_cookbook::{RecipeContext, Result};

fn main() -> Result<()> {
    let mut ctx = RecipeContext::new("quality_null_detection")?;

    let batch = create_batch_with_nulls(&mut ctx)?;

    // Count nulls per column
    let null_counts = null_count(&batch);

    let total_nulls: usize = null_counts.values().sum();
    let total_cells = batch.num_rows() * batch.num_columns();
    let null_percentage = (total_nulls as f64 / total_cells as f64) * 100.0;

    ctx.report(&format!(
        "Null Analysis:\n  Total cells: {}\n  Null cells: {}\n  Null percentage: {:.2}%",
        total_cells, total_nulls, null_percentage
    ))?;

    Ok(())
}
```

## PMAT Testing

### Property Tests

```rust
proptest! {
    // Null count is non-negative
    #[test]
    fn null_count_non_negative(batch in batch_with_nulls_strategy()) {
        let counts = null_count(&batch);
        for count in counts.values() {
            prop_assert!(*count >= 0);
        }
    }

    // Null count doesn't exceed row count
    #[test]
    fn null_count_bounded(batch in batch_with_nulls_strategy()) {
        let counts = null_count(&batch);
        for count in counts.values() {
            prop_assert!(*count <= batch.num_rows());
        }
    }

    // Non-nullable columns have zero nulls
    #[test]
    fn non_nullable_zero_nulls(batch in non_nullable_batch_strategy()) {
        let counts = null_count(&batch);
        for (col_name, count) in &counts {
            let field = batch.schema().field_with_name(col_name).unwrap();
            if !field.is_nullable() {
                prop_assert_eq!(*count, 0);
            }
        }
    }

    // Result includes all columns
    #[test]
    fn null_count_all_columns(batch in batch_strategy()) {
        let counts = null_count(&batch);
        prop_assert_eq!(counts.len(), batch.num_columns());
    }
}
```

### Mutation Testing Targets

| Mutation | Expected Behavior |
|----------|-------------------|
| Count → Count + 1 | Bounded test fails |
| Skip column | All columns test fails |
| Invert null check | Values don't match |

### Adversarial Tests

```rust
#[test]
fn test_null_count_empty_batch() {
    let batch = empty_batch();
    let counts = null_count(&batch);
    assert!(counts.is_empty() || counts.values().all(|&c| c == 0));
}

#[test]
fn test_null_count_all_null_column() {
    let batch = batch_with_all_null_column();
    let counts = null_count(&batch);
    assert_eq!(counts["nullable_col"], batch.num_rows());
}

#[test]
fn test_null_count_no_nulls() {
    let batch = batch_without_nulls();
    let counts = null_count(&batch);
    assert!(counts.values().all(|&c| c == 0));
}
```

## QA Checklist

| # | Check | Status |
|---|-------|--------|
| 1 | `cargo run` succeeds | Pass |
| 2 | `cargo test` passes | Pass |
| 3 | Deterministic output | Pass |
| 4 | No temp files leaked | Pass |
| 5 | Memory usage stable | Pass |
| 6 | Platform independent | Pass |
| 7 | Clippy clean | Pass |
| 8 | Rustfmt standard | Pass |
| 9 | No `unwrap()` in logic | Pass |
| 10 | Property tests pass | Pass |
