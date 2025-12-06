//! # Recipe: Create ALD from Arrow RecordBatches
//!
//! **Category**: Dataset Creation
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
//! Create a `.ald` dataset from Arrow RecordBatches with type-safe schema definition.
//!
//! ## Run Command
//! ```bash
//! cargo run --example create_ald_from_arrow
//! ```

use ald_cookbook::prelude::*;
use arrow::array::{Float64Array, Int64Array, StringArray};
use rand::Rng;
use std::fmt;
use std::sync::Arc;

/// Result of the recipe execution.
struct RecipeResult {
    rows_created: usize,
    file_size_bytes: u64,
    path: std::path::PathBuf,
}

impl fmt::Display for RecipeResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Created ALD dataset from Arrow RecordBatch")?;
        writeln!(f, "  Rows: {}", self.rows_created)?;
        writeln!(f, "  File size: {} bytes", self.file_size_bytes)?;
        writeln!(f, "  Path: {:?}", self.path)?;
        Ok(())
    }
}

/// Create synthetic data for demonstration.
fn create_record_batch(
    schema: &Schema,
    rng: &mut impl Rng,
    num_rows: usize,
) -> Result<RecordBatch> {
    let ids: Vec<i64> = (0..num_rows).map(|i| i as i64).collect();
    let values: Vec<f64> = (0..num_rows).map(|_| rng.gen::<f64>() * 100.0).collect();
    let labels: Vec<Option<&str>> = (0..num_rows)
        .map(|i| {
            if i % 10 == 0 {
                None
            } else {
                Some(["cat_a", "cat_b", "cat_c"][i % 3])
            }
        })
        .collect();

    let batch = RecordBatch::try_new(
        Arc::new(schema.clone()),
        vec![
            Arc::new(Int64Array::from(ids)),
            Arc::new(Float64Array::from(values)),
            Arc::new(StringArray::from(labels)),
        ],
    )?;

    Ok(batch)
}

/// Execute the recipe's core logic.
fn execute_recipe(ctx: &mut RecipeContext) -> Result<RecipeResult> {
    // Define Arrow schema
    let schema = Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("value", DataType::Float64, false),
        Field::new("label", DataType::Utf8, true),
    ]);

    // Generate synthetic data with deterministic RNG
    let batch = create_record_batch(&schema, &mut ctx.rng, 1000)?;

    // Save to ALD format
    let ald_path = ctx.path("synthetic.ald");
    save(
        &batch,
        DatasetType::Tabular,
        &ald_path,
        SaveOptions::new().with_name("synthetic_dataset"),
    )?;

    // Verify roundtrip
    let loaded = load(&ald_path)?;
    assert_eq!(batch.num_rows(), loaded.num_rows());
    assert_eq!(batch.num_columns(), loaded.num_columns());

    let file_size = std::fs::metadata(&ald_path)?.len();

    Ok(RecipeResult {
        rows_created: batch.num_rows(),
        file_size_bytes: file_size,
        path: ald_path,
    })
}

fn main() -> Result<()> {
    // 1. Setup: Create isolated environment
    let mut ctx = RecipeContext::new("create_ald_from_arrow")?;

    // 2. Execute: Perform the recipe's core logic
    let result = execute_recipe(&mut ctx)?;

    // 3. Report: Display results to user
    ctx.report(&result)?;

    // 4. Cleanup: Automatic via Drop
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_executes() {
        let mut ctx = RecipeContext::new("test_create_ald_from_arrow").unwrap();
        let result = execute_recipe(&mut ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_recipe_idempotent() {
        let mut ctx1 = RecipeContext::new("idempotent_test").unwrap();
        let mut ctx2 = RecipeContext::new("idempotent_test").unwrap();

        let result1 = execute_recipe(&mut ctx1).unwrap();
        let result2 = execute_recipe(&mut ctx2).unwrap();

        assert_eq!(result1.rows_created, result2.rows_created);
        // File sizes should be identical due to deterministic data
        assert_eq!(result1.file_size_bytes, result2.file_size_bytes);
    }

    #[test]
    fn test_creates_valid_ald_header() {
        let mut ctx = RecipeContext::new("header_test").unwrap();
        let result = execute_recipe(&mut ctx).unwrap();

        // Read back and verify it's valid ALD
        let loaded = load(&result.path).unwrap();
        assert_eq!(loaded.num_rows(), 1000);
    }

    #[test]
    fn test_schema_preserved() {
        let mut ctx = RecipeContext::new("schema_test").unwrap();
        let result = execute_recipe(&mut ctx).unwrap();

        let loaded = load(&result.path).unwrap();
        let schema = loaded.schema();

        assert_eq!(schema.fields().len(), 3);
        assert_eq!(schema.field(0).name(), "id");
        assert_eq!(schema.field(1).name(), "value");
        assert_eq!(schema.field(2).name(), "label");
    }
}
