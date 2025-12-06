# Publish Dataset

**Category**: I (Registry)
**Status**: Verified
**Isolation**: Full
**Idempotency**: Guaranteed

## Overview

Publish datasets to a local or remote registry with versioning support. Enables reproducible dataset distribution.

## Run the Recipe

```bash
cargo run --example registry_publish
```

## Code

```rust
use ald_cookbook::prelude::*;
use ald_cookbook::registry::LocalRegistry;
use ald_cookbook::{RecipeContext, Result};

fn main() -> Result<()> {
    let mut ctx = RecipeContext::new("registry_publish")?;

    let registry = LocalRegistry::new(ctx.temp_path("registry"))?;
    let batch = create_sample_batch(&mut ctx)?;

    // Publish with version
    let path = registry.publish(&batch, "my-dataset", "1.0.0")?;

    // Verify it exists
    let exists = registry.exists("my-dataset", "1.0.0")?;

    ctx.report(&format!(
        "Published my-dataset@1.0.0\n  Path: {:?}\n  Exists: {}",
        path, exists
    ))?;

    Ok(())
}
```

## PMAT Testing

### Property Tests

```rust
proptest! {
    // Published dataset can be retrieved
    #[test]
    fn publish_then_exists(batch in batch_strategy(), name in dataset_name(), version in version_string()) {
        let registry = LocalRegistry::new(temp_dir())?;

        registry.publish(&batch, &name, &version)?;
        let exists = registry.exists(&name, &version)?;

        prop_assert!(exists);
    }

    // Different versions are independent
    #[test]
    fn versions_independent(batch1 in batch_strategy(), batch2 in batch_strategy(), name in dataset_name()) {
        let registry = LocalRegistry::new(temp_dir())?;

        registry.publish(&batch1, &name, "1.0.0")?;
        registry.publish(&batch2, &name, "2.0.0")?;

        let (loaded1, _) = registry.pull(&name, "1.0.0")?;
        let (loaded2, _) = registry.pull(&name, "2.0.0")?;

        prop_assert_eq!(loaded1.num_rows(), batch1.num_rows());
        prop_assert_eq!(loaded2.num_rows(), batch2.num_rows());
    }

    // List shows all published datasets
    #[test]
    fn list_all_published(batches in prop::collection::vec(batch_strategy(), 1..5)) {
        let registry = LocalRegistry::new(temp_dir())?;

        for (i, batch) in batches.iter().enumerate() {
            registry.publish(batch, &format!("dataset-{}", i), "1.0.0")?;
        }

        let list = registry.list()?;
        prop_assert_eq!(list.len(), batches.len());
    }

    // Overwrite same version updates data
    #[test]
    fn overwrite_updates(batch1 in batch_strategy(), batch2 in batch_strategy(), name in dataset_name()) {
        prop_assume!(batch1.num_rows() != batch2.num_rows());

        let registry = LocalRegistry::new(temp_dir())?;

        registry.publish(&batch1, &name, "1.0.0")?;
        registry.publish(&batch2, &name, "1.0.0")?;  // Overwrite

        let (loaded, _) = registry.pull(&name, "1.0.0")?;
        prop_assert_eq!(loaded.num_rows(), batch2.num_rows());
    }
}
```

### Mutation Testing Targets

| Mutation | Expected Behavior |
|----------|-------------------|
| Skip file write | Exists check fails |
| Wrong version path | Versions not independent |
| Missing metadata | List incomplete |

### Adversarial Tests

```rust
#[test]
fn test_publish_invalid_name() {
    let registry = LocalRegistry::new(temp_dir())?;
    let result = registry.publish(&batch, "", "1.0.0");
    assert!(result.is_err());
}

#[test]
fn test_publish_invalid_version() {
    let registry = LocalRegistry::new(temp_dir())?;
    let result = registry.publish(&batch, "dataset", "");
    assert!(result.is_err());
}

#[test]
fn test_publish_to_readonly_dir() {
    let readonly_registry = LocalRegistry::new("/readonly")?;
    let result = readonly_registry.publish(&batch, "dataset", "1.0.0");
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
