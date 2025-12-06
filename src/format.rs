//! ALD (Alimentar Dataset) format implementation.
//! 
//! The ALD format provides secure, verifiable dataset distribution:
//! 
//! ```text
//! ┌─────────────────────────────────────────┐
//! │ Header (32 bytes, fixed)                │
//! │   Magic: "ALDF" (0x414C4446)            │
//! │   Version: 1.2                          │
//! │   Flags: encryption, signing, streaming │
//! ├─────────────────────────────────────────┤
//! │ Metadata (variable, MessagePack)        │
//! ├─────────────────────────────────────────┤
//! │ Schema (variable, Arrow IPC)            │
//! ├─────────────────────────────────────────┤
//! │ Payload (variable, Arrow IPC + zstd)    │
//! ├─────────────────────────────────────────┤
//! │ Checksum (4 bytes, CRC32)               │
//! └─────────────────────────────────────────┘
//! ```

#![allow(
    clippy::struct_excessive_bools,
    clippy::cast_possible_truncation,
    clippy::needless_pass_by_value
)]

use crate::error::{Error, Result};use arrow::array::RecordBatch;
use arrow::datatypes::SchemaRef;
use arrow::ipc::reader::StreamReader;
use arrow::ipc::writer::StreamWriter;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use std::io::{Cursor, Read, Write};
use std::path::Path;

/// ALD magic bytes: "ALDF" in ASCII.
pub const ALD_MAGIC: u32 = 0x464C_4441; // "ALDF" little-endian
/// Current major version.
pub const VERSION_MAJOR: u8 = 1;
/// Current minor version.
pub const VERSION_MINOR: u8 = 2;
/// Header size in bytes.
/// Layout: magic(4) + `version_major(1)` + `version_minor(1)` + flags(4) + `metadata_len(4)` + `schema_len(4)` + `payload_len(8)` + reserved(8) = 34
pub const HEADER_SIZE: usize = 34;

/// Flags for ALD format features.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FormatFlags {
    /// Data is encrypted.
    pub encrypted: bool,
    /// Data is signed.
    pub signed: bool,
    /// Supports streaming reads.
    pub streaming: bool,
    /// Payload is compressed.
    pub compressed: bool,
}

impl FormatFlags {
    /// Convert flags to a u32 bitmap.
    #[must_use]
    pub const fn to_bits(self) -> u32 {
        let mut bits = 0u32;
        if self.encrypted {
            bits |= 1 << 0;
        }
        if self.signed {
            bits |= 1 << 1;
        }
        if self.streaming {
            bits |= 1 << 2;
        }
        if self.compressed {
            bits |= 1 << 3;
        }
        bits
    }

    /// Create flags from a u32 bitmap.
    #[must_use]
    pub const fn from_bits(bits: u32) -> Self {
        Self {
            encrypted: bits & (1 << 0) != 0,
            signed: bits & (1 << 1) != 0,
            streaming: bits & (1 << 2) != 0,
            compressed: bits & (1 << 3) != 0,
        }
    }
}

/// ALD file header (32 bytes).
#[derive(Debug, Clone)]
pub struct Header {
    /// Magic bytes (must be `ALD_MAGIC`).
    pub magic: u32,
    /// Major version.
    pub version_major: u8,
    /// Minor version.
    pub version_minor: u8,
    /// Format flags.
    pub flags: FormatFlags,
    /// Metadata section length.
    pub metadata_len: u32,
    /// Schema section length.
    pub schema_len: u32,
    /// Payload section length.
    pub payload_len: u64,
    /// Reserved for future use.
    pub reserved: [u8; 8],
}

impl Header {
    /// Create a new header with current version.
    #[must_use]
    pub const fn new(
        metadata_len: u32,
        schema_len: u32,
        payload_len: u64,
        flags: FormatFlags,
    ) -> Self {
        Self {
            magic: ALD_MAGIC,
            version_major: VERSION_MAJOR,
            version_minor: VERSION_MINOR,
            flags,
            metadata_len,
            schema_len,
            payload_len,
            reserved: [0u8; 8],
        }
    }

