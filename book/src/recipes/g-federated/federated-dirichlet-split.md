# Dirichlet Split

**Category**: G (Federated Learning)
**Status**: Verified
**Isolation**: Full
**Idempotency**: Guaranteed

## Overview

Partition data using Dirichlet distribution for non-IID federated learning simulation. Controls label heterogeneity across clients.

## Run the Recipe

```bash
cargo run --example federated_dirichlet_split
```

## Code

```rust
use ald_cookbook::prelude::*;
use ald_cookbook::federated::dirichlet_split;
use ald_cookbook::{RecipeContext, Result};

fn main() -> Result<()> {
    let mut ctx = RecipeContext::new("federated_dirichlet_split")?;

    let batch = create_labeled_dataset(&mut ctx, 10000)?;

    // Split with alpha=0.5 (heterogeneous) into 10 clients
    let partitions = dirichlet_split(&batch, 10, 0.5, "label", ctx.rng())?;

    for (i, partition) in partitions.iter().enumerate() {
        let label_dist = compute_label_distribution(partition, "label");
        println!("Client {}: {} rows, labels: {:?}", i, partition.num_rows(), label_dist);
    }

    ctx.report(&format!(
        "Non-IID split with α=0.5 into {} clients",
        partitions.len()
    ))?;

    Ok(())
}
```

## Alpha Parameter

| Alpha (α) | Effect |
|-----------|--------|
| 0.1 | Highly heterogeneous (pathological) |
| 0.5 | Moderately heterogeneous |
| 1.0 | Mildly heterogeneous |
| 10.0 | Nearly IID |
| ∞ | Perfectly IID |

## PMAT Testing

### Property Tests

```rust
proptest! {
    // Total rows preserved
    #[test]
    fn dirichlet_preserves_total(
        batch in labeled_batch_strategy(),
        n_clients in 2usize..10,
        alpha in 0.1..10.0f64
    ) {
        let partitions = dirichlet_split(&batch, n_clients, alpha, "label", &mut rng())?;
        let total: usize = partitions.iter().map(|p| p.num_rows()).sum();
        prop_assert_eq!(total, batch.num_rows());
    }

    // Low alpha creates heterogeneous splits
    #[test]
    fn dirichlet_low_alpha_heterogeneous(batch in large_labeled_batch()) {
        let partitions = dirichlet_split(&batch, 5, 0.1, "label", &mut rng())?;

        // With low alpha, clients should have different dominant labels
        let dominant_labels: HashSet<_> = partitions.iter()
            .map(|p| get_dominant_label(p, "label"))
            .collect();

        // Not all clients should have the same dominant label
        prop_assert!(dominant_labels.len() > 1);
    }

    // High alpha approaches IID
    #[test]
    fn dirichlet_high_alpha_iid(batch in large_labeled_batch()) {
        let partitions = dirichlet_split(&batch, 5, 100.0, "label", &mut rng())?;

        // With high alpha, label distributions should be similar
        let global_dist = compute_label_distribution(&batch, "label");
        for partition in &partitions {
            let local_dist = compute_label_distribution(partition, "label");
            let kl_div = compute_kl_divergence(&global_dist, &local_dist);
            prop_assert!(kl_div < 0.1);
        }
    }

    // Deterministic with same seed
    #[test]
    fn dirichlet_deterministic(batch in labeled_batch_strategy(), seed in any::<u64>()) {
        let mut rng1 = StdRng::seed_from_u64(seed);
        let mut rng2 = StdRng::seed_from_u64(seed);

        let p1 = dirichlet_split(&batch, 5, 0.5, "label", &mut rng1)?;
        let p2 = dirichlet_split(&batch, 5, 0.5, "label", &mut rng2)?;

        for (a, b) in p1.iter().zip(p2.iter()) {
            prop_assert_eq!(a.num_rows(), b.num_rows());
        }
    }
}
```

### Mutation Testing Targets

| Mutation | Expected Behavior |
|----------|-------------------|
| Alpha → Alpha + 1 | Heterogeneity test fails |
| Wrong Dirichlet sampling | Distribution tests fail |
| Off-by-one in assignment | Total rows test fails |

### Adversarial Tests

```rust
#[test]
fn test_dirichlet_zero_alpha() {
    // Alpha must be positive
    let result = dirichlet_split(&batch, 5, 0.0, "label", &mut rng());
    assert!(result.is_err());
}

#[test]
fn test_dirichlet_negative_alpha() {
    let result = dirichlet_split(&batch, 5, -0.5, "label", &mut rng());
    assert!(result.is_err());
}

#[test]
fn test_dirichlet_nonexistent_label_column() {
    let result = dirichlet_split(&batch, 5, 0.5, "nonexistent", &mut rng());
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
