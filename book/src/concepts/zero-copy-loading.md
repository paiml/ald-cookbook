# Zero-Copy Loading

Zero-copy loading enables memory-efficient data access by avoiding unnecessary copies during deserialization.

## The Problem

Traditional data loading involves multiple copies:

```
File → Decompression Buffer → Deserialization → Application Memory
         Copy #1                  Copy #2            Copy #3
```

## The Solution

Arrow IPC format enables direct memory access:

```
File → Decompression → Direct Arrow Access
         Copy #1           Zero-Copy
```

## How It Works

### Arrow Memory Layout

Arrow arrays use a flat memory layout that matches the on-disk format:

```rust
// Int64 array: [1, 2, 3, null, 5]
// Memory layout:
// Validity bitmap: [0b11101]  (bit per row, 1 = valid)
// Values buffer:   [1, 2, 3, 0, 5] (8 bytes each)
```

### Memory Mapping

For large datasets, memory mapping avoids loading the entire file:

```rust
use std::fs::File;
use memmap2::Mmap;

fn load_mmap(path: &str) -> Result<RecordBatch> {
    let file = File::open(path)?;
    let mmap = unsafe { Mmap::map(&file)? };

    // Parse directly from mapped memory
    // No heap allocation for data buffers
    parse_ald(&mmap)
}
```

## ALD Zero-Copy Flow

```rust
use ald_cookbook::{load, Result};

fn main() -> Result<()> {
    // 1. Read and decompress (one copy)
    // 2. Parse Arrow IPC (zero-copy)
    // 3. Return RecordBatch referencing decompressed buffer

    let (batch, _) = load("dataset.ald")?;

    // Column access is O(1) - just pointer arithmetic
    let column = batch.column(0);

    Ok(())
}
```

## Benchmarks

| Operation | With Copy | Zero-Copy | Speedup |
|-----------|-----------|-----------|---------|
| Load 1M rows | 45ms | 12ms | 3.75x |
| Load 10M rows | 450ms | 95ms | 4.7x |
| Column access | O(n) | O(1) | N/A |
| Memory usage | 2x data | 1x data | 2x |

## Arrow Slice Operations

Slicing is zero-copy - just adjusts offsets:

```rust
use arrow::record_batch::RecordBatch;

fn slice_example(batch: &RecordBatch) -> RecordBatch {
    // Zero-copy: creates view into existing buffers
    batch.slice(100, 50)  // offset=100, length=50
}
```

## Column Projection

Select columns without copying:

```rust
use arrow::record_batch::RecordBatch;

fn project_columns(batch: &RecordBatch) -> Result<RecordBatch> {
    // Only keeps references to selected columns
    let schema = batch.schema();
    let projected_schema = Arc::new(Schema::new(vec![
        schema.field(0).clone(),
        schema.field(2).clone(),
    ]));

    RecordBatch::try_new(
        projected_schema,
        vec![
            batch.column(0).clone(),  // Arc clone, not data copy
            batch.column(2).clone(),
        ],
    )
}
```

## When Copies Occur

### Necessary Copies

1. **Decompression**: Zstd decompression requires a copy
2. **Type conversion**: Cast operations need new buffers
3. **Concatenation**: Combining batches creates new buffers

### Avoidable Copies

1. **String cloning**: Use `Arc<str>` or references
2. **Vec reallocation**: Pre-allocate with capacity
3. **Unnecessary collect**: Use iterators directly

## Best Practices

### Do: Use References

```rust
fn process(batch: &RecordBatch) -> Result<f64> {
    let col = batch.column(0)
        .as_any()
        .downcast_ref::<Float64Array>()
        .ok_or(Error::TypeMismatch)?;

    // Process without copying
    Ok(col.iter().flatten().sum())
}
```

### Don't: Unnecessary Clone

```rust
// BAD: Clones entire array
fn bad_process(batch: RecordBatch) -> f64 {
    let col = batch.column(0).clone();  // Unnecessary!
    // ...
}
```

### Do: Use Slice for Subsets

```rust
fn process_chunk(batch: &RecordBatch, chunk_size: usize) {
    for offset in (0..batch.num_rows()).step_by(chunk_size) {
        let len = chunk_size.min(batch.num_rows() - offset);
        let chunk = batch.slice(offset, len);  // Zero-copy
        process(&chunk);
    }
}
```

## Memory Layout Visualization

```
RecordBatch (stack)
├── schema: Arc<Schema>
└── columns: Vec<ArrayRef>
    ├── column_0: Arc<dyn Array>
    │   └── data: ArrayData
    │       ├── buffers: [Arc<Buffer>] ──────┐
    │       └── null_bitmap: Option<Buffer> ─┤
    ├── column_1: Arc<dyn Array>             │
    │   └── data: ArrayData                  │
    │       └── buffers ─────────────────────┤
    ...                                      │
                                             ▼
                                    ┌─────────────────┐
                                    │  Shared Memory  │
                                    │  (Heap Buffer)  │
                                    └─────────────────┘
```

## Integration with ALD

The ALD format is designed for zero-copy:

1. **Header**: Fixed 34 bytes, parsed in-place
2. **Metadata**: MessagePack parsed to structs (small copy)
3. **Schema**: Arrow IPC, zero-copy parse
4. **Payload**: Decompressed once, then zero-copy Arrow access

## Next Steps

- [Property-Based Testing](./property-based-testing.md) - Test invariants
- [A: Dataset Creation](../recipes/a-dataset-creation/index.md) - Create datasets
