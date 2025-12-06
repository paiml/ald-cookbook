# Error Handling

## Error Type

All errors in ALD Cookbook use the `Error` enum:

```rust
use ald_cookbook::Error;

pub enum Error {
    /// I/O errors from file operations
    Io(std::io::Error),

    /// Arrow-related errors
    Arrow(arrow::error::ArrowError),

    /// Parquet-related errors
    Parquet(parquet::errors::ParquetError),

    /// Invalid ALD format
    InvalidFormat(String),

    /// Checksum verification failed
    ChecksumMismatch { expected: u32, actual: u32 },

    /// Unsupported ALD version
    UnsupportedVersion { major: u8, minor: u8 },

    /// Invalid metadata
    InvalidMetadata(String),

    /// Type mismatch
    TypeMismatch(String),

    /// Conversion error
    Conversion(String),

    /// Registry error
    Registry(String),
}
```

## Result Type

Convenience alias:

```rust
pub type Result<T> = std::result::Result<T, Error>;
```

## Error Handling Patterns

### Using `?` Operator

```rust
use ald_cookbook::{load, Result};

fn process_dataset(path: &str) -> Result<usize> {
    let (batch, _) = load(path)?;
    Ok(batch.num_rows())
}
```

### Pattern Matching

```rust
use ald_cookbook::{load, Error};

fn handle_error(path: &str) {
    match load(path) {
        Ok((batch, _)) => println!("Loaded {} rows", batch.num_rows()),
        Err(Error::Io(e)) => eprintln!("I/O error: {}", e),
        Err(Error::InvalidFormat(msg)) => eprintln!("Invalid format: {}", msg),
        Err(Error::ChecksumMismatch { expected, actual }) => {
            eprintln!("Checksum mismatch: expected {}, got {}", expected, actual);
        }
        Err(e) => eprintln!("Other error: {}", e),
    }
}
```

### Converting From Other Errors

```rust
use ald_cookbook::Error;

// std::io::Error -> Error
let io_error = std::fs::read("nonexistent").unwrap_err();
let error: Error = io_error.into();

// String -> Error (InvalidFormat)
let error = Error::InvalidFormat("missing magic bytes".to_string());
```

## Error Display

All errors implement `Display`:

```rust
let error = Error::ChecksumMismatch {
    expected: 0x12345678,
    actual: 0xDEADBEEF,
};

println!("{}", error);
// Output: Checksum mismatch: expected 305419896, got 3735928559
```

## Best Practices

### 1. Use `?` for Propagation

```rust
// Good: propagate with ?
fn good(path: &str) -> Result<RecordBatch> {
    let (batch, _) = load(path)?;
    Ok(batch)
}
```

### 2. Avoid `unwrap()` in Production

```rust
// Bad: panics on error
fn bad(path: &str) -> RecordBatch {
    load(path).unwrap().0
}

// Good: handle error
fn good(path: &str) -> Result<RecordBatch> {
    let (batch, _) = load(path)?;
    Ok(batch)
}
```

### 3. Add Context with `map_err`

```rust
fn load_with_context(path: &str) -> Result<RecordBatch> {
    let (batch, _) = load(path)
        .map_err(|e| Error::InvalidFormat(
            format!("Failed to load {}: {}", path, e)
        ))?;
    Ok(batch)
}
```

### 4. Use `expect()` Only in Tests

```rust
#[test]
fn test_load() {
    let (batch, _) = load("test.ald").expect("test file should exist");
    assert!(batch.num_rows() > 0);
}
```

## Thiserror Integration

Errors are defined using `thiserror`:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    #[error("Checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch { expected: u32, actual: u32 },
}
```
