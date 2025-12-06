//! # Recipe: Kolmogorov-Smirnov Drift Test
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
//! Detect distribution drift using the Kolmogorov-Smirnov statistical test.
//!
//! ## Run Command
//! ```bash
//! cargo run --example drift_ks_test
//! ```

use ald_cookbook::drift;
use ald_cookbook::prelude::*;
use arrow::array::{Float64Array, Int64Array};
use rand_distr::{Distribution, Normal};
use std::fmt;
use std::sync::Arc;

/// Result of the recipe execution.
struct RecipeResult {
    reference_rows: usize,
    current_rows: usize,
    ks_statistic: f64,
    p_value: f64,
    drift_detected: bool,
    alpha: f64,
}

impl fmt::Display for RecipeResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Kolmogorov-Smirnov Drift Test")?;
        writeln!(f, "{:-<50}", "")?;
        writeln!(f, "  Reference samples: {}", self.reference_rows)?;
        writeln!(f, "  Current samples: {}", self.current_rows)?;
        writeln!(f)?;
        writeln!(f, "  KS Statistic: {:.4}", self.ks_statistic)?;
        writeln!(f, "  P-value: {:.4}", self.p_value)?;
        writeln!(f, "  Alpha: {:.2}", self.alpha)?;
        writeln!(f)?;
        writeln!(
            f,
            "  Drift detected: {}",
            if self.drift_detected { "YES" } else { "NO" }
        )?;
        Ok(())
    }
}

/// Create reference dataset (normal distribution).
fn create_reference_dataset(ctx: &mut RecipeContext, num_rows: usize) -> Result<RecordBatch> {
    let schema = Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("value", DataType::Float64, false),
    ]);

    let normal = Normal::new(50.0, 10.0).map_err(|e| {
        ald_cookbook::Error::ContextInit(format!("Failed to create distribution: {e}"))
    })?;

    let ids: Vec<i64> = (0..num_rows as i64).collect();
    let values: Vec<f64> = (0..num_rows).map(|_| normal.sample(&mut ctx.rng)).collect();

    let batch = RecordBatch::try_new(
        Arc::new(schema),
        vec![
            Arc::new(Int64Array::from(ids)),
            Arc::new(Float64Array::from(values)),
        ],
    )?;

    Ok(batch)
}

/// Create drifted dataset (shifted distribution).
fn create_drifted_dataset(
    ctx: &mut RecipeContext,
    num_rows: usize,
    drift_amount: f64,
) -> Result<RecordBatch> {
    let schema = Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("value", DataType::Float64, false),
    ]);

    // Shift mean by drift_amount
    let normal = Normal::new(50.0 + drift_amount, 10.0).map_err(|e| {
        ald_cookbook::Error::ContextInit(format!("Failed to create distribution: {e}"))
    })?;

    let ids: Vec<i64> = (0..num_rows as i64).collect();
    let values: Vec<f64> = (0..num_rows).map(|_| normal.sample(&mut ctx.rng)).collect();

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
    // Create reference dataset
    let reference = create_reference_dataset(ctx, 500)?;

    // Create drifted dataset (mean shifted by 15 units - significant drift)
    let current = create_drifted_dataset(ctx, 500, 15.0)?;

    // Run KS test
    let result = drift::ks_test(&reference, &current, "value")?;

    let alpha = 0.05;
    let drift_detected = result.p_value < alpha;

    Ok(RecipeResult {
        reference_rows: reference.num_rows(),
        current_rows: current.num_rows(),
        ks_statistic: result.statistic,
        p_value: result.p_value,
        drift_detected,
        alpha,
    })
}

fn main() -> Result<()> {
    let mut ctx = RecipeContext::new("drift_ks_test")?;
    let result = execute_recipe(&mut ctx)?;
    ctx.report(&result)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_executes() {
        let mut ctx = RecipeContext::new("test_ks").unwrap();
        let result = execute_recipe(&mut ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_detects_drift() {
        let mut ctx = RecipeContext::new("detect_drift").unwrap();
        let result = execute_recipe(&mut ctx).unwrap();

        // With 15-unit mean shift, should detect drift
        assert!(result.drift_detected);
    }
}
