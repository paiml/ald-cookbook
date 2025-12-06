# The ALD Format

The **Alimentar Dataset Format** (`.ald`) is a binary format for secure, verifiable dataset distribution. It combines Apache Arrow IPC with MessagePack metadata and zstd compression.

## Format Structure

```
┌─────────────────────────────────────┐
│ Header (34 bytes, fixed)            │
│  Magic: "ALDF" (0x414C4446)         │
│  Version: major.minor (2 bytes)     │
│  Flags: 4 bytes (feature bits)      │
│  metadata_len: 4 bytes (u32 LE)     │
│  schema_len: 4 bytes (u32 LE)       │
│  payload_len: 8 bytes (u64 LE)      │
│  reserved: 8 bytes                  │
├─────────────────────────────────────┤
│ Metadata (variable, MessagePack)    │
│  name, description, created_at      │
│  row_count, schema_hash, etc.       │
├─────────────────────────────────────┤
│ Schema (variable, Arrow IPC)        │
│  Full Arrow schema with metadata    │
├─────────────────────────────────────┤
│ Payload (variable, Arrow IPC)       │
│  RecordBatch data (zstd compressed) │
├─────────────────────────────────────┤
│ Checksum (4 bytes, CRC32)           │
│  Integrity over all prior sections  │
└─────────────────────────────────────┘
```

## Header Structure

The 34-byte header is always at offset 0:

```rust
pub struct Header {
    pub magic: [u8; 4],        // "ALDF" = 0x414C4446
    pub version_major: u8,     // Currently 1
    pub version_minor: u8,     // Currently 2
    pub flags: FormatFlags,    // Feature flags (4 bytes)
    pub metadata_len: u32,     // MessagePack metadata length
    pub schema_len: u32,       // Arrow IPC schema length
    pub payload_len: u64,      // Compressed payload length
    pub reserved: [u8; 8],     // Future use (zeroed)
}
```

## Format Flags

Feature flags indicate optional capabilities:

```rust
bitflags! {
    pub struct FormatFlags: u32 {
        const ENCRYPTED  = 0b0001;  // AES-256-GCM encryption
        const SIGNED     = 0b0010;  // Ed25519 signature
        const STREAMING  = 0b0100;  // Chunked loading support
        const COMPRESSED = 0b1000;  // Zstd compression (default)
    }
}
```

## Metadata Section

MessagePack-encoded metadata provides dataset information:

```rust
pub struct Metadata {
    pub name: String,              // Dataset identifier
    pub description: String,       // Human-readable description
    pub created_at: DateTime<Utc>, // Creation timestamp
    pub row_count: u64,            // Number of rows
    pub schema_hash: String,       // SHA-256 of schema
    pub dataset_type: DatasetType, // Tabular, TimeSeries, Text, Image
    pub custom: HashMap<String, Value>, // User metadata
}
```

## Schema Section

The schema is stored as Arrow IPC format:

```rust
// Arrow schema with field types and metadata
let schema = Schema::new(vec![
    Field::new("id", DataType::Int64, false),
    Field::new("value", DataType::Float64, true),
    Field::new("label", DataType::Utf8, false),
]);
```

## Payload Section

The payload contains compressed Arrow RecordBatch data:

1. **Serialization**: Arrow IPC format
2. **Compression**: zstd (level 3 default)
3. **Alignment**: 8-byte aligned for zero-copy

## Checksum

CRC32 checksum over all preceding bytes for integrity verification:

```rust
let checksum = crc32fast::hash(&file_bytes[..file_bytes.len() - 4]);
```

## Version History

| Version | Changes |
|---------|---------|
| 1.0 | Initial format |
| 1.1 | Added streaming flag |
| 1.2 | Added signature support |

## Loading Process

```rust
use ald_cookbook::{load, Result};

fn main() -> Result<()> {
    // 1. Read header (34 bytes)
    // 2. Verify magic bytes
    // 3. Parse metadata (MessagePack)
    // 4. Parse schema (Arrow IPC)
    // 5. Decompress payload (zstd)
    // 6. Verify checksum
    // 7. Return RecordBatch

    let (batch, metadata) = load("dataset.ald")?;
    println!("Loaded {} rows", batch.num_rows());
    Ok(())
}
```

## Saving Process

```rust
use ald_cookbook::{save, SaveOptions, DatasetType, Result};

fn main() -> Result<()> {
    let options = SaveOptions::new("my_dataset")
        .with_description("Example dataset")
        .with_dataset_type(DatasetType::Tabular)
        .with_compression_level(3);

    save(&batch, "output.ald", options)?;
    Ok(())
}
```

## Design Rationale

### Why Arrow IPC?

- **Zero-copy**: Memory-mapped access without deserialization
- **Type-safe**: Rich type system with nullable fields
- **Interoperable**: Works with pandas, polars, DuckDB
- **Efficient**: Columnar format for analytical queries

### Why MessagePack?

- **Compact**: Smaller than JSON
- **Schema-less**: Flexible metadata
- **Fast**: Single-pass parsing
- **Compatible**: Wide language support

### Why Zstd?

- **Speed**: Fast decompression (>1GB/s)
- **Ratio**: Excellent compression ratios
- **Streaming**: Supports partial decompression
- **Standard**: Industry-wide adoption

### Why CRC32?

- **Fast**: Hardware-accelerated
- **Sufficient**: Detects corruption reliably
- **Standard**: Well-understood checksum

## Next Steps

- [IIUR Principles](./iiur-principles.md) - Recipe design principles
- [Zero-Copy Loading](./zero-copy-loading.md) - Memory-efficient loading
