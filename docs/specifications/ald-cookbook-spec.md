# ALD Cookbook: Isolated, Idempotent & Reproducible Dataset Recipes Specification

**Version**: 1.0.0
**Status**: DRAFT
**Author**: Sovereign AI Stack Team
**Date**: 2025-12-06
**MSRV**: 1.75
**Repository**: [github.com/paiml/ald-cookbook](https://github.com/paiml/ald-cookbook)

---

## Executive Summary

This specification defines the complete implementation of ALD Cookbook examples following the **IIUR Principles**: **Isolated**, **Idempotent**, **Useful**, and **Reproducible**. Each recipe is a self-contained example demonstrating dataset operations using the Alimentar Dataset Format (`.ald`) that can be executed independently with deterministic outcomes, regardless of prior state.

Guided by the **Toyota Production System (TPS)** principles, this cookbook eliminates the *Muda* (waste) of shared state, environmental dependencies, and non-deterministic behavior that plague traditional data engineering workflows [1, 2].

> **Theoretical Basis**: This specification is supported by 10 peer-reviewed citations (Section 6) covering Lean Manufacturing [1, 2], Technical Debt [3], Data Management [4-7], and Distributed Systems [8-10].

**Design Philosophy**: Each recipe is a *cell* in a lean production line—completely self-sufficient, producing consistent output every time, and ready for integration into larger data pipelines without side effects.

---

## Table of Contents

1. [IIUR Principles](#1-iiur-principles)
2. [Recipe Architecture](#2-recipe-architecture)
3. [Complete Recipe Catalog](#3-complete-recipe-catalog)
4. [Quality Gates & Testing Requirements](#4-quality-gates--testing-requirements)
5. [Implementation Guidelines](#5-implementation-guidelines)
6. [Peer-Reviewed Citations](#6-peer-reviewed-citations)
7. [Appendices](#7-appendices)
    * [Appendix D: Documentation Integration Strategy](#appendix-d-documentation-integration-strategy)

---

## 1. IIUR Principles

### 1.1 Isolated

> **Rationale**: Sculley et al. [3] identify "glue code" and "pipeline jungles" as massive sources of technical debt. Isolation ensures each recipe is a modular component, preventing these entanglements.

Each recipe MUST:

- **No shared mutable state**: No global variables, no shared filesystems, no persistent databases between runs
- **Self-contained dependencies**: All required datasets created inline or embedded via `include_bytes!()`
- **Temp directory isolation**: Any file I/O uses `tempfile::tempdir()` with automatic cleanup
- **Feature flag independence**: Recipes work with their declared features only; no implicit feature dependencies
- **Thread safety**: Concurrent execution of any two recipes produces identical results

```rust
// CORRECT: Isolated recipe
fn main() -> Result<()> {
    let temp = tempfile::tempdir()?;  // Ephemeral, isolated
    let dataset_path = temp.path().join("data.ald");
    // ... work within temp directory
    Ok(())  // temp directory automatically cleaned up
}

// INCORRECT: Shares state
static mut GLOBAL_DATASET: Option<Dataset> = None;  // Violates isolation
```

### 1.2 Idempotent

> **Rationale**: In TPS, "Standard Work" is the basis for improvement [1]. Idempotency provides this baseline stability for data operations.

Each recipe MUST:

- **f(f(x)) = f(x)**: Running a recipe twice produces identical output
- **No accumulation**: Repeated runs do not accumulate files, state, or side effects
- **Deterministic seeds**: Any randomness uses fixed seeds for reproducibility
- **Atomic operations**: Either fully succeeds or fully fails with no partial state

```rust
// CORRECT: Idempotent with deterministic seed
let rng = StdRng::seed_from_u64(42);
let dataset = generate_synthetic_data(&mut rng, 1000)?;

// INCORRECT: Non-deterministic
let dataset = generate_data()?;  // Uses thread_rng internally
```

### 1.3 Useful

Each recipe MUST:

- **Solve a real problem**: Addresses a concrete use case from production data workflows
- **Executable demonstration**: `cargo run --example <name>` produces meaningful output
- **Clear learning objective**: Single concept per recipe with explicit takeaway
- **Copy-paste ready**: Code can be directly adapted for production use

### 1.4 Reproducible

Each recipe MUST:

- **Pinned dependencies**: Uses exact versions from workspace `Cargo.lock`
- **Cross-platform**: Works on x86_64 Linux, aarch64 Linux, aarch64 macOS, WASM
- **CI-verified**: All recipes run in CI on every commit
- **Documented environment**: Clearly states any system requirements

---

## 2. Recipe Architecture

### 2.1 Standard Recipe Structure

Every recipe follows this canonical structure and MUST include the **10-Point QA Checklist** in its documentation block.

```
examples/
└── category/
    └── recipe_name.rs
```

Each recipe file:

```rust
//! # Recipe: [Descriptive Title]
//!
//! **Category**: [Category Name]
//! **Isolation Level**: Full
//! **Idempotency**: Guaranteed
//! **Dependencies**: [List feature flags required]
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
//! [One sentence describing what this recipe teaches]
//!
//! ## Run Command
//! ```bash
//! cargo run --example recipe_name [--features feature1,feature2]
//! ```

use ald_cookbook::prelude::*;

/// Recipe entry point - isolated and idempotent
fn main() -> ald_cookbook::Result<()> {
    // 1. Setup: Create isolated environment
    let ctx = RecipeContext::new("recipe_name")?;

    // 2. Execute: Perform the recipe's core logic
    let result = execute_recipe(&ctx)?;

    // 3. Report: Display results to user
    ctx.report(&result)?;

    // 4. Cleanup: Automatic via Drop
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_idempotent() {
        let result1 = main();
        let result2 = main();
        assert_eq!(result1.is_ok(), result2.is_ok());
    }

    #[test]
    fn test_recipe_isolated() {
        // Verify no side effects persist
    }
}
```

### 2.2 RecipeContext Utility

Provides standardized isolation primitives:

```rust
pub struct RecipeContext {
    /// Isolated temporary directory (auto-cleanup on drop)
    pub temp_dir: TempDir,
    /// Deterministic RNG seeded by recipe name
    pub rng: StdRng,
    /// Recipe metadata for reporting
    pub metadata: RecipeMetadata,
}

impl RecipeContext {
    pub fn new(name: &str) -> Result<Self> {
        let seed = hash_name_to_seed(name);
        Ok(Self {
            temp_dir: tempfile::tempdir()?,
            rng: StdRng::seed_from_u64(seed),
            metadata: RecipeMetadata::from_name(name),
        })
    }

    pub fn path(&self, filename: &str) -> PathBuf {
        self.temp_dir.path().join(filename)
    }
}
```

### 2.3 ALD Format Overview

> **Rationale**: The format design leverages columnar storage principles [6] and zero-copy IPC standards [8] to minimize *Muda* (waste) in serialization/deserialization.

The Alimentar Dataset Format (`.ald`) provides secure, verifiable dataset distribution:

```text
┌─────────────────────────────────────────┐
│ Header (32 bytes, fixed)                │
│   Magic: "ALDF" (0x414C4446)            │
│   Version: 1.2                          │
│   Flags: encryption, signing, streaming │
├─────────────────────────────────────────┤
│ Metadata (variable, MessagePack)        │
├─────────────────────────────────────────┤
│ Schema (variable, Arrow IPC)            │
├─────────────────────────────────────────┤
│ Payload (variable, Arrow IPC + zstd)    │
├─────────────────────────────────────────┤
│ Checksum (4 bytes, CRC32)               │
└─────────────────────────────────────────┘
```

### 2.4 Test Harness Requirements

Every recipe includes:

| Test Type | Requirement | Coverage |
|-----------|-------------|----------|
| Unit Tests | Core logic verification | 95% minimum |
| Idempotency Test | `main(); main();` produces same result | Required |
| Isolation Test | No filesystem leaks after run | Required |
| Property Tests | Proptest for input variations | 3+ properties |
| Doc Tests | All code examples compile | Required |

---

## 3. Complete Recipe Catalog

### Category A: Dataset Creation

#### A.1 `create_ald_from_arrow`
**Objective**: Create a `.ald` dataset from Arrow RecordBatches.

```rust
//! Create .ald dataset from Arrow RecordBatches
//! Run: cargo run --example create_ald_from_arrow

fn main() -> Result<()> {
    let ctx = RecipeContext::new("create_ald_from_arrow")?;

    // Create Arrow schema
    let schema = Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("value", DataType::Float64, false),
        Field::new("label", DataType::Utf8, true),
    ]);

    // Generate synthetic data
    let batch = create_record_batch(&schema, &mut ctx.rng, 1000)?;

    // Save to .ald format
    let ald_path = ctx.path("synthetic.ald");
    alimentar::format::save(&batch, DatasetType::Tabular, &ald_path, SaveOptions::default())?;

    // Verify roundtrip
    let loaded = alimentar::format::load(&ald_path)?;
    assert_eq!(batch.num_rows(), loaded.num_rows());

    println!("Created .ald dataset: {} bytes", std::fs::metadata(&ald_path)?.len());
    Ok(())
}
```

**Tests**:
- `test_creates_valid_ald_header`
- `test_schema_preserved_exactly`
- `test_data_roundtrip`
- `proptest_random_schemas`

#### A.2 `create_ald_tabular`
**Objective**: Create tabular dataset from CSV-like structure.

#### A.3 `create_ald_timeseries`
**Objective**: Create time-series dataset with temporal indexing.

#### A.4 `create_ald_text_corpus`
**Objective**: Create NLP text corpus dataset.

#### A.5 `create_ald_image_dataset`
**Objective**: Create image classification dataset.

---

### Category B: Dataset Loading & Streaming

#### B.1 `load_ald_zero_copy`
**Objective**: Load `.ald` with zero-copy memory mapping.

```rust
//! Zero-copy loading via memory mapping
//! Run: cargo run --example load_ald_zero_copy

fn main() -> Result<()> {
    let ctx = RecipeContext::new("load_ald_zero_copy")?;

    // Create test dataset
    let dataset = create_demo_dataset(&mut ctx.rng, 100_000)?;
    let ald_path = ctx.path("large.ald");
    alimentar::format::save(&dataset, DatasetType::Tabular, &ald_path, SaveOptions::default())?;

    // Memory-mapped load (zero-copy)
    let mmap = alimentar::mmap::MmapDataset::open(&ald_path)?;

    println!("Dataset loaded via mmap:");
    println!("  Rows: {}", mmap.num_rows());
    println!("  Memory footprint: {} KB (mapped, not copied)", mmap.resident_size()? / 1024);

    // Iterate without loading full dataset
    for batch in mmap.iter_batches(1024) {
        println!("  Batch: {} rows", batch?.num_rows());
    }

    Ok(())
}
```

#### B.2 `load_ald_streaming`
**Objective**: Stream large datasets chunk-by-chunk.

#### B.3 `load_ald_lazy`
**Objective**: Lazy loading with async prefetch.

#### B.4 `load_ald_parallel`
**Objective**: Parallel batch loading across threads.

---

### Category C: Format Conversion

#### C.1 `convert_parquet_to_ald`
**Objective**: Convert Parquet files to `.ald` format.

```rust
//! Convert Parquet to ALD format
//! Run: cargo run --example convert_parquet_to_ald

fn main() -> Result<()> {
    let ctx = RecipeContext::new("convert_parquet_to_ald")?;

    // Create mock Parquet file
    let parquet_path = ctx.path("source.parquet");
    create_demo_parquet(&parquet_path, &mut ctx.rng)?;

    // Convert to ALD
    let ald_path = ctx.path("converted.ald");
    alimentar::convert::parquet_to_ald(&parquet_path, &ald_path, ConvertOptions::default())?;

    // Verify
    let original_rows = count_parquet_rows(&parquet_path)?;
    let converted = alimentar::format::load(&ald_path)?;

    println!("Converted Parquet to ALD:");
    println!("  Original: {} rows", original_rows);
    println!("  Converted: {} rows", converted.num_rows());
    println!("  Size: {} KB", std::fs::metadata(&ald_path)?.len() / 1024);

    Ok(())
}
```

#### C.2 `convert_csv_to_ald`
**Objective**: Convert CSV files with schema inference.

#### C.3 `convert_ald_to_parquet`
**Objective**: Export `.ald` to Parquet format.

#### C.4 `convert_hf_to_ald`
**Objective**: Convert HuggingFace datasets to `.ald`.

#### C.5 `convert_jsonl_to_ald`
**Objective**: Convert JSON Lines to `.ald` format.

---

### Category D: Data Transforms

#### D.1 `transform_filter`
**Objective**: Filter rows by predicate.

```rust
//! Filter dataset rows by predicate
//! Run: cargo run --example transform_filter

fn main() -> Result<()> {
    let ctx = RecipeContext::new("transform_filter")?;

    // Load dataset
    let dataset = create_demo_dataset(&mut ctx.rng, 10_000)?;

    // Filter: keep rows where value > 0.5
    let filtered = dataset.filter(|row| row.get_f64("value")? > 0.5)?;

    println!("Filter transform:");
    println!("  Original: {} rows", dataset.num_rows());
    println!("  Filtered: {} rows", filtered.num_rows());
    println!("  Reduction: {:.1}%",
        (1.0 - filtered.num_rows() as f64 / dataset.num_rows() as f64) * 100.0);

    Ok(())
}
```

#### D.2 `transform_map`
**Objective**: Apply transformation to columns.

#### D.3 `transform_shuffle`
**Objective**: Deterministic shuffle with seed.

#### D.4 `transform_sample`
**Objective**: Random sampling with stratification.

#### D.5 `transform_normalize`
**Objective**: Feature normalization (z-score, min-max).

---

### Category E: Data Quality

#### E.1 `quality_null_detection`
**Objective**: Detect and report null values.

```rust
//! Detect null values in dataset
//! Run: cargo run --example quality_null_detection

fn main() -> Result<()> {
    let ctx = RecipeContext::new("quality_null_detection")?;

    // Create dataset with nulls
    let dataset = create_dataset_with_nulls(&mut ctx.rng)?;

    // Analyze null patterns
    let report = alimentar::quality::null_report(&dataset)?;

    println!("Null Detection Report:");
    println!("{:-<50}", "");
    for (column, stats) in &report.columns {
        println!("  {}: {}/{} nulls ({:.1}%)",
            column, stats.null_count, stats.total_count,
            stats.null_percentage());
    }
    println!("{:-<50}", "");
    println!("Total: {:.1}% null values", report.overall_null_percentage());

    Ok(())
}
```

#### E.2 `quality_duplicate_detection`
**Objective**: Find duplicate rows or near-duplicates.

#### E.3 `quality_outlier_detection`
**Objective**: Statistical outlier detection.

#### E.4 `quality_schema_validation`
**Objective**: Validate data against schema constraints.

---

### Category F: Drift Detection

#### F.1 `drift_ks_test`
**Objective**: Kolmogorov-Smirnov test for distribution drift.
> **Context**: Addresses "Data Cascades" [5] by detecting upstream data changes before they corrupt model training.

```rust
//! Detect distribution drift using KS test
//! Run: cargo run --example drift_ks_test

fn main() -> Result<()> {
    let ctx = RecipeContext::new("drift_ks_test")?;

    // Create reference and current datasets
    let reference = create_reference_dataset(&mut ctx.rng)?;
    let current = create_drifted_dataset(&mut ctx.rng, 0.3)?; // 30% drift

    // Run KS test
    let result = alimentar::drift::ks_test(&reference, &current, "value")?;

    println!("Kolmogorov-Smirnov Drift Test:");
    println!("  Statistic: {:.4}", result.statistic);
    println!("  P-value: {:.4}", result.p_value);
    println!("  Drift detected: {}", result.p_value < 0.05);

    Ok(())
}
```

#### F.2 `drift_chi_square`
**Objective**: Chi-square test for categorical drift.

#### F.3 `drift_psi`
**Objective**: Population Stability Index calculation.

#### F.4 `drift_monitoring`
**Objective**: Continuous drift monitoring pipeline.

---

### Category G: Federated Learning Splits

#### G.1 `federated_iid_split`
**Objective**: IID split for federated learning.

```rust
//! IID split for federated learning simulation
//! Run: cargo run --example federated_iid_split

fn main() -> Result<()> {
    let ctx = RecipeContext::new("federated_iid_split")?;

    let dataset = create_demo_dataset(&mut ctx.rng, 10_000)?;

    // Split into 5 clients, IID distribution
    let splits = alimentar::federated::iid_split(&dataset, 5, &mut ctx.rng)?;

    println!("Federated IID Split (5 clients):");
    for (i, split) in splits.iter().enumerate() {
        println!("  Client {}: {} rows", i, split.num_rows());
    }

    Ok(())
}
```

#### G.2 `federated_non_iid_split`
**Objective**: Non-IID split with label skew.

#### G.3 `federated_stratified_split`
**Objective**: Stratified split maintaining class balance.

#### G.4 `federated_dirichlet_split`
**Objective**: Dirichlet-based heterogeneous split.

---

### Category H: Security & Encryption

#### H.1 `encrypt_ald_aes256`
**Objective**: Encrypt dataset with AES-256-GCM.

```rust
//! Encrypt dataset with AES-256-GCM
//! Run: cargo run --example encrypt_ald_aes256 --features encryption

fn main() -> Result<()> {
    let ctx = RecipeContext::new("encrypt_ald_aes256")?;

    let dataset = create_demo_dataset(&mut ctx.rng, 1000)?;

    // Encrypt with password-derived key (Argon2id KDF)
    let encrypted_path = ctx.path("encrypted.ald");
    let opts = SaveOptions::default()
        .with_encryption(EncryptionOptions::password("secret-key-123"));
    alimentar::format::save(&dataset, DatasetType::Tabular, &encrypted_path, opts)?;

    // Decrypt and verify
    let decrypted = alimentar::format::load_encrypted(&encrypted_path, "secret-key-123")?;
    assert_eq!(dataset.num_rows(), decrypted.num_rows());

    println!("Encrypted dataset:");
    println!("  Size: {} bytes", std::fs::metadata(&encrypted_path)?.len());
    println!("  Cipher: AES-256-GCM");
    println!("  KDF: Argon2id");

    Ok(())
}
```

#### H.2 `sign_ald_ed25519`
**Objective**: Sign dataset with Ed25519.

#### H.3 `license_ald_commercial`
**Objective**: Attach commercial license block.

#### H.4 `verify_ald_integrity`
**Objective**: Verify checksum and signatures.

---

### Category I: Registry & Distribution

#### I.1 `registry_publish`
**Objective**: Publish dataset to local registry.

```rust
//! Publish dataset to local registry
//! Run: cargo run --example registry_publish

fn main() -> Result<()> {
    let ctx = RecipeContext::new("registry_publish")?;

    // Create isolated registry
    let registry_path = ctx.path("registry");
    let registry = alimentar::registry::Registry::new(&registry_path)?;

    // Create and publish dataset
    let dataset = create_demo_dataset(&mut ctx.rng, 1000)?;
    registry.publish(
        "demo-dataset",
        &dataset,
        PublishOptions {
            version: "1.0.0".into(),
            description: "Demo dataset for testing".into(),
            license: License::MIT,
            ..Default::default()
        },
    )?;

    println!("Published dataset: demo-dataset@1.0.0");
    println!("Registry location: {:?}", registry_path);

    Ok(())
}
```

#### I.2 `registry_pull`
**Objective**: Pull dataset from registry.

#### I.3 `registry_version`
**Objective**: Dataset versioning and history.

#### I.4 `registry_s3_backend`
**Objective**: S3-compatible storage backend.

---

### Category J: Built-in Datasets

#### J.1 `builtin_mnist`
**Objective**: Load MNIST digit dataset.

```rust
//! Load MNIST dataset
//! Run: cargo run --example builtin_mnist

fn main() -> Result<()> {
    let ctx = RecipeContext::new("builtin_mnist")?;

    // Load MNIST (cached automatically)
    let mnist = alimentar::datasets::mnist()?;

    println!("MNIST Dataset:");
    println!("  Training samples: {}", mnist.train.num_rows());
    println!("  Test samples: {}", mnist.test.num_rows());
    println!("  Image shape: 28x28");
    println!("  Classes: 0-9");

    // Save as .ald for offline use
    let ald_path = ctx.path("mnist.ald");
    alimentar::format::save(&mnist.train, DatasetType::ImageClassification, &ald_path, SaveOptions::default())?;

    Ok(())
}
```

#### J.2 `builtin_fashion_mnist`
**Objective**: Load Fashion-MNIST dataset.

#### J.3 `builtin_cifar10`
**Objective**: Load CIFAR-10 dataset.

#### J.4 `builtin_iris`
**Objective**: Load Iris classification dataset.

---

### Category K: WASM & Browser

#### K.1 `wasm_dataset_viewer`
**Objective**: View `.ald` datasets in browser.

```rust
//! WASM dataset viewer
//! Build: cargo build --example wasm_dataset_viewer --target wasm32-unknown-unknown --features browser

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct DatasetViewer {
    dataset: Dataset,
}

#[wasm_bindgen]
impl DatasetViewer {
    #[wasm_bindgen(constructor)]
    pub fn from_bytes(bytes: &[u8]) -> Result<DatasetViewer, JsValue> {
        let dataset = alimentar::format::load_from_bytes(bytes)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(Self { dataset })
    }

    #[wasm_bindgen]
    pub fn num_rows(&self) -> usize {
        self.dataset.num_rows()
    }

    #[wasm_bindgen]
    pub fn schema_json(&self) -> String {
        serde_json::to_string(&self.dataset.schema()).unwrap_or_default()
    }
}
```

#### K.2 `wasm_data_transform`
**Objective**: Client-side data transformations.

#### K.3 `wasm_visualization`
**Objective**: Data visualization dashboard.

#### K.4 `wasm_quality_check`
**Objective**: Browser-based quality analysis.

---

### Category L: CLI Tools

#### L.1 `cli_ald_info`
**Objective**: Inspect `.ald` dataset metadata.

#### L.2 `cli_ald_convert`
**Objective**: Format conversion CLI.

#### L.3 `cli_ald_validate`
**Objective**: Validate `.ald` integrity.

#### L.4 `cli_ald_schema`
**Objective**: Print/modify dataset schema.

---

## 4. Quality Gates & Testing Requirements

### 4.1 Coverage Requirements

| Metric | Target | Enforcement |
|--------|--------|-------------|
| Line Coverage | 95% | `cargo llvm-cov --fail-under 95` |
| Branch Coverage | 90% | `cargo llvm-cov --branch` |
| Mutation Score | 80% | `cargo mutants` |
| Property Tests | 3+ per recipe | proptest |

### 4.2 PMAT Integration

```toml
# .pmat/tdg-rules.toml
[quality_gates]
rust_min_grade = "A"
max_score_drop = 3.0
mode = "strict"

[thresholds]
test_coverage = 95
mutation_score = 80
cyclomatic_complexity = 10

[quality_gate_exceptions]
# Cookbook-specific exceptions (pedagogical patterns)
entropy_violations = 10
documentation_strict = false
```

### 4.3 Automated QA Checklist (PMAT)

1.  **Execution Success**: `cargo run --example <name>` must exit with code 0.
2.  **Test Pass Rate**: `cargo test --example <name>` must pass all tests.
3.  **Lint Compliance**: `cargo clippy --example <name>` must return 0 warnings.
4.  **Style Compliance**: `cargo fmt --check` must pass.
5.  **Deterministic Output**: Two sequential runs must produce identical artifacts.
6.  **Resource Isolation**: No temp files leaked after execution.
7.  **Proptest Coverage**: At least 3 distinct property tests executed.
8.  **Code Coverage**: Line coverage must exceed 95%.
9.  **Mutation Robustness**: `cargo mutants` score must exceed 80%.
10. **Documentation Standards**: Doc comments must contain "Run Command" and "Learning Objective".

---

## 5. Implementation Guidelines

### 5.1 Toyota Way Compliance

Each recipe MUST embody:

| Principle | Implementation |
|-----------|----------------|
| **Jidoka** (Built-in Quality) | Type-safe errors, compile-time validation, property tests [1, 9] |
| **Muda** (Waste Elimination) | No unnecessary dependencies, minimal allocations, zero-copy where possible [2] |
| **Heijunka** (Level Loading) | Consistent recipe structure, predictable resource usage [1] |
| **Kaizen** (Continuous Improvement) | Benchmarks for every recipe, performance regression tests [2] |
| **Genchi Genbutsu** (Go and See) | Observable metrics, clear output, no hidden side effects [1] |
| **Poka-Yoke** (Error-Proofing) | Impossible states unrepresentable, exhaustive pattern matching [1] |

### 5.2 Error Handling

```rust
// Use thiserror for domain errors
#[derive(Debug, thiserror::Error)]
pub enum RecipeError {
    #[error("Dataset not found: {0}")]
    DatasetNotFound(PathBuf),

    #[error("Invalid format: expected {expected}, got {actual}")]
    InvalidFormat { expected: String, actual: String },

    #[error("Schema mismatch: {0}")]
    SchemaMismatch(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, RecipeError>;
```

---

## 6. Peer-Reviewed Citations

### [1] Ohno, T. (1988). Toyota Production System: Beyond Large-Scale Production
*Productivity Press. ISBN 978-0915299140*

The foundational text on lean manufacturing. Our IIUR principles directly map to TPS concepts: Isolation→Autonomation (Jidoka), Idempotency→Standard Work, Reproducibility→Heijunka.

### [2] Womack, J.P. & Jones, D.T. (1996). Lean Thinking: Banish Waste and Create Wealth
*Simon & Schuster. ISBN 978-0743249270*

Defines the five lean principles (Value, Value Stream, Flow, Pull, Perfection) adapted here for data engineering workflows.

### [3] Sculley, D. et al. (2015). Hidden Technical Debt in Machine Learning Systems
*NIPS 2015. https://papers.nips.cc/paper/5656-hidden-technical-debt-in-machine-learning-systems*

Identifies ML-specific anti-patterns (glue code, pipeline jungles, dead experimental codepaths) that our isolated recipe pattern explicitly prevents.

### [4] Polyzotis, N. et al. (2017). Data Management Challenges in Production Machine Learning
*SIGMOD 2017. https://doi.org/10.1145/3035918.3054782*

Google's study on data management in ML systems. Our data quality and drift detection recipes address identified challenges.

### [5] Sambasivan, N. et al. (2021). "Everyone wants to do the model work, not the data work": Data Cascades in High-Stakes AI
*CHI 2021. https://doi.org/10.1145/3411764.3445518*

Documents how data quality issues cascade through ML systems. Our quality recipes provide systematic detection.

### [6] Kleppmann, M. (2017). Designing Data-Intensive Applications
*O'Reilly Media. ISBN 978-1449373320*

Chapters 3-4 inform our format design and encoding choices. Chapter 10 informs our batch processing recipes.

### [7] Armbrust, M. et al. (2015). Spark SQL: Relational Data Processing in Spark
*SIGMOD 2015. https://doi.org/10.1145/2723372.2742797*

Informs our columnar format design and query optimization strategies.

### [8] Apache Arrow Specification (2024)
*https://arrow.apache.org/docs/format/Columnar.html*

The Arrow IPC format specification that underlies our `.ald` payload encoding.

### [9] Claessen, K. & Hughes, J. (2000). QuickCheck: A Lightweight Tool for Random Testing
*ICFP 2000. https://doi.org/10.1145/351240.351266*

Foundational property-based testing paper. Our proptest requirements follow this methodology.

### [10] Li, T. et al. (2020). Federated Learning: Challenges, Methods, and Future Directions
*IEEE Signal Processing Magazine. https://doi.org/10.1109/MSP.2020.2975749*

Survey on federated learning challenges. Informs our federated split recipes and non-IID handling.

---

## 7. Appendices

### A. Recipe Dependency Matrix

| Recipe | alimentar | arrow | parquet | serde |
|--------|-----------|-------|---------|-------|
| A.1-A.5 | Required | Required | - | Required |
| B.1-B.4 | Required | Required | - | - |
| C.1-C.5 | Required | Required | Required | Required |
| D.1-D.5 | Required | Required | - | - |
| E.1-E.4 | Required | Required | - | - |
| F.1-F.4 | Required | Required | - | - |
| G.1-G.4 | Required | Required | - | - |
| H.1-H.4 | Required | - | - | Required |
| I.1-I.4 | Required | Required | - | Required |
| J.1-J.4 | Required | Required | - | - |
| K.1-K.4 | Required | - | - | Required |
| L.1-L.4 | Required | Required | - | Required |

### B. Feature Flag Matrix

| Feature | Description | Recipes |
|---------|-------------|---------|
| `default` | Core functionality | A.*, B.*, C.1-C.3, D.*, L.1-L.2 |
| `encryption` | AES-256-GCM | H.1, H.4 |
| `signing` | Ed25519 signatures | H.2, H.4 |
| `streaming` | Chunked loading | B.2-B.4 |
| `browser` | WASM target | K.* |
| `hf-hub` | HuggingFace integration | C.4, J.* |
| `s3` | S3 backend | I.4 |
| `full` | All features | All recipes |

### C. Checklist: Recipe Compliance

Before submitting a recipe, verify:

- [ ] **Isolation**: Uses `tempfile::tempdir()` for all file I/O
- [ ] **Isolation**: No global/static mutable state
- [ ] **Idempotency**: Fixed RNG seed via `RecipeContext`
- [ ] **Idempotency**: Running twice produces identical output
- [ ] **Useful**: Addresses real production use case
- [ ] **Useful**: Copy-paste ready code
- [ ] **Reproducible**: Works on Linux, macOS, WASM
- [ ] **Reproducible**: Pinned dependency versions
- [ ] **Testing**: 95%+ line coverage
- [ ] **Testing**: 3+ proptest properties
- [ ] **Testing**: Idempotency test present
- [ ] **Testing**: Isolation test present
- [ ] **Documentation**: Module doc with run command
- [ ] **Documentation**: Learning objective stated
- [ ] **Toyota Way**: No unnecessary abstraction (Muda)
- [ ] **Toyota Way**: Error handling via types (Jidoka)
- [ ] **PMAT**: 10-point QA checklist included and verified

### D. Documentation Integration Strategy

To ensure documentation never drifts from code (a form of *Muda*), all recipes are integrated into the `mdbook` documentation using `mdbook-include`. This enforces the **Genchi Genbutsu** principle—the documentation shows the actual, working code.

**Pattern**:
```markdown
# Recipe: Create ALD from Arrow

This recipe demonstrates how to create a dataset from Arrow RecordBatches.

{{#include ../../../examples/dataset_creation/create_ald_from_arrow.rs:10:}}
```

**Requirements**:
1.  **Source of Truth**: The Rust example file is the single source of truth.
2.  **Verified Code**: Only code that passes CI is included in documentation.
3.  **Context**: Use line ranges (e.g., `:10:`) to skip license headers.
4.  **Automation**: Documentation updates automatically on build.

---

## Approval

**Status**: DRAFT - AWAITING REVIEW

| Role | Name | Date | Signature |
|------|------|------|-----------|
| Author | Sovereign AI Stack Team | 2025-12-06 | ✓ |
| QA Lead | - | - | PENDING |
| Tech Lead | - | - | PENDING |

---

*This specification follows Toyota Production System principles for lean, efficient, and high-quality data engineering. All recipes must pass quality gates before implementation.*
