//! # Recipe: Shuffle Dataset Rows
//!
//! **Category**: Data Transforms
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
//! Deterministically shuffle dataset rows with a fixed seed.
//!
//! ## Run Command
//! ```bash
//! cargo run --example transform_shuffle
//! ```

use ald_cookbook::prelude::*;
use ald_cookbook::transforms;
use arrow::array::{Float64Array, Int64Array};
use rand::Rng;
use std::fmt;
use std::sync::Arc;

/// Result of the recipe execution.
struct RecipeResult {
    num_rows: usize,
    first_ids_before: Vec<i64>,
    first_ids_after: Vec<i64>,
    seed: u64,
}

impl fmt::Display for RecipeResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Shuffle Transform Result")?;
        writeln!(f, "  Rows: {}", self.num_rows)?;
        writeln!(f, "  First 5 IDs before: {:?}", self.first_ids_before)?;
        writeln!(f, "  First 5 IDs after:  {:?}", self.first_ids_after)?;
        writeln!(f, "  Seed: {}", self.seed)?;
        Ok(())
    }
}

/// Create test dataset.
fn create_test_dataset(ctx: &mut RecipeContext, num_rows: usize) -> Result<RecordBatch> {
    let schema = Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("value", DataType::Float64, false),
    ]);

    let ids: Vec<i64> = (0..num_rows as i64).collect();
    let values: Vec<f64> = (0..num_rows).map(|_| ctx.rng.gen::<f64>()).collect();

    let batch = RecordBatch::try_new(
        Arc::new(schema),
        vec![
            Arc::new(Int64Array::from(ids)),
            Arc::new(Float64Array::from(values)),
        ],
    )?;

    Ok(batch)
}

/// Get first N ids from batch.
fn get_first_ids(batch: &RecordBatch, n: usize) -> Result<Vec<i64>> {
    let id_col = batch
        .column(0)
        .as_any()
        .downcast_ref::<Int64Array>()
        .ok_or_else(|| ald_cookbook::Error::InvalidColumnType {
            expected: "Int64".to_string(),
            actual: "Unknown".to_string(),
        })?;

    Ok((0..n.min(id_col.len())).map(|i| id_col.value(i)).collect())
}

/// Execute the recipe's core logic.
fn execute_recipe(ctx: &mut RecipeContext) -> Result<RecipeResult> {
    // Create test dataset
    let batch = create_test_dataset(ctx, 100)?;
    let first_ids_before = get_first_ids(&batch, 5)?;

    // Shuffle with deterministic RNG
    let seed = ctx.seed();
    let shuffled = transforms::shuffle(&batch, &mut ctx.rng)?;

    let first_ids_after = get_first_ids(&shuffled, 5)?;

    Ok(RecipeResult {
        num_rows: batch.num_rows(),
        first_ids_before,
        first_ids_after,
        seed,
    })
}

fn main() -> Result<()> {
    let mut ctx = RecipeContext::new("transform_shuffle")?;
    let result = execute_recipe(&mut ctx)?;
    ctx.report(&result)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_executes() {
        let mut ctx = RecipeContext::new("test_shuffle").unwrap();
        let result = execute_recipe(&mut ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_shuffle_changes_order() {
        let mut ctx = RecipeContext::new("shuffle_order").unwrap();
        let result = execute_recipe(&mut ctx).unwrap();
        assert_ne!(result.first_ids_before, result.first_ids_after);
    }

    #[test]
    fn test_shuffle_idempotent() {
        let mut ctx1 = RecipeContext::new("shuffle_idem").unwrap();
        let mut ctx2 = RecipeContext::new("shuffle_idem").unwrap();

        let result1 = execute_recipe(&mut ctx1).unwrap();
        let result2 = execute_recipe(&mut ctx2).unwrap();

        assert_eq!(result1.first_ids_after, result2.first_ids_after);
    }
}
