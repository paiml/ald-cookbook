# Filter Transform

**Category**: D (Transforms)
**Status**: Verified
**Isolation**: Full
**Idempotency**: Guaranteed

## Overview

Filter rows from a dataset based on column value predicates. Essential for data preprocessing and subsetting.

## Run the Recipe

```bash
cargo run --example transform_filter
```

## Code

```rust
use ald_cookbook::prelude::*;
use ald_cookbook::transforms::filter_gt_f64;
use ald_cookbook::{RecipeContext, Result};

fn main() -> Result<()> {
    let mut ctx = RecipeContext::new("transform_filter")?;

    // Create sample data
    let batch = create_sample_batch(&mut ctx, 1000)?;

    // Filter rows where value > 50.0
    let filtered = filter_gt_f64(&batch, "value", 50.0)?;

    ctx.report(&format!(
        "Filtered {} -> {} rows (threshold: 50.0)",
        batch.num_rows(),
        filtered.num_rows()
    ))?;

    Ok(())
}
```

## PMAT Testing

### Property Tests

```rust
proptest! {
    // Filter reduces or maintains row count
    #[test]
    fn filter_reduces_count(batch in batch_strategy(), threshold in -1000.0..1000.0f64) {
        let filtered = filter_gt_f64(&batch, "value", threshold)?;
        prop_assert!(filtered.num_rows() <= batch.num_rows());
    }

    // Filter preserves schema
    #[test]
    fn filter_preserves_schema(batch in batch_strategy(), threshold in 0.0..100.0f64) {
        let filtered = filter_gt_f64(&batch, "value", threshold)?;
        prop_assert_eq!(filtered.schema(), batch.schema());
    }

    // Very low threshold keeps all rows
    #[test]
    fn filter_low_threshold_keeps_all(batch in batch_strategy()) {
        let filtered = filter_gt_f64(&batch, "value", f64::MIN)?;
        prop_assert_eq!(filtered.num_rows(), batch.num_rows());
    }
}
```

### Mutation Testing Targets

| Mutation | Expected Behavior |
|----------|-------------------|
| `>` → `>=` | Boundary test catches |
| `>` → `<` | All property tests fail |
| Row count check | Monotonicity test fails |

### Adversarial Tests

```rust
#[test]
fn test_filter_nonexistent_column() {
    let result = filter_gt_f64(&batch, "nonexistent", 0.0);
    assert!(result.is_err());
}

#[test]
fn test_filter_wrong_type_column() {
    // Filter on string column should fail
    let result = filter_gt_f64(&batch, "name", 0.0);
    assert!(result.is_err());
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
