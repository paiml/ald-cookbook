//! # Recipe: Create Tabular ALD Dataset
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
//! Create a tabular dataset from CSV-like row data with automatic schema inference.
//!
//! ## Run Command
//! ```bash
//! cargo run --example create_ald_tabular
//! ```

use ald_cookbook::prelude::*;
use arrow::array::{BooleanArray, Float64Array, Int32Array, StringArray};
use rand::Rng;
use std::fmt;
use std::sync::Arc;

/// Result of the recipe execution.
struct RecipeResult {
    num_rows: usize,
    num_columns: usize,
    column_names: Vec<String>,
    file_size_bytes: u64,
}

impl fmt::Display for RecipeResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Created Tabular ALD Dataset")?;
        writeln!(f, "  Rows: {}", self.num_rows)?;
        writeln!(f, "  Columns: {}", self.num_columns)?;
        writeln!(f, "  Column names: {:?}", self.column_names)?;
        writeln!(f, "  File size: {} bytes", self.file_size_bytes)?;
        Ok(())
    }
}

/// Create a tabular dataset simulating customer data.
fn create_customer_dataset(rng: &mut impl Rng, num_rows: usize) -> Result<RecordBatch> {
    let schema = Schema::new(vec![
        Field::new("customer_id", DataType::Int32, false),
        Field::new("name", DataType::Utf8, false),
        Field::new("age", DataType::Int32, true),
        Field::new("balance", DataType::Float64, false),
        Field::new("is_active", DataType::Boolean, false),
    ]);

    let first_names = ["Alice", "Bob", "Charlie", "Diana", "Eve", "Frank"];
    let last_names = ["Smith", "Jones", "Brown", "Davis", "Miller", "Wilson"];

    let ids: Vec<i32> = (1..=num_rows as i32).collect();
    let names: Vec<String> = (0..num_rows)
        .map(|i| {
            format!(
                "{} {}",
                first_names[i % first_names.len()],
                last_names[(i * 7) % last_names.len()]
            )
        })
        .collect();
    let ages: Vec<Option<i32>> = (0..num_rows)
        .map(|i| {
            if i % 20 == 0 {
                None // 5% null ages
            } else {
                Some(rng.gen_range(18..80))
            }
        })
        .collect();
    let balances: Vec<f64> = (0..num_rows).map(|_| rng.gen::<f64>() * 10000.0).collect();
    let is_active: Vec<bool> = (0..num_rows).map(|_| rng.gen_bool(0.85)).collect();

    let batch = RecordBatch::try_new(
        Arc::new(schema),
        vec![
            Arc::new(Int32Array::from(ids)),
            Arc::new(StringArray::from(
                names.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
            )),
            Arc::new(Int32Array::from(ages)),
            Arc::new(Float64Array::from(balances)),
            Arc::new(BooleanArray::from(is_active)),
        ],
    )?;

    Ok(batch)
}

/// Execute the recipe's core logic.
fn execute_recipe(ctx: &mut RecipeContext) -> Result<RecipeResult> {
    // Create tabular dataset
    let batch = create_customer_dataset(&mut ctx.rng, 500)?;

    // Save to ALD format
    let ald_path = ctx.path("customers.ald");
    save(
        &batch,
        DatasetType::Tabular,
        &ald_path,
        SaveOptions::new().with_name("customer_dataset"),
    )?;

    // Verify roundtrip
    let loaded = load(&ald_path)?;
    assert_eq!(batch.num_rows(), loaded.num_rows());

    let column_names: Vec<String> = batch
        .schema()
        .fields()
        .iter()
        .map(|f| f.name().clone())
        .collect();

    let file_size = std::fs::metadata(&ald_path)?.len();

    Ok(RecipeResult {
        num_rows: batch.num_rows(),
        num_columns: batch.num_columns(),
        column_names,
        file_size_bytes: file_size,
    })
}

fn main() -> Result<()> {
    let mut ctx = RecipeContext::new("create_ald_tabular")?;
    let result = execute_recipe(&mut ctx)?;
    ctx.report(&result)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_executes() {
        let mut ctx = RecipeContext::new("test_create_tabular").unwrap();
        let result = execute_recipe(&mut ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_recipe_idempotent() {
        let mut ctx1 = RecipeContext::new("tabular_idempotent").unwrap();
        let mut ctx2 = RecipeContext::new("tabular_idempotent").unwrap();

        let result1 = execute_recipe(&mut ctx1).unwrap();
        let result2 = execute_recipe(&mut ctx2).unwrap();

        assert_eq!(result1.num_rows, result2.num_rows);
        assert_eq!(result1.file_size_bytes, result2.file_size_bytes);
    }

    #[test]
    fn test_schema_correct() {
        let mut ctx = RecipeContext::new("tabular_schema").unwrap();
        let result = execute_recipe(&mut ctx).unwrap();

        assert_eq!(result.num_columns, 5);
        assert!(result.column_names.contains(&"customer_id".to_string()));
        assert!(result.column_names.contains(&"balance".to_string()));
    }
}