    /// Write header to a writer.
    ///
    /// # Errors
    ///
    /// Returns `Error::Io` if writing fails.
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u32::<LittleEndian>(self.magic)?;
        writer.write_u8(self.version_major)?;
        writer.write_u8(self.version_minor)?;
        writer.write_u32::<LittleEndian>(self.flags.to_bits())?;
        writer.write_u32::<LittleEndian>(self.metadata_len)?;
        writer.write_u32::<LittleEndian>(self.schema_len)?;
        writer.write_u64::<LittleEndian>(self.payload_len)?;
        writer.write_all(&self.reserved)?;
        Ok(())
    }

    /// Read header from a reader.
    ///
    /// # Errors
    ///
    /// Returns `Error::InvalidMagic` if magic bytes don't match.
    /// Returns `Error::UnsupportedVersion` if version is not supported.
    /// Returns `Error::Io` if reading fails.
    pub fn read<R: Read>(reader: &mut R) -> Result<Self> {
        let magic = reader.read_u32::<LittleEndian>()?;
        if magic != ALD_MAGIC {
            return Err(Error::InvalidMagic(magic));
        }

        let version_major = reader.read_u8()?;
        let version_minor = reader.read_u8()?;

        if version_major != VERSION_MAJOR {
            return Err(Error::UnsupportedVersion {
                major: version_major,
                minor: version_minor,
            });
        }

        let flags = FormatFlags::from_bits(reader.read_u32::<LittleEndian>()?);
        let metadata_len = reader.read_u32::<LittleEndian>()?;
        let schema_len = reader.read_u32::<LittleEndian>()?;
        let payload_len = reader.read_u64::<LittleEndian>()?;

        let mut reserved = [0u8; 8];
        reader.read_exact(&mut reserved)?;

        Ok(Self {
            magic,
            version_major,
            version_minor,
            flags,
            metadata_len,
            schema_len,
            payload_len,
            reserved,
        })
    }
}

/// Dataset type enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum DatasetType {
    /// Tabular data (rows and columns).
    #[default]
    Tabular,
    /// Time series data.
    TimeSeries,
    /// Text corpus for NLP.
    TextCorpus,
    /// Image classification dataset.
    ImageClassification,
    /// Generic binary data.
    Binary,
}

/// Dataset metadata stored in `MessagePack` format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    /// Dataset name.
    pub name: Option<String>,
    /// Dataset description.
    pub description: Option<String>,
    /// Dataset type.
    pub dataset_type: DatasetType,
    /// Number of rows.
    pub num_rows: usize,
    /// Number of columns.
    pub num_columns: usize,
    /// Creation timestamp (ISO 8601).
    pub created_at: String,
    /// License identifier.
    pub license: Option<String>,
    /// Custom key-value attributes.
    #[serde(default)]
    pub attributes: std::collections::HashMap<String, String>,
}

impl Metadata {
    /// Create new metadata.
    #[must_use]
    pub fn new(dataset_type: DatasetType, num_rows: usize, num_columns: usize) -> Self {
        Self {
            name: None,
            description: None,
            dataset_type,
            num_rows,
            num_columns,
            created_at: chrono::Utc::now().to_rfc3339(),
            license: None,
            attributes: std::collections::HashMap::new(),
        }
    }

    /// Set the name.
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the description.
    #[must_use]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the license.
    #[must_use]
    pub fn with_license(mut self, license: impl Into<String>) -> Self {
        self.license = Some(license.into());
        self
    }

    /// Serialize to `MessagePack` bytes.
    ///
    /// # Errors
    ///
    /// Returns `Error::Serialization` if serialization fails.
    pub fn to_msgpack(&self) -> Result<Vec<u8>> {
        rmp_serde::to_vec(self).map_err(|e| Error::Serialization(e.to_string()))
    }

    /// Deserialize from `MessagePack` bytes.
    ///
    /// # Errors
    ///
    /// Returns `Error::Deserialization` if deserialization fails.
    pub fn from_msgpack(bytes: &[u8]) -> Result<Self> {
        rmp_serde::from_slice(bytes).map_err(|e| Error::Deserialization(e.to_string()))
    }
}

/// Options for saving datasets.
#[derive(Debug, Clone, Default)]
pub struct SaveOptions {
    /// Enable compression (default: true).
    pub compress: bool,
    /// Compression level (1-22 for zstd, default: 3).
    pub compression_level: i32,
    /// Dataset name.
    pub name: Option<String>,
    /// Dataset description.
    pub description: Option<String>,
    /// License identifier.
    pub license: Option<String>,
}

impl SaveOptions {
    /// Create default save options with compression enabled.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            compress: true,
            compression_level: 3,
            name: None,
            description: None,
            license: None,
        }
    }

    /// Set compression level.
    #[must_use]
    pub const fn with_compression_level(mut self, level: i32) -> Self {
        self.compression_level = level;
        self
    }

    /// Disable compression.
    #[must_use]
    pub const fn without_compression(mut self) -> Self {
        self.compress = false;
        self
    }

    /// Set dataset name.
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}

