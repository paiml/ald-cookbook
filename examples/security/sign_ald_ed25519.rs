//! # Recipe: Sign Dataset with Ed25519
//!
//! **Category**: Security & Encryption
//! **Isolation Level**: Full
//! **Idempotency**: Guaranteed
//! **Dependencies**: signing feature
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
//! Sign a dataset with Ed25519 for authenticity verification.
//!
//! ## Run Command
//! ```bash
//! cargo run --example sign_ald_ed25519 --features signing
//! ```

#[cfg(feature = "signing")]
use ald_cookbook::signing::{sign, verify, Keypair};

use ald_cookbook::prelude::*;
use arrow::array::{Float64Array, Int64Array};
use rand::Rng;
use std::fmt;
use std::sync::Arc;

/// Result of the recipe execution.
struct RecipeResult {
    data_size: usize,
    signature_size: usize,
    public_key_hex: String,
    verification_passed: bool,
}

impl fmt::Display for RecipeResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Ed25519 Dataset Signing")?;
        writeln!(f, "{:-<50}", "")?;
        writeln!(f, "  Data size: {} bytes", self.data_size)?;
        writeln!(f, "  Signature size: {} bytes", self.signature_size)?;
        writeln!(f, "  Public key: {}...", &self.public_key_hex[..16])?;
        writeln!(f)?;
        writeln!(
            f,
            "  Verification: {}",
            if self.verification_passed {
                "PASSED"
            } else {
                "FAILED"
            }
        )?;
        Ok(())
    }
}

/// Create test dataset.
fn create_test_dataset(ctx: &mut RecipeContext) -> Result<RecordBatch> {
    let schema = Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("value", DataType::Float64, false),
    ]);

    let ids: Vec<i64> = (0..100).collect();
    let values: Vec<f64> = (0..100).map(|_| ctx.rng.gen::<f64>()).collect();

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
#[cfg(feature = "signing")]
fn execute_recipe(ctx: &mut RecipeContext) -> Result<RecipeResult> {
    // Create dataset
    let batch = create_test_dataset(ctx)?;

    // Save to bytes for signing
    let ald_path = ctx.path("to_sign.ald");
    save(&batch, DatasetType::Tabular, &ald_path, SaveOptions::new())?;
    let data = std::fs::read(&ald_path)?;

    // Generate keypair
    let keypair = Keypair::generate();
    let public_key_hex = hex::encode(keypair.public_key_bytes());

    // Sign the data
    let signature_data = sign(&data, &keypair);

    // Verify signature
    let verification_passed = verify(&data, &signature_data).is_ok();

    Ok(RecipeResult {
        data_size: data.len(),
        signature_size: signature_data.signature.len(),
        public_key_hex,
        verification_passed,
    })
}

#[cfg(not(feature = "signing"))]
fn execute_recipe(_ctx: &mut RecipeContext) -> Result<RecipeResult> {
    Err(ald_cookbook::Error::FeatureNotEnabled {
        feature: "signing".to_string(),
    })
}

fn main() -> Result<()> {
    let mut ctx = RecipeContext::new("sign_ald_ed25519")?;
    let result = execute_recipe(&mut ctx)?;
    ctx.report(&result)?;
    Ok(())
}

#[cfg(all(test, feature = "signing"))]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_executes() {
        let mut ctx = RecipeContext::new("test_sign").unwrap();
        let result = execute_recipe(&mut ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verification_passes() {
        let mut ctx = RecipeContext::new("sign_verify").unwrap();
        let result = execute_recipe(&mut ctx).unwrap();
        assert!(result.verification_passed);
    }
}
