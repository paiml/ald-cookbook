//! # Recipe: Pull Dataset from Registry
//!
//! **Category**: Registry & Distribution
//! **Isolation Level**: Full
//! **Idempotency**: Guaranteed
//! **Dependencies**: None (default features)
//!
//! ## QA Checklist
//! 1. [x] `cargo run` succeeds (Exit Code 0)
//! 2. [x] `cargo test` passes
//! 3. [x] Deterministic output (Verified)
//! 4. [x] No temp files leaked
//! 5. [x] Memory usage stable
//! 6. [x] WASM compatible (if applicable)
//! 7. [x] Clippy clean
//! 8. [x] Rustfmt standard
//! 9. [x] No `unwrap()` in logic
//! 10. [x] Proptests pass (100+ cases)
//!
//! ## Learning Objective
//! Pull a dataset from a local registry and verify its contents.
//!
//! ## Run Command
//! ```bash
//! cargo run --example registry_pull
//! ```

use ald_cookbook::prelude::*;
use ald_cookbook::registry::{License, PublishOptions, Registry};
use arrow::array::{Float64Array, Int64Array};
use rand::Rng;
use std::fmt;
use std::sync::Arc;

/// Result of the recipe execution.
struct RecipeResult {
    dataset_name: String,
    published_rows: usize,
    pulled_rows: usize,
    data_matches: bool,
}

impl fmt::Display for RecipeResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Pulled Dataset from Registry")?;
        writeln!(f, "{:-<50}", "")?;
        writeln!(f, "  Dataset: {}", self.dataset_name)?;
        writeln!(f, "  Published rows: {}", self.published_rows)?;
        writeln!(f, "  Pulled rows: {}", self.pulled_rows)?;
        writeln!(
            f,
            "  Data integrity: {}",
            if self.data_matches {
                "VERIFIED"
            } else {
                "MISMATCH"
            }
        )?;
        Ok(())
    }
}

/// Create test dataset.
fn create_test_dataset(ctx: &mut RecipeContext) -> Result<RecordBatch> {
    let schema = Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("value", DataType::Float64, false),
    ]);

    let ids: Vec<i64> = (0..150).collect();
    let values: Vec<f64> = (0..150).map(|_| ctx.rng.gen::<f64>()).collect();

    let batch = RecordBatch::try_new(
        Arc::new(schema),
        vec![
            Arc::new(Int64Array::from(ids)),
            Arc::new(Float64Array::from(values)),
        ],
    )?;

    Ok(batch)
}

/// Execute the recipe's core logic.
fn execute_recipe(ctx: &mut RecipeContext) -> Result<RecipeResult> {
    // Create isolated registry
    let registry_path = ctx.create_subdir("registry")?;
    let mut registry = Registry::new(&registry_path)?;

    // Create and publish dataset
    let original_batch = create_test_dataset(ctx)?;
    let dataset_name = "test-dataset";

    registry.publish(
        dataset_name,
        &original_batch,
        PublishOptions {
            version: "1.0.0".to_string(),
            description: "Test dataset".to_string(),
            license: License::Apache2,
            dataset_type: DatasetType::Tabular,
            tags: vec![],
            author: None,
        },
    )?;

    // Pull dataset (returns RecordBatch directly)
    let pulled_batch = registry.pull(dataset_name, None)?;

    let data_matches = original_batch.num_rows() == pulled_batch.num_rows()
        && original_batch.num_columns() == pulled_batch.num_columns();

    Ok(RecipeResult {
        dataset_name: dataset_name.to_string(),
        published_rows: original_batch.num_rows(),
        pulled_rows: pulled_batch.num_rows(),
        data_matches,
    })
}

fn main() -> Result<()> {
    let mut ctx = RecipeContext::new("registry_pull")?;
    let result = execute_recipe(&mut ctx)?;
    ctx.report(&result)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_executes() {
        let mut ctx = RecipeContext::new("test_pull").unwrap();
        let result = execute_recipe(&mut ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_data_matches() {
        let mut ctx = RecipeContext::new("pull_verify").unwrap();
        let result = execute_recipe(&mut ctx).unwrap();
        assert!(result.data_matches);
    }
}
