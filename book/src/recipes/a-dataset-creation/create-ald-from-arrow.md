# Create ALD from Arrow

**Category**: A (Dataset Creation)
**Status**: Verified
**Isolation**: Full
**Idempotency**: Guaranteed

## Overview

Create an ALD dataset directly from an Apache Arrow RecordBatch. This is the foundational recipe for all ALD creation.

## Prerequisites

- Understanding of Arrow data types
- Basic Rust error handling

## Run the Recipe

```bash
cargo run --example create_ald_from_arrow
```

**Expected Output**:
```
Recipe: create_ald_from_arrow
Seed: 17407780358490817044
--------------------------------------------------
Created ALD dataset from Arrow RecordBatch
  Rows: 1000
  File size: 13542 bytes
  Path: "/tmp/.tmpXHblvZ/synthetic.ald"
--------------------------------------------------
```

## Code

```rust
use ald_cookbook::prelude::*;
use ald_cookbook::{save, RecipeContext, Result, SaveOptions};
use arrow::array::{Float64Array, Int64Array};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use std::sync::Arc;

fn main() -> Result<()> {
    let mut ctx = RecipeContext::new("create_ald_from_arrow")?;

    // Generate deterministic synthetic data
    let rows = 1000;
    let ids: Vec<i64> = (0..rows).collect();
    let values: Vec<f64> = (0..rows)
        .map(|_| ctx.rng().gen_range(0.0..100.0))
        .collect();

    // Create Arrow schema
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("value", DataType::Float64, false),
    ]));

    // Create RecordBatch
    let batch = RecordBatch::try_new(
        schema,
        vec![
            Arc::new(Int64Array::from(ids)),
            Arc::new(Float64Array::from(values)),
        ],
    )?;

    // Save as ALD
    let path = ctx.temp_path("synthetic.ald");
    let options = SaveOptions::new("synthetic")
        .with_description("Synthetic dataset from Arrow");

    save(&batch, &path, options)?;

    // Report results
    let file_size = std::fs::metadata(&path)?.len();
    ctx.report(&format!(
        "Created ALD dataset from Arrow RecordBatch\n  \
         Rows: {}\n  \
         File size: {} bytes\n  \
         Path: {:?}",
        batch.num_rows(),
        file_size,
        path
    ))?;

    Ok(())
}
```

## Key Concepts

### RecipeContext

The `RecipeContext` provides:
- **Isolated temp directory**: All files created in temporary space
- **Deterministic RNG**: Same seed produces same output
- **Automatic cleanup**: Temp directory removed on drop

### Arrow Schema

Define column types with Arrow's type system:

```rust
let schema = Schema::new(vec![
    Field::new("id", DataType::Int64, false),      // Non-nullable
    Field::new("value", DataType::Float64, true),  // Nullable
]);
```

### SaveOptions

Configure metadata and compression:

```rust
let options = SaveOptions::new("dataset_name")
    .with_description("Human-readable description")
    .with_dataset_type(DatasetType::Tabular)
    .with_compression_level(3);
```

## Property Tests

```rust
proptest! {
    #[test]
    fn roundtrip_preserves_row_count(rows in 1..10000usize) {
        let batch = generate_batch(rows);
        let path = temp_path();

        save(&batch, &path, SaveOptions::default())?;
        let (loaded, _) = load(&path)?;

        prop_assert_eq!(batch.num_rows(), loaded.num_rows());
    }
}
```

## QA Checklist

| # | Check | Status |
|---|-------|--------|
| 1 | `cargo run` succeeds (Exit Code 0) | Pass |
| 2 | `cargo test` passes | Pass |
| 3 | Deterministic output (run twice, same result) | Pass |
| 4 | No temp files leaked after execution | Pass |
| 5 | Memory usage stable (no leaks) | Pass |
| 6 | Platform independent (Linux, macOS) | Pass |
| 7 | Clippy clean (zero warnings) | Pass |
| 8 | Rustfmt standard | Pass |
| 9 | No `unwrap()` in recipe logic | Pass |
| 10 | Property tests pass (50+ cases) | Pass |

## Common Errors

### Schema Mismatch

```rust
// ERROR: Array length doesn't match schema
let batch = RecordBatch::try_new(
    schema,  // expects 2 columns
    vec![Arc::new(Int64Array::from(vec![1, 2, 3]))],  // only 1 column
)?;
```

### Type Mismatch

```rust
// ERROR: Field expects Int64, got Float64
Field::new("id", DataType::Int64, false)
// but array is Float64Array
```

## Next Steps

- [Create Tabular Dataset](./create-ald-tabular.md) - Multi-type columns
- [Create TimeSeries Dataset](./create-ald-timeseries.md) - Temporal data