/// Save a `RecordBatch` to ALD format.
///
/// # Errors
///
/// Returns `Error::Io` if file operations fail.
/// Returns `Error::Arrow` if Arrow IPC encoding fails.
/// Returns `Error::Serialization` if metadata serialization fails.
pub fn save(
    batch: &RecordBatch,
    dataset_type: DatasetType,
    path: impl AsRef<Path>,
    options: SaveOptions,
) -> Result<()> {
    let file = std::fs::File::create(path.as_ref())?;
    let mut writer = std::io::BufWriter::new(file);
    save_to_writer(batch, dataset_type, &mut writer, options)
}

/// Save a `RecordBatch` to ALD format using a writer.
///
/// # Errors
///
/// Returns `Error::Io` if write operations fail.
/// Returns `Error::Arrow` if Arrow IPC encoding fails.
pub fn save_to_writer<W: Write>(
    batch: &RecordBatch,
    dataset_type: DatasetType,
    writer: &mut W,
    options: SaveOptions,
) -> Result<()> {
    // Create metadata
    let mut metadata = Metadata::new(dataset_type, batch.num_rows(), batch.num_columns());
    if let Some(name) = options.name {
        metadata = metadata.with_name(name);
    }
    if let Some(desc) = options.description {
        metadata = metadata.with_description(desc);
    }
    if let Some(license) = options.license {
        metadata = metadata.with_license(license);
    }

    let metadata_bytes = metadata.to_msgpack()?;

    // Encode schema to Arrow IPC
    let schema_bytes = encode_schema(batch.schema())?;

    // Encode payload to Arrow IPC (optionally compressed)
    let payload_bytes = encode_payload(batch, options.compress, options.compression_level)?;

    // Create header
    let flags = FormatFlags {
        compressed: options.compress,
        ..Default::default()
    };

    let header = Header::new(
        u32::try_from(metadata_bytes.len()).unwrap_or(u32::MAX),
        u32::try_from(schema_bytes.len()).unwrap_or(u32::MAX),
        payload_bytes.len() as u64,
        flags,
    );

    // Write header placeholder (we'll update with checksum later)
    header.write(writer)?;

    // Write sections
    writer.write_all(&metadata_bytes)?;
    writer.write_all(&schema_bytes)?;
    writer.write_all(&payload_bytes)?;

    // Compute checksum over all data (header + metadata + schema + payload)
    let mut hasher = crc32fast::Hasher::new();

    // Compute checksum from components
    let mut header_bytes = Vec::new();
    header.write(&mut header_bytes)?;
    hasher.update(&header_bytes);
    hasher.update(&metadata_bytes);
    hasher.update(&schema_bytes);
    hasher.update(&payload_bytes);

    let checksum = hasher.finalize();
    writer.write_u32::<LittleEndian>(checksum)?;

    Ok(())
}

/// Load a `RecordBatch` from ALD format.
///
/// # Errors
///
/// Returns `Error::DatasetNotFound` if file doesn't exist.
/// Returns `Error::InvalidMagic` if magic bytes don't match.
/// Returns `Error::ChecksumMismatch` if checksum verification fails.
pub fn load(path: impl AsRef<Path>) -> Result<RecordBatch> {
    let path = path.as_ref();
    if !path.exists() {
        return Err(Error::DatasetNotFound(path.to_path_buf()));
    }

    let data = std::fs::read(path)?;
    load_from_bytes(&data)
}

