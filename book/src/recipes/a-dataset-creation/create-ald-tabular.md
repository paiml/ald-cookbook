# Create Tabular Dataset

**Category**: A (Dataset Creation)
**Status**: Verified
**Isolation**: Full
**Idempotency**: Guaranteed

## Overview

Create a structured tabular dataset with multiple column types including integers, floats, strings, and nullable fields.

## Run the Recipe

```bash
cargo run --example create_ald_tabular
```

## Code Highlights

```rust
let schema = Arc::new(Schema::new(vec![
    Field::new("id", DataType::Int64, false),
    Field::new("name", DataType::Utf8, false),
    Field::new("score", DataType::Float64, true),  // Nullable
    Field::new("category", DataType::Utf8, false),
]));
```

## Use Cases

- Machine learning training datasets
- Analytics data warehousing
- Cross-platform data exchange

## QA Checklist

All 10 points verified. See [QA Checklist](../../appendix/qa-checklist.md).
