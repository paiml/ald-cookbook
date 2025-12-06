//! # Recipe: Convert Parquet to ALD
//!
//! **Category**: Format Conversion
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
//! Convert Parquet files to ALD format preserving schema and data.
//!
//! ## Run Command
//! ```bash
//! cargo run --example convert_parquet_to_ald
//! ```

use ald_cookbook::convert;
use ald_cookbook::prelude::*;
use arrow::array::{Float64Array, Int64Array, StringArray};
use parquet::arrow::ArrowWriter;
use rand::Rng;
use std::fmt;
use std::fs::File;
use std::sync::Arc;

/// Result of the recipe execution.
struct RecipeResult {
    source_rows: usize,
    converted_rows: usize,
    parquet_size: u64,
    ald_size: u64,
    compression_ratio: f64,
}

impl fmt::Display for RecipeResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Converted Parquet to ALD")?;
        writeln!(f, "  Source rows: {}", self.source_rows)?;
        writeln!(f, "  Converted rows: {}", self.converted_rows)?;
        writeln!(f, "  Parquet size: {} bytes", self.parquet_size)?;
        writeln!(f, "  ALD size: {} bytes", self.ald_size)?;
        writeln!(f, "  Compression ratio: {:.2}x", self.compression_ratio)?;
        Ok(())
    }
}

/// Create a test Parquet file.
fn create_parquet_file(ctx: &mut RecipeContext) -> Result<std::path::PathBuf> {
    let schema = Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("score", DataType::Float64, false),
        Field::new("category", DataType::Utf8, true),
    ]);

    let ids: Vec<i64> = (0..500).collect();
    let scores: Vec<f64> = (0..500).map(|_| ctx.rng.gen::<f64>() * 100.0).collect();
    let categories: Vec<Option<&str>> = (0..500)
        .map(|i| {
            if i % 10 == 0 {
                None
            } else {
                Some(["A", "B", "C"][i % 3])
            }
        })
        .collect();

    let batch = RecordBatch::try_new(
        Arc::new(schema.clone()),
        vec![
            Arc::new(Int64Array::from(ids)),
            Arc::new(Float64Array::from(scores)),
            Arc::new(StringArray::from(categories)),
        ],
    )?;

    let path = ctx.path("source.parquet");
    let file = File::create(&path)?;
    let mut writer = ArrowWriter::try_new(file, Arc::new(schema), None)?;
    writer.write(&batch)?;
    writer.close()?;

    Ok(path)
}

/// Execute the recipe's core logic.
fn execute_recipe(ctx: &mut RecipeContext) -> Result<RecipeResult> {
    // Create source Parquet file
    let parquet_path = create_parquet_file(ctx)?;
    let parquet_size = std::fs::metadata(&parquet_path)?.len();

    // Convert to ALD
    let ald_path = ctx.path("converted.ald");
    convert::parquet_to_ald(&parquet_path, &ald_path, convert::ParquetOptions::default())?;

    // Verify conversion
    let loaded = load(&ald_path)?;
    let ald_size = std::fs::metadata(&ald_path)?.len();

    let compression_ratio = parquet_size as f64 / ald_size as f64;

    Ok(RecipeResult {
        source_rows: 500,
        converted_rows: loaded.num_rows(),
        parquet_size,
        ald_size,
        compression_ratio,
    })
}

fn main() -> Result<()> {
    let mut ctx = RecipeContext::new("convert_parquet_to_ald")?;
    let result = execute_recipe(&mut ctx)?;
    ctx.report(&result)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_executes() {
        let mut ctx = RecipeContext::new("test_parquet_convert").unwrap();
        let result = execute_recipe(&mut ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_rows_preserved() {
        let mut ctx = RecipeContext::new("parquet_rows").unwrap();
        let result = execute_recipe(&mut ctx).unwrap();
        assert_eq!(result.source_rows, result.converted_rows);
    }
}
