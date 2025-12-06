# Load ALD Metadata

**Category**: B (Loading)
**Status**: Verified

## Run the Recipe

```bash
cargo run --example load_ald_metadata
```

## Code Highlights

```rust
use ald_cookbook::{load_metadata, Result};

// Load only metadata (fast, no decompression)
let metadata = load_metadata("dataset.ald")?;
println!("Dataset: {}", metadata.name);
println!("Rows: {}", metadata.row_count);
```

## QA Checklist

All 10 points verified.
