# Project Structure

## Directory Layout

```
ald-cookbook/
├── Cargo.toml              # Package manifest
├── Cargo.lock              # Pinned dependencies (reproducibility)
├── CLAUDE.md               # AI assistant guidelines
├── README.md               # Project overview
├── Makefile                # Build automation & TDD targets
│
├── src/                    # Core library
│   ├── lib.rs              # Public API exports
│   ├── prelude.rs          # Convenient re-exports
│   ├── error.rs            # Domain error types (thiserror)
│   ├── context.rs          # RecipeContext isolation primitives
│   ├── format.rs           # ALD format I/O
│   ├── transforms.rs       # Data transformations
│   ├── convert.rs          # Format conversion
│   ├── drift.rs            # Distribution drift detection
│   ├── federated.rs        # Federated learning splits
│   ├── quality.rs          # Data quality validation
│   ├── registry.rs         # Dataset registry
│   ├── signing.rs          # Ed25519 signatures (feature-gated)
│   ├── encryption.rs       # AES-256-GCM (feature-gated)
│   └── browser.rs          # WASM bindings (feature-gated)
│
├── examples/               # 22 production recipes
│   ├── dataset_creation/   # Category A (5 recipes)
│   ├── loading/            # Category B (2 recipes)
│   ├── conversion/         # Category C (2 recipes)
│   ├── transforms/         # Category D (3 recipes)
│   ├── quality/            # Category E (1 recipe)
│   ├── drift/              # Category F (2 recipes)
│   ├── federated/          # Category G (2 recipes)
│   ├── security/           # Category H (1 recipe)
│   ├── registry/           # Category I (2 recipes)
│   └── cli/                # Category L (1 recipe)
│
├── tests/                  # Property-based tests
│   ├── proptest_format.rs  # Format invariants
│   ├── proptest_transforms.rs  # Transform invariants
│   └── proptest_drift.rs   # Drift detection invariants
│
├── benches/                # Criterion benchmarks
│   └── format_benchmarks.rs
│
├── book/                   # mdBook documentation
│   ├── book.toml           # Book configuration
│   └── src/                # Markdown chapters
│
├── docs/                   # Additional documentation
│   ├── specifications/     # ALD format specification
│   └── qa/                 # QA checklists
│
└── .github/
    └── workflows/
        └── ci.yml          # CI pipeline
```

## Source Module Architecture

### Core Modules

| Module | LOC | Purpose |
|--------|-----|---------|
| `format.rs` | 776 | ALD file I/O, header parsing, checksums |
| `transforms.rs` | 532 | Filter, map, normalize, shuffle, sample |
| `quality.rs` | 695 | Null detection, outliers, duplicates |
| `drift.rs` | 600 | KS test, chi-square, PSI |
| `federated.rs` | 628 | IID, Dirichlet, stratified splits |
| `convert.rs` | 614 | Parquet, CSV, JSON conversion |
| `registry.rs` | 573 | Publish, pull, versioning |
| `signing.rs` | 355 | Ed25519 signatures |
| `encryption.rs` | 330 | AES-256-GCM encryption |
| `context.rs` | 273 | RecipeContext isolation |
| `browser.rs` | 268 | WASM bindings |
| `error.rs` | 217 | Error types |

### Public API (prelude.rs)

```rust
// Re-exported from Arrow
pub use arrow::array::*;
pub use arrow::datatypes::*;
pub use arrow::record_batch::RecordBatch;

// Core types
pub use crate::context::RecipeContext;
pub use crate::error::{Error, Result};
pub use crate::format::{load, save, DatasetType, SaveOptions};
```

## Recipe Categories

### A: Dataset Creation (5 recipes)

| Recipe | Description |
|--------|-------------|
| `create_ald_from_arrow` | Create ALD from Arrow RecordBatch |
| `create_ald_tabular` | Create structured tabular dataset |
| `create_ald_timeseries` | Create time series dataset |
| `create_ald_text_corpus` | Create text corpus dataset |
| `create_ald_image_dataset` | Create image dataset |

### B: Loading & Streaming (2 recipes)

| Recipe | Description |
|--------|-------------|
| `load_ald_basic` | Basic dataset loading |
| `load_ald_metadata` | Metadata-only loading |

### C: Format Conversion (2 recipes)

| Recipe | Description |
|--------|-------------|
| `convert_parquet_to_ald` | Parquet → ALD |
| `convert_csv_to_ald` | CSV → ALD |

### D: Data Transforms (3 recipes)

| Recipe | Description |
|--------|-------------|
| `transform_filter` | Row filtering |
| `transform_shuffle` | Deterministic shuffle |
| `transform_sample` | Random sampling |

### E: Data Quality (1 recipe)

| Recipe | Description |
|--------|-------------|
| `quality_null_detection` | Detect null values |

### F: Drift Detection (2 recipes)

| Recipe | Description |
|--------|-------------|
| `drift_ks_test` | Kolmogorov-Smirnov test |
| `drift_psi` | Population Stability Index |

### G: Federated Learning (2 recipes)

| Recipe | Description |
|--------|-------------|
| `federated_iid_split` | IID data partitioning |
| `federated_dirichlet_split` | Non-IID Dirichlet partitioning |

### H: Security (1 recipe)

| Recipe | Description |
|--------|-------------|
| `sign_ald_ed25519` | Ed25519 digital signatures |

### I: Registry (2 recipes)

| Recipe | Description |
|--------|-------------|
| `registry_publish` | Publish to registry |
| `registry_pull` | Pull from registry |

### L: CLI Tools (1 recipe)

| Recipe | Description |
|--------|-------------|
| `cli_ald_info` | Dataset inspection |

## Test Architecture

### Unit Tests (in-module)

Located in each source file with `#[cfg(test)]`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_specific_behavior() { ... }
}
```

### Property Tests (tests/)

Located in `tests/proptest_*.rs`:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn roundtrip_preserves_data(data in any::<Vec<u8>>()) {
        // Property: encode then decode = original
    }
}
```

### Integration Tests

Recipes themselves serve as integration tests, verified in CI.

## Next Steps

- [The ALD Format](../concepts/ald-format.md) - Binary format deep dive
- [IIUR Principles](../concepts/iiur-principles.md) - Recipe design principles
