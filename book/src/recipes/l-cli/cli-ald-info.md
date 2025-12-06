# ALD Info

**Category**: L (CLI Tools)
**Status**: Verified
**Isolation**: Full
**Idempotency**: Guaranteed

## Overview

Inspect ALD dataset metadata and structure from the command line. Displays format version, compression, schema, and row counts.

## Run the Recipe

```bash
cargo run --example cli_ald_info
```

## Code

```rust
use ald_cookbook::prelude::*;
use ald_cookbook::format::{load_metadata, Header};
use ald_cookbook::{RecipeContext, Result};
use std::fs::File;

fn main() -> Result<()> {
    let mut ctx = RecipeContext::new("cli_ald_info")?;

    // Create a sample ALD file
    let batch = create_sample_batch(&mut ctx)?;
    let path = ctx.temp_path("sample.ald");
    save(&batch, DatasetType::Tabular, &path, SaveOptions::new())?;

    // Read and display info
    let file = File::open(&path)?;
    let header = Header::read(&mut std::io::BufReader::new(&file))?;
    let metadata = load_metadata(&path)?;

    println!("ALD File: {:?}", path);
    println!("Version: {}.{}", header.version_major, header.version_minor);
    println!("Flags: {:?}", header.flags);
    println!("Rows: {}", metadata.num_rows);
    println!("Columns: {}", metadata.num_columns);
    println!("Type: {:?}", metadata.dataset_type);

    ctx.report("Displayed ALD file info")?;

    Ok(())
}
```

## Output Format

```
ALD File: "/tmp/.tmpXXX/sample.ald"
Version: 1.2
Flags: FormatFlags { compressed: true, ... }
Rows: 1000
Columns: 5
Type: Tabular
```

## PMAT Testing

### Property Tests

```rust
proptest! {
    // Info matches saved data
    #[test]
    fn info_matches_saved(batch in batch_strategy(), dtype in dataset_type()) {
        let path = temp_path();
        save(&batch, dtype, &path, SaveOptions::new())?;

        let metadata = load_metadata(&path)?;

        prop_assert_eq!(metadata.num_rows, batch.num_rows());
        prop_assert_eq!(metadata.num_columns, batch.num_columns());
        prop_assert_eq!(metadata.dataset_type, dtype);
    }

    // Header version is current
    #[test]
    fn header_version_current(batch in batch_strategy()) {
        let path = temp_path();
        save(&batch, DatasetType::Tabular, &path, SaveOptions::new())?;

        let header = read_header(&path)?;

        prop_assert_eq!(header.version_major, 1);
        prop_assert_eq!(header.version_minor, 2);
    }

    // Compression flag reflects options
    #[test]
    fn compression_flag_correct(batch in batch_strategy(), compress in any::<bool>()) {
        let path = temp_path();
        let options = if compress {
            SaveOptions::new()
        } else {
            SaveOptions::new().without_compression()
        };

        save(&batch, DatasetType::Tabular, &path, options)?;

        let header = read_header(&path)?;
        prop_assert_eq!(header.flags.compressed, compress);
    }
}
```

### Mutation Testing Targets

| Mutation | Expected Behavior |
|----------|-------------------|
| Wrong row count read | Info matches test fails |
| Version parsing error | Current version test fails |
| Flag interpretation wrong | Compression flag test fails |

### Adversarial Tests

```rust
#[test]
fn test_info_nonexistent_file() {
    let result = load_metadata("nonexistent.ald");
    assert!(result.is_err());
}

#[test]
fn test_info_invalid_file() {
    let path = temp_path();
    std::fs::write(&path, "not an ALD file")?;

    let result = load_metadata(&path);
    assert!(result.is_err());
}

#[test]
fn test_info_truncated_file() {
    let path = temp_path();
    save(&batch, DatasetType::Tabular, &path, SaveOptions::new())?;

    // Truncate the file
    let content = std::fs::read(&path)?;
    std::fs::write(&path, &content[..content.len() / 2])?;

    let result = load_metadata(&path);
    assert!(result.is_err());
}

#[test]
fn test_info_empty_file() {
    let path = temp_path();
    std::fs::write(&path, "")?;

    let result = load_metadata(&path);
    assert!(result.is_err());
}
```

## QA Checklist

| # | Check | Status |
|---|-------|--------|
| 1 | `cargo run` succeeds | Pass |
| 2 | `cargo test` passes | Pass |
| 3 | Deterministic output | Pass |
| 4 | No temp files leaked | Pass |
| 5 | Memory usage stable | Pass |
| 6 | Platform independent | Pass |
| 7 | Clippy clean | Pass |
| 8 | Rustfmt standard | Pass |
| 9 | No `unwrap()` in logic | Pass |
| 10 | Property tests pass | Pass |