/// Load a `RecordBatch` from ALD bytes.
///
/// # Errors
///
/// Returns `Error::InvalidMagic` if magic bytes don't match.
/// Returns `Error::ChecksumMismatch` if checksum verification fails.
pub fn load_from_bytes(data: &[u8]) -> Result<RecordBatch> {
    if data.len() < HEADER_SIZE + 4 {
        return Err(Error::InvalidFormat {
            expected: format!("at least {} bytes", HEADER_SIZE + 4),
            actual: format!("{} bytes", data.len()),
        });
    }

    let mut cursor = Cursor::new(data);

    // Read and validate header
    let header = Header::read(&mut cursor)?;

    // Verify checksum
    let checksum_pos = data.len() - 4;
    let stored_checksum = u32::from_le_bytes([
        data[checksum_pos],
        data[checksum_pos + 1],
        data[checksum_pos + 2],
        data[checksum_pos + 3],
    ]);

    let mut hasher = crc32fast::Hasher::new();
    hasher.update(&data[..checksum_pos]);
    let computed_checksum = hasher.finalize();

    if stored_checksum != computed_checksum {
        return Err(Error::ChecksumMismatch {
            expected: stored_checksum,
            actual: computed_checksum,
        });
    }

    // Read metadata
    let metadata_start = HEADER_SIZE;
    let metadata_end = metadata_start + header.metadata_len as usize;
    let _metadata = Metadata::from_msgpack(&data[metadata_start..metadata_end])?;

    // Read schema
    let schema_start = metadata_end;
    let schema_end = schema_start + header.schema_len as usize;
    let _schema = decode_schema(&data[schema_start..schema_end])?;

    // Read payload
    let payload_start = schema_end;
    let payload_end = payload_start + header.payload_len as usize;
    let payload_data = &data[payload_start..payload_end];

    // Decode payload (decompress if needed)
    decode_payload(payload_data, header.flags.compressed)
}

/// Load metadata without loading the full dataset.
///
/// # Errors
///
/// Returns errors if file reading or parsing fails.
pub fn load_metadata(path: impl AsRef<Path>) -> Result<Metadata> {
    let path = path.as_ref();
    if !path.exists() {
        return Err(Error::DatasetNotFound(path.to_path_buf()));
    }

    let mut file = std::fs::File::open(path)?;
    let header = Header::read(&mut file)?;

    let mut metadata_bytes = vec![0u8; header.metadata_len as usize];
    file.read_exact(&mut metadata_bytes)?;

    Metadata::from_msgpack(&metadata_bytes)
}

// Helper: Encode schema to Arrow IPC format
fn encode_schema(schema: SchemaRef) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    {
        let mut writer = StreamWriter::try_new(&mut buf, &schema)?;
        writer.finish()?;
    }
    Ok(buf)
}

// Helper: Decode schema from Arrow IPC format
fn decode_schema(data: &[u8]) -> Result<SchemaRef> {
    let cursor = Cursor::new(data);
    let reader = StreamReader::try_new(cursor, None)?;
    Ok(reader.schema())
}

// Helper: Encode payload to Arrow IPC (optionally compressed)
fn encode_payload(batch: &RecordBatch, compress: bool, level: i32) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    {
        let mut writer = StreamWriter::try_new(&mut buf, &batch.schema())?;
        writer.write(batch)?;
        writer.finish()?;
    }

    if compress {
        let compressed = zstd::encode_all(buf.as_slice(), level)
            .map_err(|e| Error::Compression(e.to_string()))?;
        Ok(compressed)
    } else {
        Ok(buf)
    }
}

