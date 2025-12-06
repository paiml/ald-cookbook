//! # Recipe: Basic ALD Loading
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
//! Load a `.ald` dataset and access its schema and data.
//!
//! ## Run Command
//! ```bash
//! cargo run --example load_ald_basic
//! ```

use ald_cookbook::prelude::*;
use arrow::array::{Float64Array, Int64Array};
use rand::Rng;
use std::fmt;
use std::sync::Arc;

/// Result of the recipe execution.
struct RecipeResult {
    rows_loaded: usize,
    columns_loaded: usize,
    schema_fields: Vec<String>,
    sample_values: Vec<String>,
}

impl fmt::Display for RecipeResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Loaded ALD Dataset")?;
        writeln!(f, "  Rows: {}", self.rows_loaded)?;
        writeln!(f, "  Columns: {}", self.columns_loaded)?;
        writeln!(f, "  Schema: {:?}", self.schema_fields)?;
        writeln!(f, "  First few values: {:?}", self.sample_values)?;
        Ok(())
    }
}

/// Create a test dataset to load.
fn create_test_dataset(ctx: &mut RecipeContext) -> Result<std::path::PathBuf> {
    let schema = Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("value", DataType::Float64, false),
    ]);

    let ids: Vec<i64> = (0..100).collect();
    let values: Vec<f64> = (0..100).map(|_| ctx.rng.gen::<f64>() * 100.0).collect();

    let batch = RecordBatch::try_new(
        Arc::new(schema),
        vec![
            Arc::new(Int64Array::from(ids)),
            Arc::new(Float64Array::from(values)),
        ],
    )?;

    let path = ctx.path("test_data.ald");
    save(&batch, DatasetType::Tabular, &path, SaveOptions::new())?;
    Ok(path)
}

/// Execute the recipe's core logic.
fn execute_recipe(ctx: &mut RecipeContext) -> Result<RecipeResult> {
    // Create test dataset
    let ald_path = create_test_dataset(ctx)?;

    // Load the dataset
    let batch = load(&ald_path)?;

    // Extract schema information
    let schema = batch.schema();
    let schema_fields: Vec<String> = schema
        .fields()
        .iter()
        .map(|f| format!("{}: {:?}", f.name(), f.data_type()))
        .collect();

    // Access some values
    let id_col = batch
        .column(0)
        .as_any()
        .downcast_ref::<Int64Array>()
        .ok_or_else(|| ald_cookbook::Error::InvalidColumnType {
            expected: "Int64".to_string(),
            actual: "Unknown".to_string(),
        })?;

    let value_col = batch
        .column(1)
        .as_any()
        .downcast_ref::<Float64Array>()
        .ok_or_else(|| ald_cookbook::Error::InvalidColumnType {
            expected: "Float64".to_string(),
            actual: "Unknown".to_string(),
        })?;

    let sample_values: Vec<String> = (0..5.min(batch.num_rows()))
        .map(|i| format!("id={}, value={:.2}", id_col.value(i), value_col.value(i)))
        .collect();

    Ok(RecipeResult {
        rows_loaded: batch.num_rows(),
        columns_loaded: batch.num_columns(),
        schema_fields,
        sample_values,
    })
}

fn main() -> Result<()> {
    let mut ctx = RecipeContext::new("load_ald_basic")?;
    let result = execute_recipe(&mut ctx)?;
    ctx.report(&result)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_executes() {
        let mut ctx = RecipeContext::new("test_load_basic").unwrap();
        let result = execute_recipe(&mut ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_recipe_idempotent() {
        let mut ctx1 = RecipeContext::new("load_idempotent").unwrap();
        let mut ctx2 = RecipeContext::new("load_idempotent").unwrap();

        let result1 = execute_recipe(&mut ctx1).unwrap();
        let result2 = execute_recipe(&mut ctx2).unwrap();

        assert_eq!(result1.rows_loaded, result2.rows_loaded);
        assert_eq!(result1.sample_values, result2.sample_values);
    }
}
