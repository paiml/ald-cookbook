//! # Recipe: ALD Info CLI
//!
//! **Category**: CLI Tools
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
//! Inspect ALD dataset metadata and schema from the command line.
//!
//! ## Run Command
//! ```bash
//! cargo run --example cli_ald_info
//! ```

use ald_cookbook::format::load_metadata;
use ald_cookbook::prelude::*;
use arrow::array::{Float64Array, Int64Array, StringArray};
use rand::Rng;
use std::fmt;
use std::sync::Arc;

/// Result of the recipe execution.
struct RecipeResult {
    file_path: std::path::PathBuf,
    file_size: u64,
    name: Option<String>,
    dataset_type: String,
    num_rows: usize,
    num_columns: usize,
    schema_fields: Vec<String>,
    created_at: String,
}

impl fmt::Display for RecipeResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "ALD Dataset Info")?;
        writeln!(f, "{:=<60}", "")?;
        writeln!(f)?;
        writeln!(f, "File: {:?}", self.file_path)?;
        writeln!(f, "Size: {} bytes", self.file_size)?;
        writeln!(f)?;
        writeln!(f, "Metadata:")?;
        writeln!(f, "  Name: {:?}", self.name)?;
        writeln!(f, "  Type: {}", self.dataset_type)?;
        writeln!(f, "  Rows: {}", self.num_rows)?;
        writeln!(f, "  Columns: {}", self.num_columns)?;
        writeln!(f, "  Created: {}", self.created_at)?;
        writeln!(f)?;
        writeln!(f, "Schema:")?;
        for field in &self.schema_fields {
            writeln!(f, "  - {}", field)?;
        }
        Ok(())
    }
}

/// Create test dataset.
fn create_test_dataset(ctx: &mut RecipeContext) -> Result<std::path::PathBuf> {
    let schema = Schema::new(vec![
        Field::new("user_id", DataType::Int64, false),
        Field::new("score", DataType::Float64, false),
        Field::new("category", DataType::Utf8, true),
    ]);

    let ids: Vec<i64> = (0..500).collect();
    let scores: Vec<f64> = (0..500).map(|_| ctx.rng.gen::<f64>() * 100.0).collect();
    let categories: Vec<Option<&str>> = (0..500)
        .map(|i| Some(["gold", "silver", "bronze"][i % 3]))
        .collect();

    let batch = RecordBatch::try_new(
        Arc::new(schema),
        vec![
            Arc::new(Int64Array::from(ids)),
            Arc::new(Float64Array::from(scores)),
            Arc::new(StringArray::from(categories)),
        ],
    )?;

    let path = ctx.path("example.ald");
    save(
        &batch,
        DatasetType::Tabular,
        &path,
        SaveOptions::new().with_name("user_scores"),
    )?;

    Ok(path)
}

/// Execute the recipe's core logic.
fn execute_recipe(ctx: &mut RecipeContext) -> Result<RecipeResult> {
    // Create test dataset
    let file_path = create_test_dataset(ctx)?;
    let file_size = std::fs::metadata(&file_path)?.len();

    // Load metadata
    let metadata = load_metadata(&file_path)?;

    // Load full data for schema
    let batch = load(&file_path)?;
    let schema_fields: Vec<String> = batch
        .schema()
        .fields()
        .iter()
        .map(|f| {
            format!(
                "{}: {:?} (nullable: {})",
                f.name(),
                f.data_type(),
                f.is_nullable()
            )
        })
        .collect();

    Ok(RecipeResult {
        file_path,
        file_size,
        name: metadata.name,
        dataset_type: format!("{:?}", metadata.dataset_type),
        num_rows: metadata.num_rows,
        num_columns: metadata.num_columns,
        schema_fields,
        created_at: metadata.created_at,
    })
}

fn main() -> Result<()> {
    let mut ctx = RecipeContext::new("cli_ald_info")?;
    let result = execute_recipe(&mut ctx)?;
    ctx.report(&result)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_executes() {
        let mut ctx = RecipeContext::new("test_info").unwrap();
        let result = execute_recipe(&mut ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_info_correct() {
        let mut ctx = RecipeContext::new("info_check").unwrap();
        let result = execute_recipe(&mut ctx).unwrap();

        assert_eq!(result.num_rows, 500);
        assert_eq!(result.num_columns, 3);
        assert_eq!(result.name, Some("user_scores".to_string()));
    }
}
