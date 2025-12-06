# Sample Transform

**Category**: D (Transforms)
**Status**: Verified
**Isolation**: Full
**Idempotency**: Guaranteed

## Overview

Randomly sample rows from a dataset with or without replacement. Supports both fixed-size and ratio-based sampling.

## Run the Recipe

```bash
cargo run --example transform_sample
```

## Code

```rust
use ald_cookbook::prelude::*;
use ald_cookbook::transforms::sample;
use ald_cookbook::{RecipeContext, Result};

fn main() -> Result<()> {
    let mut ctx = RecipeContext::new("transform_sample")?;

    let batch = create_sample_batch(&mut ctx, 1000)?;

    // Sample 100 rows without replacement
    let sampled = sample(&batch, 100, ctx.rng(), false)?;

    ctx.report(&format!(
        "Sampled {} rows from {} (without replacement)",
        sampled.num_rows(),
        batch.num_rows()
    ))?;

    Ok(())
}
```

## PMAT Testing

### Property Tests

```rust
proptest! {
    // Sample without replacement returns exact count
    #[test]
    fn sample_exact_count(batch in batch_strategy(), seed in any::<u64>()) {
        let n = batch.num_rows() / 2;
        let mut rng = StdRng::seed_from_u64(seed);
        let sampled = sample(&batch, n, &mut rng, false)?;
        prop_assert_eq!(sampled.num_rows(), n);
    }

    // Sample with replacement can exceed original size
    #[test]
    fn sample_with_replacement_larger(batch in batch_strategy(), seed in any::<u64>()) {
        let n = batch.num_rows() * 2;
        let mut rng = StdRng::seed_from_u64(seed);
        let sampled = sample(&batch, n, &mut rng, true)?;
        prop_assert_eq!(sampled.num_rows(), n);
    }

    // Sample preserves schema
    #[test]
    fn sample_preserves_schema(batch in batch_strategy(), seed in any::<u64>()) {
        let n = batch.num_rows() / 2;
        let mut rng = StdRng::seed_from_u64(seed);
        let sampled = sample(&batch, n, &mut rng, false)?;
        prop_assert_eq!(sampled.schema(), batch.schema());
    }

    // Sample is deterministic
    #[test]
    fn sample_deterministic(batch in batch_strategy(), seed in any::<u64>()) {
        let n = batch.num_rows() / 2;
        let mut rng1 = StdRng::seed_from_u64(seed);
        let mut rng2 = StdRng::seed_from_u64(seed);

        let sampled1 = sample(&batch, n, &mut rng1, false)?;
        let sampled2 = sample(&batch, n, &mut rng2, false)?;

        assert_batches_equal(&sampled1, &sampled2);
    }
}
```

### Mutation Testing Targets

| Mutation | Expected Behavior |
|----------|-------------------|
| `n` → `n-1` | Exact count test fails |
| Remove replace logic | With replacement test fails |
| Seed handling | Determinism test fails |

### Adversarial Tests

```rust
#[test]
fn test_sample_zero() {
    let batch = create_batch(100);
    let mut rng = StdRng::seed_from_u64(42);
    let sampled = sample(&batch, 0, &mut rng, false)?;
    assert_eq!(sampled.num_rows(), 0);
}

#[test]
fn test_sample_more_than_available_no_replacement() {
    let batch = create_batch(10);
    let mut rng = StdRng::seed_from_u64(42);
    // Should cap at batch size
    let sampled = sample(&batch, 100, &mut rng, false)?;
    assert!(sampled.num_rows() <= 10);
}

#[test]
fn test_sample_empty_batch() {
    let batch = empty_batch();
    let mut rng = StdRng::seed_from_u64(42);
    let sampled = sample(&batch, 10, &mut rng, false)?;
    assert_eq!(sampled.num_rows(), 0);
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
