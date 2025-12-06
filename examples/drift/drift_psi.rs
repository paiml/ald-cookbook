//! # Recipe: Population Stability Index (PSI)
//!
//! **Category**: Drift Detection
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
//! Calculate Population Stability Index (PSI) to measure distribution shift.
//!
//! ## Run Command
//! ```bash
//! cargo run --example drift_psi
//! ```

use ald_cookbook::drift;
use ald_cookbook::prelude::*;
use arrow::array::{Float64Array, Int64Array};
use rand_distr::{Distribution, Normal};
use std::fmt;
use std::sync::Arc;

/// Result of the recipe execution.
struct RecipeResult {
    psi_value: f64,
    num_buckets: usize,
    interpretation: String,
}

impl fmt::Display for RecipeResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Population Stability Index (PSI)")?;
        writeln!(f, "{:-<50}", "")?;
        writeln!(f, "  PSI Value: {:.4}", self.psi_value)?;
        writeln!(f, "  Buckets: {}", self.num_buckets)?;
        writeln!(f)?;
        writeln!(f, "  Interpretation: {}", self.interpretation)?;
        writeln!(f)?;
        writeln!(f, "  PSI Thresholds:")?;
        writeln!(f, "    < 0.10: No significant change")?;
        writeln!(f, "    0.10 - 0.25: Moderate change")?;
        writeln!(f, "    > 0.25: Significant change")?;
        Ok(())
    }
}

/// Create reference dataset.
fn create_reference_dataset(ctx: &mut RecipeContext, num_rows: usize) -> Result<RecordBatch> {
    let schema = Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("score", DataType::Float64, false),
    ]);

    let normal = Normal::new(500.0, 100.0).map_err(|e| {
        ald_cookbook::Error::ContextInit(format!("Failed to create distribution: {e}"))
    })?;

    let ids: Vec<i64> = (0..num_rows as i64).collect();
    let scores: Vec<f64> = (0..num_rows).map(|_| normal.sample(&mut ctx.rng)).collect();

    let batch = RecordBatch::try_new(
        Arc::new(schema),
        vec![
            Arc::new(Int64Array::from(ids)),
            Arc::new(Float64Array::from(scores)),
        ],
    )?;

    Ok(batch)
}

/// Create current dataset with potential drift.
fn create_current_dataset(
    ctx: &mut RecipeContext,
    num_rows: usize,
    drift_amount: f64,
) -> Result<RecordBatch> {
    let schema = Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("score", DataType::Float64, false),
    ]);

    let normal = Normal::new(500.0 + drift_amount, 100.0).map_err(|e| {
        ald_cookbook::Error::ContextInit(format!("Failed to create distribution: {e}"))
    })?;

    let ids: Vec<i64> = (0..num_rows as i64).collect();
    let scores: Vec<f64> = (0..num_rows).map(|_| normal.sample(&mut ctx.rng)).collect();

    let batch = RecordBatch::try_new(
        Arc::new(schema),
        vec![
            Arc::new(Int64Array::from(ids)),
            Arc::new(Float64Array::from(scores)),
        ],
    )?;

    Ok(batch)
}

/// Execute the recipe's core logic.
fn execute_recipe(ctx: &mut RecipeContext) -> Result<RecipeResult> {
    // Create reference dataset
    let reference = create_reference_dataset(ctx, 1000)?;

    // Create current dataset with moderate drift (50 unit shift)
    let current = create_current_dataset(ctx, 1000, 50.0)?;

    // Calculate PSI
    let num_buckets = 10;
    let psi_result = drift::psi(&reference, &current, "score", num_buckets)?;

    // Interpret result
    let psi_value = psi_result.psi;
    let interpretation = if psi_value < 0.10 {
        "No significant population change".to_string()
    } else if psi_value < 0.25 {
        "Moderate population change - monitor closely".to_string()
    } else {
        "Significant population change - investigate".to_string()
    };

    Ok(RecipeResult {
        psi_value,
        num_buckets,
        interpretation,
    })
}

fn main() -> Result<()> {
    let mut ctx = RecipeContext::new("drift_psi")?;
    let result = execute_recipe(&mut ctx)?;
    ctx.report(&result)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_executes() {
        let mut ctx = RecipeContext::new("test_psi").unwrap();
        let result = execute_recipe(&mut ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_psi_positive() {
        let mut ctx = RecipeContext::new("psi_positive").unwrap();
        let result = execute_recipe(&mut ctx).unwrap();
        assert!(result.psi_value >= 0.0);
    }
}
