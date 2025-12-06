# Quick Start

Create and load your first ALD dataset in under 5 minutes.

## Step 1: Create a Dataset

```bash
cargo run --example create_ald_from_arrow
```

**Expected Output:**
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

## Step 2: Understand the Code

Every recipe follows this canonical structure:

```rust
use ald_cookbook::prelude::*;
use ald_cookbook::{RecipeContext, Result};

fn main() -> Result<()> {
    // 1. Create isolated context with deterministic RNG
    let ctx = RecipeContext::new("create_ald_from_arrow")?;

    // 2. Execute the recipe
    let result = execute_recipe(&ctx)?;

    // 3. Report results
    ctx.report(&result)?;

    // 4. Cleanup automatic via Drop
    Ok(())
}
```

## Step 3: Run Tests

```bash
# All tests
cargo test

# With verbose output
cargo test -- --nocapture

# Specific module
cargo test format::
```

## Step 4: Check Coverage

```bash
cargo llvm-cov
```

**Expected Output:**
```
Filename                      Regions    Missed  Cover   Lines  Missed   Cover
src/format.rs                    156        8   94.87%    776      35   95.49%
src/transforms.rs                 89        4   95.51%    532      22   95.86%
...
TOTAL                            892       39   95.63%   5979     245   95.90%
```

## Step 5: Explore More Recipes

### Dataset Creation (Category A)
```bash
cargo run --example create_ald_tabular
cargo run --example create_ald_timeseries
cargo run --example create_ald_text_corpus
cargo run --example create_ald_image_dataset
```

### Format Conversion (Category C)
```bash
cargo run --example convert_parquet_to_ald
cargo run --example convert_csv_to_ald
```

### Data Transforms (Category D)
```bash
cargo run --example transform_filter
cargo run --example transform_shuffle
cargo run --example transform_sample
```

### Drift Detection (Category F)
```bash
cargo run --example drift_ks_test
cargo run --example drift_psi
```

## Step 6: With Features

Some recipes require feature flags:

```bash
# Ed25519 signing
cargo run --example sign_ald_ed25519 --features signing

# All features
cargo test --features full
```

## The IIUR Guarantee

Every recipe you run is:

| Principle | Guarantee |
|-----------|-----------|
| **Isolated** | Uses temp directory; no files left behind |
| **Idempotent** | Same seed = same output, every time |
| **Useful** | Real production patterns, copy-paste ready |
| **Reproducible** | Works on Linux, macOS, WASM |

## Next Steps

- [Project Structure](./project-structure.md) - Understand the codebase layout
- [The ALD Format](../concepts/ald-format.md) - Deep dive into the format
- [Property-Based Testing](../concepts/property-based-testing.md) - Learn the testing methodology
