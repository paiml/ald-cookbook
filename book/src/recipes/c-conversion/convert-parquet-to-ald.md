# Convert Parquet to ALD

**Category**: C (Conversion)
**Status**: Verified

## Run the Recipe

```bash
cargo run --example convert_parquet_to_ald
```

## Code Highlights

```rust
use ald_cookbook::convert::parquet_to_ald;

parquet_to_ald("input.parquet", "output.ald", SaveOptions::default())?;
```

## QA Checklist

All 10 points verified.
