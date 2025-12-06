# PSI Analysis

**Category**: F (Drift Detection)
**Status**: Verified
**Isolation**: Full
**Idempotency**: Guaranteed

## Overview

Population Stability Index (PSI) for measuring distribution drift. Widely used in credit scoring and risk modeling.

## Run the Recipe

```bash
cargo run --example drift_psi
```

## Code

```rust
use ald_cookbook::prelude::*;
use ald_cookbook::drift::calculate_psi;
use ald_cookbook::{RecipeContext, Result};

fn main() -> Result<()> {
    let mut ctx = RecipeContext::new("drift_psi")?;

    let reference = create_reference_batch(&mut ctx)?;
    let current = create_current_batch(&mut ctx)?;

    // Calculate PSI with 10 bins
    let psi = calculate_psi(&reference, &current, "value", 10)?;

    let interpretation = match psi {
        x if x < 0.1 => "No significant change",
        x if x < 0.25 => "Moderate change",
        _ => "Significant change - investigate",
    };

    ctx.report(&format!(
        "PSI Analysis:\n  PSI: {:.4}\n  Interpretation: {}",
        psi, interpretation
    ))?;

    Ok(())
}
```

## PSI Interpretation

| PSI Value | Interpretation |
|-----------|----------------|
| < 0.10 | No significant change |
| 0.10 - 0.25 | Moderate change |
| > 0.25 | Significant change |

## PMAT Testing

### Property Tests

```rust
proptest! {
    // PSI is non-negative
    #[test]
    fn psi_non_negative(
        baseline in distribution_strategy(),
        current in distribution_strategy(),
        bins in 5usize..20
    ) {
        let psi = calculate_psi(&baseline_batch, &current_batch, "value", bins)?;
        prop_assert!(psi >= 0.0);
    }

    // Identical distributions have PSI near 0
    #[test]
    fn psi_identical_low(values in distribution_strategy(), bins in 5usize..20) {
        let batch = create_batch(values.clone());
        let psi = calculate_psi(&batch, &batch, "value", bins)?;
        prop_assert!(psi < 0.01);
    }

    // PSI is approximately symmetric
    #[test]
    fn psi_symmetric(
        baseline in distribution_strategy(),
        current in distribution_strategy(),
        bins in 5usize..20
    ) {
        let psi1 = calculate_psi(&baseline_batch, &current_batch, "value", bins)?;
        let psi2 = calculate_psi(&current_batch, &baseline_batch, "value", bins)?;
        prop_assert!((psi1 - psi2).abs() < 0.01);
    }

    // More bins doesn't change interpretation dramatically
    #[test]
    fn psi_stable_across_bins(
        baseline in distribution_strategy(),
        current in distribution_strategy()
    ) {
        let psi_10 = calculate_psi(&baseline_batch, &current_batch, "value", 10)?;
        let psi_20 = calculate_psi(&baseline_batch, &current_batch, "value", 20)?;
        // Should be in same interpretation bucket
        let same_bucket = (psi_10 < 0.25 && psi_20 < 0.25) || (psi_10 >= 0.25 && psi_20 >= 0.25);
        prop_assert!(same_bucket);
    }
}
```

### Mutation Testing Targets

| Mutation | Expected Behavior |
|----------|-------------------|
| Bin calculation error | Symmetry test fails |
| PSI formula wrong | Non-negative test fails |
| Log calculation wrong | Values don't match |

### Adversarial Tests

```rust
#[test]
fn test_psi_zero_bins() {
    let result = calculate_psi(&sample_batch(), &sample_batch(), "value", 0);
    assert!(result.is_err());
}

#[test]
fn test_psi_one_bin() {
    // Single bin should still work
    let result = calculate_psi(&sample_batch(), &sample_batch(), "value", 1);
    assert!(result.is_ok());
}

#[test]
fn test_psi_empty_baseline() {
    let result = calculate_psi(&empty_batch(), &sample_batch(), "value", 10);
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
