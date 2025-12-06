# A: Dataset Creation

This category covers creating ALD datasets from various data sources.

## Recipes

| Recipe | Status | Coverage | Idempotent |
|--------|--------|----------|------------|
| [Create ALD from Arrow](./create-ald-from-arrow.md) | Verified | 95%+ | Yes |
| [Create Tabular Dataset](./create-ald-tabular.md) | Verified | 95%+ | Yes |
| [Create TimeSeries Dataset](./create-ald-timeseries.md) | Verified | 95%+ | Yes |
| [Create Text Corpus](./create-ald-text-corpus.md) | Verified | 95%+ | Yes |
| [Create Image Dataset](./create-ald-image-dataset.md) | Verified | 95%+ | Yes |

## Learning Objectives

After completing this category, you will be able to:

1. Create ALD datasets from Arrow RecordBatches
2. Build tabular datasets with mixed column types
3. Construct time series datasets with temporal indices
4. Package text corpora for NLP tasks
5. Create image datasets for computer vision

## Prerequisites

- Basic understanding of Arrow data types
- Familiarity with the ALD format structure
- Rust basics (structs, Result, Error handling)

## Common Patterns

### Creating a RecordBatch

```rust
use arrow::array::{Int64Array, Float64Array, StringArray};
use arrow::datatypes::{Schema, Field, DataType};
use arrow::record_batch::RecordBatch;

let schema = Arc::new(Schema::new(vec![
    Field::new("id", DataType::Int64, false),
    Field::new("value", DataType::Float64, true),
    Field::new("label", DataType::Utf8, false),
]));

let batch = RecordBatch::try_new(
    schema,
    vec![
        Arc::new(Int64Array::from(vec![1, 2, 3])),
        Arc::new(Float64Array::from(vec![1.0, 2.0, 3.0])),
        Arc::new(StringArray::from(vec!["a", "b", "c"])),
    ],
)?;
```

### Saving to ALD

```rust
use ald_cookbook::{save, SaveOptions, DatasetType};

let options = SaveOptions::new("my_dataset")
    .with_description("Example dataset")
    .with_dataset_type(DatasetType::Tabular);

save(&batch, "output.ald", options)?;
```

## Run All Category A Recipes

```bash
cargo run --example create_ald_from_arrow
cargo run --example create_ald_tabular
cargo run --example create_ald_timeseries
cargo run --example create_ald_text_corpus
cargo run --example create_ald_image_dataset
```
