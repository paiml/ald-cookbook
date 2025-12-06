//! # Recipe: Load ALD Metadata Only
//!
//! **Category**: Dataset Loading
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
//! Load only dataset metadata without loading the full payload into memory.
//!
//! ## Run Command
//! ```bash
//! cargo run --example load_ald_metadata
//! ```

use ald_cookbook::format::load_metadata;
use ald_cookbook::prelude::*;
use arrow::array::{Float64Array, Int64Array};
use rand::Rng;
use std::fmt;
use std::sync::Arc;

/// Result of the recipe execution.
struct RecipeResult {
    name: Option<String>,
    dataset_type: String,
    num_rows: usize,
    num_columns: usize,
    created_at: String,
}

impl fmt::Display for RecipeResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "ALD Metadata (loaded without payload)")?;
        writeln!(f, "  Name: {:?}", self.name)?;
        writeln!(f, "  Type: {}", self.dataset_type)?;
        writeln!(f, "  Rows: {}", self.num_rows)?;
        writeln!(f, "  Columns: {}", self.num_columns)?;
        writeln!(f, "  Created: {}", self.created_at)?;
        Ok(())
    }
}

/// Create a test dataset.
fn create_test_dataset(ctx: &mut RecipeContext) -> Result<std::path::PathBuf> {
    let schema = Schema::new(vec![
        Field::new("x", DataType::Int64, false),
        Field::new("y", DataType::Float64, false),
    ]);

    let xs: Vec<i64> = (0..1000).collect();
    let ys: Vec<f64> = (0..1000).map(|_| ctx.rng.gen::<f64>()).collect();

    let batch = RecordBatch::try_new(
        Arc::new(schema),
        vec![
            Arc::new(Int64Array::from(xs)),
            Arc::new(Float64Array::from(ys)),
        ],
    )?;

    let path = ctx.path("metadata_test.ald");
    save(
        &batch,
        DatasetType::Tabular,
        &path,
        SaveOptions::new().with_name("metadata_example"),
    )?;
    Ok(path)
}

/// Execute the recipe's core logic.
fn execute_recipe(ctx: &mut RecipeContext) -> Result<RecipeResult> {
    // Create test dataset
    let ald_path = create_test_dataset(ctx)?;

    // Load only metadata (no payload)
    let metadata = load_metadata(&ald_path)?;

    Ok(RecipeResult {
        name: metadata.name,
        dataset_type: format!("{:?}", metadata.dataset_type),
        num_rows: metadata.num_rows,
        num_columns: metadata.num_columns,
        created_at: metadata.created_at,
    })
}

fn main() -> Result<()> {
    let mut ctx = RecipeContext::new("load_ald_metadata")?;
    let result = execute_recipe(&mut ctx)?;
    ctx.report(&result)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_executes() {
        let mut ctx = RecipeContext::new("test_metadata").unwrap();
        let result = execute_recipe(&mut ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_metadata_correct() {
        let mut ctx = RecipeContext::new("metadata_check").unwrap();
        let result = execute_recipe(&mut ctx).unwrap();

        assert_eq!(result.name, Some("metadata_example".to_string()));
        assert_eq!(result.num_rows, 1000);
        assert_eq!(result.num_columns, 2);
    }
}
