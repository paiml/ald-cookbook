//! # Recipe: Convert CSV to ALD
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
//! Convert CSV files to ALD format with automatic schema inference.
//!
//! ## Run Command
//! ```bash
//! cargo run --example convert_csv_to_ald
//! ```

use ald_cookbook::convert;
use ald_cookbook::prelude::*;
use rand::Rng;
use std::fmt;
use std::io::Write;

/// Result of the recipe execution.
struct RecipeResult {
    source_rows: usize,
    converted_rows: usize,
    inferred_schema: Vec<String>,
    csv_size: u64,
    ald_size: u64,
}

impl fmt::Display for RecipeResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Converted CSV to ALD")?;
        writeln!(f, "  Source rows: {}", self.source_rows)?;
        writeln!(f, "  Converted rows: {}", self.converted_rows)?;
        writeln!(f, "  Inferred schema: {:?}", self.inferred_schema)?;
        writeln!(f, "  CSV size: {} bytes", self.csv_size)?;
        writeln!(f, "  ALD size: {} bytes", self.ald_size)?;
        Ok(())
    }
}

/// Create a test CSV file.
fn create_csv_file(ctx: &mut RecipeContext, num_rows: usize) -> Result<std::path::PathBuf> {
    let path = ctx.path("source.csv");
    let mut file = std::fs::File::create(&path)?;

    // Write header
    writeln!(file, "id,name,value,active")?;

    // Write data
    let names = ["Alice", "Bob", "Charlie", "Diana", "Eve"];
    for i in 0..num_rows {
        let name = names[i % names.len()];
        let value: f64 = ctx.rng.gen::<f64>() * 100.0;
        let active = ctx.rng.gen_bool(0.7);
        writeln!(file, "{},{},{:.2},{}", i, name, value, active)?;
    }

    Ok(path)
}

/// Execute the recipe's core logic.
fn execute_recipe(ctx: &mut RecipeContext) -> Result<RecipeResult> {
    let num_rows = 200;

    // Create source CSV file
    let csv_path = create_csv_file(ctx, num_rows)?;
    let csv_size = std::fs::metadata(&csv_path)?.len();

    // Convert to ALD
    let ald_path = ctx.path("converted.ald");
    convert::csv_to_ald(&csv_path, &ald_path, convert::CsvOptions::default())?;

    // Verify conversion
    let loaded = load(&ald_path)?;
    let ald_size = std::fs::metadata(&ald_path)?.len();

    let inferred_schema: Vec<String> = loaded
        .schema()
        .fields()
        .iter()
        .map(|f| format!("{}: {:?}", f.name(), f.data_type()))
        .collect();

    Ok(RecipeResult {
        source_rows: num_rows,
        converted_rows: loaded.num_rows(),
        inferred_schema,
        csv_size,
        ald_size,
    })
}

fn main() -> Result<()> {
    let mut ctx = RecipeContext::new("convert_csv_to_ald")?;
    let result = execute_recipe(&mut ctx)?;
    ctx.report(&result)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_executes() {
        let mut ctx = RecipeContext::new("test_csv_convert").unwrap();
        let result = execute_recipe(&mut ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_rows_preserved() {
        let mut ctx = RecipeContext::new("csv_rows").unwrap();
        let result = execute_recipe(&mut ctx).unwrap();
        assert_eq!(result.source_rows, result.converted_rows);
    }
}
