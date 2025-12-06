# Load ALD Basic

**Category**: B (Loading)
**Status**: Verified

## Run the Recipe

```bash
cargo run --example load_ald_basic
```

## Code Highlights

```rust
use ald_cookbook::{load, Result};

let (batch, metadata) = load("dataset.ald")?;
println!("Loaded {} rows", batch.num_rows());
println!("Dataset: {}", metadata.name);
```

## QA Checklist

All 10 points verified.
