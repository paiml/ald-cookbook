# Create Text Corpus

**Category**: A (Dataset Creation)
**Status**: Verified
**Isolation**: Full
**Idempotency**: Guaranteed

## Overview

Create a text corpus dataset for NLP tasks with document text and metadata.

## Run the Recipe

```bash
cargo run --example create_ald_text_corpus
```

## Code Highlights

```rust
let schema = Arc::new(Schema::new(vec![
    Field::new("doc_id", DataType::Int64, false),
    Field::new("text", DataType::LargeUtf8, false),  // Large text support
    Field::new("label", DataType::Utf8, true),
    Field::new("source", DataType::Utf8, false),
]));
```

## Use Cases

- NLP training datasets
- Document classification
- Sentiment analysis

## QA Checklist

All 10 points verified. See [QA Checklist](../../appendix/qa-checklist.md).
