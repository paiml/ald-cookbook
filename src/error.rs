//! Error types for ALD Cookbook recipes.
//!
//! Follows the Toyota Way principle of **Jidoka** (built-in quality):
//! type-safe errors that make invalid states unrepresentable.

use std::path::PathBuf;

/// All errors that can occur in ALD Cookbook recipes.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Dataset file not found at the specified path.
    #[error("Dataset not found: {0}")]
    DatasetNotFound(PathBuf),

    /// Invalid ALD format - magic bytes, version, or structure mismatch.
    #[error("Invalid format: expected {expected}, got {actual}")]
    InvalidFormat {
        /// Expected format identifier.
        expected: String,
        /// Actual format identifier found.
        actual: String,
    },

    /// Invalid ALD magic bytes.
    #[error("Invalid magic bytes: expected ALDF (0x414C4446), got {0:#010X}")]
    InvalidMagic(u32),

    /// Unsupported ALD format version.
    #[error("Unsupported version: {major}.{minor} (supported: 1.x)")]
    UnsupportedVersion {
        /// Major version number.
        major: u8,
        /// Minor version number.
        minor: u8,
    },

    /// Schema mismatch between expected and actual.
    #[error("Schema mismatch: {0}")]
    SchemaMismatch(String),

    /// Checksum verification failed.
    #[error("Checksum mismatch: expected {expected:#010X}, got {actual:#010X}")]
    ChecksumMismatch {
        /// Expected CRC32 checksum.
        expected: u32,
        /// Actual computed checksum.
        actual: u32,
    },

    /// Column not found in dataset.
    #[error("Column not found: {0}")]
    ColumnNotFound(String),

    /// Invalid column type for operation.
    #[error("Invalid column type: expected {expected}, got {actual}")]
    InvalidColumnType {
        /// Expected data type.
        expected: String,
        /// Actual data type.
        actual: String,
    },

    /// Empty dataset error.
    #[error("Dataset is empty")]
    EmptyDataset,

    /// Invalid row index.
    #[error("Row index {index} out of bounds (dataset has {total} rows)")]
    RowIndexOutOfBounds {
        /// Requested index.
        index: usize,
        /// Total number of rows.
        total: usize,
    },

    /// IO error wrapper.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Arrow error wrapper.
    #[error("Arrow error: {0}")]
    Arrow(#[from] arrow::error::ArrowError),

    /// Parquet error wrapper.
    #[error("Parquet error: {0}")]
    Parquet(#[from] parquet::errors::ParquetError),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Deserialization error.
    #[error("Deserialization error: {0}")]
    Deserialization(String),

    /// Compression error.
    #[error("Compression error: {0}")]
    Compression(String),

    /// Decompression error.
    #[error("Decompression error: {0}")]
    Decompression(String),

    /// Recipe context initialization error.
    #[error("Failed to initialize recipe context: {0}")]
    ContextInit(String),

    /// Feature not enabled.
    #[error("Feature not enabled: {feature} (enable with --features {feature})")]
    FeatureNotEnabled {
        /// The required feature.
        feature: String,
    },

    /// Encryption error (requires `encryption` feature).
    #[cfg(feature = "encryption")]
    #[error("Encryption error: {0}")]
    Encryption(String),

    /// Decryption error (requires `encryption` feature).
    #[cfg(feature = "encryption")]
    #[error("Decryption error: {0}")]
    Decryption(String),

    /// Signing error (requires `signing` feature).
    #[cfg(feature = "signing")]
    #[error("Signing error: {0}")]
    Signing(String),

    /// Signature verification error (requires `signing` feature).
    #[cfg(feature = "signing")]
    #[error("Signature verification failed: {0}")]
    SignatureVerification(String),
}

/// Result type alias for ALD Cookbook operations.
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_dataset_not_found() {
        let err = Error::DatasetNotFound(PathBuf::from("/tmp/missing.ald"));
        assert_eq!(err.to_string(), "Dataset not found: /tmp/missing.ald");
    }

    #[test]
    fn test_error_display_invalid_format() {
        let err = Error::InvalidFormat {
            expected: "ALDF".to_string(),
            actual: "PARQ".to_string(),
        };
        assert_eq!(err.to_string(), "Invalid format: expected ALDF, got PARQ");
    }

    #[test]
    fn test_error_display_invalid_magic() {
        let err = Error::InvalidMagic(0x5041_5251);
        assert_eq!(
            err.to_string(),
            "Invalid magic bytes: expected ALDF (0x414C4446), got 0x50415251"
        );
    }

    #[test]
    fn test_error_display_unsupported_version() {
        let err = Error::UnsupportedVersion { major: 2, minor: 0 };
        assert_eq!(err.to_string(), "Unsupported version: 2.0 (supported: 1.x)");
    }

    #[test]
    fn test_error_display_checksum_mismatch() {
        let err = Error::ChecksumMismatch {
            expected: 0x1234_5678,
            actual: 0xABCD_EF01,
        };
        assert_eq!(
            err.to_string(),
            "Checksum mismatch: expected 0x12345678, got 0xABCDEF01"
        );
    }

    #[test]
    fn test_error_display_row_index_out_of_bounds() {
        let err = Error::RowIndexOutOfBounds {
            index: 100,
            total: 50,
        };
        assert_eq!(
            err.to_string(),
            "Row index 100 out of bounds (dataset has 50 rows)"
        );
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: Error = io_err.into();
        assert!(matches!(err, Error::Io(_)));
    }

    #[test]
    fn test_result_type_alias() {
        fn succeeds() -> Result<i32> {
            Ok(42)
        }

        fn fails() -> Result<i32> {
            Err(Error::EmptyDataset)
        }

        assert_eq!(succeeds().unwrap(), 42);
        assert!(fails().is_err());
    }
}
