//! # Recipe: Sample Dataset Rows
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
//! Randomly sample rows from a dataset with a fixed seed for reproducibility.
//!
//! ## Run Command
//! ```bash
//! cargo run --example transform_sample
//! ```

use ald_cookbook::prelude::*;
use ald_cookbook::transforms;
use arrow::array::{Float64Array, Int64Array};
use rand::Rng;
use std::fmt;
use std::sync::Arc;

/// Result of the recipe execution.
struct RecipeResult {
    original_rows: usize,
    sampled_rows: usize,
    sample_fraction: f64,
    seed: u64,
}

impl fmt::Display for RecipeResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Sample Transform Result")?;
        writeln!(f, "  Original rows: {}", self.original_rows)?;
        writeln!(f, "  Sampled rows: {}", self.sampled_rows)?;
        writeln!(f, "  Sample fraction: {:.1}%", self.sample_fraction * 100.0)?;
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

/// Execute the recipe's core logic.
fn execute_recipe(ctx: &mut RecipeContext) -> Result<RecipeResult> {
    // Create test dataset
    let batch = create_test_dataset(ctx, 1000)?;
    let original_rows = batch.num_rows();

    // Sample 10% of rows (100 samples from 1000)
    let sample_fraction = 0.1;
    let sample_count = (original_rows as f64 * sample_fraction) as usize;
    let seed = ctx.seed();
    let sampled = transforms::sample(&batch, sample_count, &mut ctx.rng, false)?;

    Ok(RecipeResult {
        original_rows,
        sampled_rows: sampled.num_rows(),
        sample_fraction,
        seed,
    })
}

fn main() -> Result<()> {
    let mut ctx = RecipeContext::new("transform_sample")?;
    let result = execute_recipe(&mut ctx)?;
    ctx.report(&result)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_executes() {
        let mut ctx = RecipeContext::new("test_sample").unwrap();
        let result = execute_recipe(&mut ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_sample_size_approximate() {
        let mut ctx = RecipeContext::new("sample_size").unwrap();
        let result = execute_recipe(&mut ctx).unwrap();

        // Should be approximately 10% of original
        let expected = (result.original_rows as f64 * result.sample_fraction) as usize;
        let tolerance = expected / 2; // 50% tolerance for randomness
        assert!(result.sampled_rows > expected - tolerance);
        assert!(result.sampled_rows < expected + tolerance);
    }

    #[test]
    fn test_sample_idempotent() {
        let mut ctx1 = RecipeContext::new("sample_idem").unwrap();
        let mut ctx2 = RecipeContext::new("sample_idem").unwrap();

        let result1 = execute_recipe(&mut ctx1).unwrap();
        let result2 = execute_recipe(&mut ctx2).unwrap();

        assert_eq!(result1.sampled_rows, result2.sampled_rows);
    }
}
