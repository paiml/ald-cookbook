# Create Image Dataset

**Category**: A (Dataset Creation)
**Status**: Verified
**Isolation**: Full
**Idempotency**: Guaranteed

## Overview

Create an image dataset with pixel data stored as binary arrays alongside labels.

## Run the Recipe

```bash
cargo run --example create_ald_image_dataset
```

## Code Highlights

```rust
let schema = Arc::new(Schema::new(vec![
    Field::new("image_id", DataType::Int64, false),
    Field::new("pixels", DataType::LargeBinary, false),  // Raw pixel data
    Field::new("width", DataType::Int32, false),
    Field::new("height", DataType::Int32, false),
    Field::new("channels", DataType::Int32, false),
    Field::new("label", DataType::Int32, false),
]));
```

## Use Cases

- Computer vision training
- Image classification
- Object detection datasets

## QA Checklist

All 10 points verified. See [QA Checklist](../../appendix/qa-checklist.md).
