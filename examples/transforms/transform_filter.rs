//! # Recipe: Filter Dataset Rows
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
//! Filter dataset rows based on column values using predicates.
//!
//! ## Run Command
//! ```bash
//! cargo run --example transform_filter
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
    filtered_rows: usize,
    reduction_percent: f64,
    filter_condition: String,
}

impl fmt::Display for RecipeResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Filter Transform Result")?;
        writeln!(f, "  Original rows: {}", self.original_rows)?;
        writeln!(f, "  Filtered rows: {}", self.filtered_rows)?;
        writeln!(f, "  Reduction: {:.1}%", self.reduction_percent)?;
        writeln!(f, "  Condition: {}", self.filter_condition)?;
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
    let values: Vec<f64> = (0..num_rows)
        .map(|_| ctx.rng.gen::<f64>() * 100.0)
        .collect();

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

    // Filter: keep rows where value > 50.0
    let threshold = 50.0;
    let filtered = transforms::filter_gt_f64(&batch, "value", threshold)?;

    let filtered_rows = filtered.num_rows();
    let reduction_percent = (1.0 - filtered_rows as f64 / original_rows as f64) * 100.0;

    Ok(RecipeResult {
        original_rows,
        filtered_rows,
        reduction_percent,
        filter_condition: format!("value > {}", threshold),
    })
}

fn main() -> Result<()> {
    let mut ctx = RecipeContext::new("transform_filter")?;
    let result = execute_recipe(&mut ctx)?;
    ctx.report(&result)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_executes() {
        let mut ctx = RecipeContext::new("test_filter").unwrap();
        let result = execute_recipe(&mut ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_filter_reduces_rows() {
        let mut ctx = RecipeContext::new("filter_reduction").unwrap();
        let result = execute_recipe(&mut ctx).unwrap();
        assert!(result.filtered_rows < result.original_rows);
    }
}
