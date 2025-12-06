# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

ALD Cookbook is a collection of self-contained example recipes demonstrating the Alimentar Dataset Format (`.ald`) - a secure, verifiable dataset distribution format built on Arrow IPC with zstd compression. Each recipe follows **IIUR Principles**: Isolated, Idempotent, Useful, and Reproducible.

## Build Commands

```bash
# Run a recipe
cargo run --example <recipe_name>

# Run with feature flags (some recipes require specific features)
cargo run --example encrypt_ald_aes256 --features encryption

# Run tests
cargo test

# Run tests for a specific example
cargo test --example <recipe_name>

# Check code quality
cargo clippy -- -D warnings
cargo fmt --check

# Check coverage (95% minimum required)
cargo llvm-cov --fail-under 95

# Mutation testing (80% score required)
cargo mutants

# Build for WASM
cargo build --example <recipe_name> --target wasm32-unknown-unknown --features browser
```

## Architecture

### Recipe Structure

All recipes follow a canonical structure in `examples/<category>/<recipe_name>.rs`:

```rust
fn main() -> Result<()> {
    let ctx = RecipeContext::new("recipe_name")?;  // Isolated temp dir + deterministic RNG
    let result = execute_recipe(&ctx)?;
    ctx.report(&result)?;
    Ok(())  // Cleanup automatic via Drop
}
```

### IIUR Compliance Requirements

1. **Isolated**: Use `tempfile::tempdir()` for all file I/O, no global/static mutable state
2. **Idempotent**: Fixed RNG seeds via `RecipeContext`, running twice produces identical output
3. **Useful**: Real production use cases, copy-paste ready code
4. **Reproducible**: Works on Linux, macOS, WASM; pinned dependencies

### ALD Format Structure

```
Header (32 bytes) → Metadata (MessagePack) → Schema (Arrow IPC) → Payload (Arrow IPC + zstd) → Checksum (CRC32)
```

### Recipe Categories

- **A**: Dataset Creation (create_ald_from_arrow, create_ald_tabular, etc.)
- **B**: Loading & Streaming (load_ald_zero_copy, load_ald_streaming, etc.)
- **C**: Format Conversion (convert_parquet_to_ald, convert_csv_to_ald, etc.)
- **D**: Data Transforms (filter, map, shuffle, sample, normalize)
- **E**: Data Quality (null detection, duplicates, outliers, schema validation)
- **F**: Drift Detection (KS test, chi-square, PSI, monitoring)
- **G**: Federated Learning Splits (IID, non-IID, stratified, Dirichlet)
- **H**: Security & Encryption (AES-256-GCM, Ed25519 signing, licensing)
- **I**: Registry & Distribution (publish, pull, versioning, S3 backend)
- **J**: Built-in Datasets (MNIST, Fashion-MNIST, CIFAR-10, Iris)
- **K**: WASM & Browser (viewer, transforms, visualization)
- **L**: CLI Tools (info, convert, validate, schema)

### Feature Flags

| Feature | Description |
|---------|-------------|
| `default` | Core functionality |
| `encryption` | AES-256-GCM encryption |
| `signing` | Ed25519 signatures |
| `streaming` | Chunked loading |
| `browser` | WASM target |
| `hf-hub` | HuggingFace integration |
| `s3` | S3 backend |
| `full` | All features |

## Quality Requirements

- 95% minimum line coverage
- 90% minimum branch coverage
- 80% minimum mutation score
- 3+ proptest properties per recipe
- No `unwrap()` in recipe logic
- Each recipe must include a 10-point QA checklist in its doc comment
- Use `thiserror` for domain errors

## Minimum Supported Rust Version

MSRV: 1.75
