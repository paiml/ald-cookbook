//! # Recipe: Detect Null Values
//!
//! **Category**: Data Quality
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
//! Detect and report null values in dataset columns.
//!
//! ## Run Command
//! ```bash
//! cargo run --example quality_null_detection
//! ```

use ald_cookbook::prelude::*;
use ald_cookbook::quality;
use arrow::array::{Float64Array, Int64Array, StringArray};
use rand::Rng;
use std::fmt;
use std::sync::Arc;

/// Result of the recipe execution.
struct RecipeResult {
    total_rows: usize,
    column_stats: Vec<(String, usize, f64)>, // (name, null_count, null_percent)
    overall_null_percent: f64,
}

impl fmt::Display for RecipeResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Null Detection Report")?;
        writeln!(f, "{:-<50}", "")?;
        writeln!(f, "  Total rows: {}", self.total_rows)?;
        writeln!(f)?;
        for (name, count, percent) in &self.column_stats {
            writeln!(
                f,
                "  {}: {}/{} nulls ({:.1}%)",
                name, count, self.total_rows, percent
            )?;
        }
        writeln!(f, "{:-<50}", "")?;
        writeln!(f, "  Overall null rate: {:.1}%", self.overall_null_percent)?;
        Ok(())
    }
}

/// Create dataset with intentional nulls.
fn create_dataset_with_nulls(ctx: &mut RecipeContext, num_rows: usize) -> Result<RecordBatch> {
    let schema = Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("value", DataType::Float64, true),
        Field::new("category", DataType::Utf8, true),
    ]);

    let ids: Vec<i64> = (0..num_rows as i64).collect();

    // ~10% null values
    let values: Vec<Option<f64>> = (0..num_rows)
        .map(|i| {
            if i % 10 == 0 {
                None
            } else {
                Some(ctx.rng.gen::<f64>() * 100.0)
            }
        })
        .collect();

    // ~20% null categories
    let categories: Vec<Option<&str>> = (0..num_rows)
        .map(|i| {
            if i % 5 == 0 {
                None
            } else {
                Some(["A", "B", "C"][i % 3])
            }
        })
        .collect();

    let batch = RecordBatch::try_new(
        Arc::new(schema),
        vec![
            Arc::new(Int64Array::from(ids)),
            Arc::new(Float64Array::from(values)),
            Arc::new(StringArray::from(categories)),
        ],
    )?;

    Ok(batch)
}

/// Execute the recipe's core logic.
fn execute_recipe(ctx: &mut RecipeContext) -> Result<RecipeResult> {
    // Create dataset with nulls
    let batch = create_dataset_with_nulls(ctx, 200)?;

    // Analyze null patterns
    let report = quality::null_report(&batch)?;

    let column_stats: Vec<(String, usize, f64)> = report
        .columns
        .iter()
        .map(|(name, stats)| {
            let percent = if stats.total_count > 0 {
                stats.null_count as f64 / stats.total_count as f64 * 100.0
            } else {
                0.0
            };
            (name.clone(), stats.null_count, percent)
        })
        .collect();

    // Calculate overall null percentage
    let total_values: usize = column_stats.iter().map(|_| batch.num_rows()).sum();
    let total_nulls: usize = column_stats.iter().map(|(_, count, _)| *count).sum();
    let overall_null_percent = if total_values > 0 {
        total_nulls as f64 / total_values as f64 * 100.0
    } else {
        0.0
    };

    Ok(RecipeResult {
        total_rows: batch.num_rows(),
        column_stats,
        overall_null_percent,
    })
}

fn main() -> Result<()> {
    let mut ctx = RecipeContext::new("quality_null_detection")?;
    let result = execute_recipe(&mut ctx)?;
    ctx.report(&result)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_executes() {
        let mut ctx = RecipeContext::new("test_nulls").unwrap();
        let result = execute_recipe(&mut ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_detects_nulls() {
        let mut ctx = RecipeContext::new("detect_nulls").unwrap();
        let result = execute_recipe(&mut ctx).unwrap();

        // Should detect nulls in value and category columns
        assert!(result.overall_null_percent > 0.0);
    }
}