// Helper: Decode payload from Arrow IPC (decompress if needed)
fn decode_payload(data: &[u8], compressed: bool) -> Result<RecordBatch> {
    let decompressed = if compressed {
        zstd::decode_all(data).map_err(|e| Error::Decompression(e.to_string()))?
    } else {
        data.to_vec()
    };

    let cursor = Cursor::new(decompressed);
    let mut reader = StreamReader::try_new(cursor, None)?;

    reader
        .next()
        .ok_or(Error::EmptyDataset)?
        .map_err(Error::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::{Float64Array, Int64Array, StringArray};
    use arrow::datatypes::{DataType, Field, Schema};
    use std::sync::Arc;

    fn create_test_batch() -> RecordBatch {
        let schema = Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("value", DataType::Float64, false),
            Field::new("label", DataType::Utf8, true),
        ]);

        let id_array = Int64Array::from(vec![1, 2, 3, 4, 5]);
        let value_array = Float64Array::from(vec![1.1, 2.2, 3.3, 4.4, 5.5]);
        let label_array = StringArray::from(vec![Some("a"), Some("b"), None, Some("d"), Some("e")]);

        RecordBatch::try_new(
            Arc::new(schema),
            vec![
                Arc::new(id_array),
                Arc::new(value_array),
                Arc::new(label_array),
            ],
        )
        .unwrap()
    }

    #[test]
    fn test_format_flags_roundtrip() {
        let flags = FormatFlags {
            encrypted: true,
            signed: false,
            streaming: true,
            compressed: true,
        };

        let bits = flags.to_bits();
        let recovered = FormatFlags::from_bits(bits);

        assert_eq!(flags, recovered);
    }

    #[test]
    fn test_header_roundtrip() {
        let header = Header::new(
            100,
            200,
            1000,
            FormatFlags {
                compressed: true,
                ..Default::default()
            },
        );

        let mut buf = Vec::new();
        header.write(&mut buf).unwrap();

        assert_eq!(buf.len(), HEADER_SIZE);

        let mut cursor = Cursor::new(buf);
        let recovered = Header::read(&mut cursor).unwrap();

        assert_eq!(recovered.magic, ALD_MAGIC);
        assert_eq!(recovered.version_major, VERSION_MAJOR);
        assert_eq!(recovered.version_minor, VERSION_MINOR);
        assert_eq!(recovered.metadata_len, 100);
        assert_eq!(recovered.schema_len, 200);
        assert_eq!(recovered.payload_len, 1000);
        assert!(recovered.flags.compressed);
    }

    #[test]
    fn test_header_invalid_magic() {
        let mut buf = vec![0u8; HEADER_SIZE];
        buf[0..4].copy_from_slice(&0x5041_5251u32.to_le_bytes()); // "PARQ"

        let mut cursor = Cursor::new(buf);
        let result = Header::read(&mut cursor);

        assert!(matches!(result, Err(Error::InvalidMagic(_))));
    }

    #[test]
    fn test_metadata_roundtrip() {
        let metadata = Metadata::new(DatasetType::Tabular, 1000, 5)
            .with_name("test_dataset")
            .with_description("A test dataset")
            .with_license("MIT");

        let bytes = metadata.to_msgpack().unwrap();
        let recovered = Metadata::from_msgpack(&bytes).unwrap();

        assert_eq!(recovered.name, Some("test_dataset".to_string()));
        assert_eq!(recovered.description, Some("A test dataset".to_string()));
        assert_eq!(recovered.dataset_type, DatasetType::Tabular);
        assert_eq!(recovered.num_rows, 1000);
        assert_eq!(recovered.num_columns, 5);
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let batch = create_test_batch();
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("test.ald");

        save(&batch, DatasetType::Tabular, &path, SaveOptions::new()).unwrap();
        assert!(path.exists());

        let loaded = load(&path).unwrap();

        assert_eq!(batch.num_rows(), loaded.num_rows());
        assert_eq!(batch.num_columns(), loaded.num_columns());
        assert_eq!(batch.schema(), loaded.schema());
    }

    #[test]
    fn test_save_and_load_uncompressed() {
        let batch = create_test_batch();
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("test_uncompressed.ald");

        save(
            &batch,
            DatasetType::Tabular,
            &path,
            SaveOptions::new().without_compression(),
        )
        .unwrap();

        let loaded = load(&path).unwrap();
        assert_eq!(batch.num_rows(), loaded.num_rows());
    }

    #[test]
    fn test_load_nonexistent_file() {
        let result = load("/nonexistent/path/file.ald");
        assert!(matches!(result, Err(Error::DatasetNotFound(_))));
    }

    #[test]
    fn test_load_metadata_only() {
        let batch = create_test_batch();
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("metadata_test.ald");

        save(
            &batch,
            DatasetType::TimeSeries,
            &path,
            SaveOptions::new().with_name("metadata_test"),
        )
        .unwrap();

        let metadata = load_metadata(&path).unwrap();
        assert_eq!(metadata.name, Some("metadata_test".to_string()));
        assert_eq!(metadata.dataset_type, DatasetType::TimeSeries);
        assert_eq!(metadata.num_rows, 5);
    }

    #[test]
    fn test_checksum_verification() {
        let batch = create_test_batch();
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("checksum_test.ald");

        save(&batch, DatasetType::Tabular, &path, SaveOptions::new()).unwrap();

        // Corrupt the file
        let mut data = std::fs::read(&path).unwrap();
        if data.len() > HEADER_SIZE + 10 {
            data[HEADER_SIZE + 5] ^= 0xFF; // Flip some bits
        }
        std::fs::write(&path, &data).unwrap();

        let result = load(&path);
        assert!(matches!(result, Err(Error::ChecksumMismatch { .. })));
    }

    #[test]
    fn test_dataset_types() {
        assert_eq!(DatasetType::default(), DatasetType::Tabular);

        let types = [
            DatasetType::Tabular,
            DatasetType::TimeSeries,
            DatasetType::TextCorpus,
            DatasetType::ImageClassification,
            DatasetType::Binary,
        ];

        for dt in types {
            let metadata = Metadata::new(dt, 100, 10);
            let bytes = metadata.to_msgpack().unwrap();
            let recovered = Metadata::from_msgpack(&bytes).unwrap();
            assert_eq!(recovered.dataset_type, dt);
        }
    }
}
