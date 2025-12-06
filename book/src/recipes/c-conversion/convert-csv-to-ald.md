# Convert CSV to ALD

**Category**: C (Conversion)
**Status**: Verified

## Run the Recipe

```bash
cargo run --example convert_csv_to_ald
```

## Code Highlights

```rust
use ald_cookbook::convert::csv_to_ald;

csv_to_ald("input.csv", "output.ald", SaveOptions::default())?;
```

## QA Checklist

All 10 points verified.
