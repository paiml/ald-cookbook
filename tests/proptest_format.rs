//! Property-based tests for the ALD format module.
//!
//! These tests verify format invariants across randomly generated inputs.

use ald_cookbook::format::{
    DatasetType, FormatFlags, Header, Metadata, SaveOptions, ALD_MAGIC, HEADER_SIZE, VERSION_MAJOR,
    VERSION_MINOR,
};
use proptest::prelude::*;
use std::io::Cursor;

/// Strategy for generating valid dataset names
fn dataset_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{2,30}".prop_map(|s| s.to_string())
}

/// Strategy for generating valid descriptions
fn description_strategy() -> impl Strategy<Value = String> {
    "[A-Za-z0-9 .,!?]{0,200}".prop_map(|s| s.to_string())
}

/// Strategy for generating row counts
fn row_count_strategy() -> impl Strategy<Value = usize> {
    1usize..1_000_000
}

/// Strategy for generating column counts
fn column_count_strategy() -> impl Strategy<Value = usize> {
    1usize..100
}

/// Strategy for generating dataset types
fn dataset_type_strategy() -> impl Strategy<Value = DatasetType> {
    prop_oneof![
        Just(DatasetType::Tabular),
        Just(DatasetType::TimeSeries),
        Just(DatasetType::TextCorpus),
        Just(DatasetType::ImageClassification),
        Just(DatasetType::Binary),
    ]
}

/// Strategy for generating compression levels
fn compression_level_strategy() -> impl Strategy<Value = i32> {
    1i32..22
}

/// Strategy for generating metadata lengths
fn length_strategy() -> impl Strategy<Value = u32> {
    1u32..1_000_000
}

/// Strategy for generating payload lengths
fn payload_length_strategy() -> impl Strategy<Value = u64> {
    1u64..1_000_000_000
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    // ========================================================================
    // Header Properties
    // ========================================================================

    /// Property: Header write/read roundtrip preserves all fields
    #[test]
    fn header_roundtrip(
        metadata_len in length_strategy(),
        schema_len in length_strategy(),
        payload_len in payload_length_strategy()
    ) {
        let flags = FormatFlags::default();
        let header = Header::new(metadata_len, schema_len, payload_len, flags);

        // Write to buffer
        let mut buffer = Vec::new();
        header.write(&mut buffer).expect("write should succeed");

        // Read back
        let mut cursor = Cursor::new(buffer);
        let parsed = Header::read(&mut cursor).expect("read should succeed");

        prop_assert_eq!(header.magic, parsed.magic);
        prop_assert_eq!(header.version_major, parsed.version_major);
        prop_assert_eq!(header.version_minor, parsed.version_minor);
        prop_assert_eq!(header.metadata_len, parsed.metadata_len);
        prop_assert_eq!(header.schema_len, parsed.schema_len);
        prop_assert_eq!(header.payload_len, parsed.payload_len);
    }

    /// Property: Header size is always exactly HEADER_SIZE bytes
    #[test]
    fn header_size_constant(
        metadata_len in length_strategy(),
        schema_len in length_strategy(),
        payload_len in payload_length_strategy()
    ) {
        let header = Header::new(metadata_len, schema_len, payload_len, FormatFlags::default());
        let mut buffer = Vec::new();
        header.write(&mut buffer).expect("write should succeed");
        prop_assert_eq!(buffer.len(), HEADER_SIZE);
    }

    /// Property: Magic bytes are always ALD_MAGIC
    #[test]
    fn header_magic_preserved(
        metadata_len in length_strategy(),
        payload_len in payload_length_strategy()
    ) {
        let header = Header::new(metadata_len, 0, payload_len, FormatFlags::default());
        prop_assert_eq!(header.magic, ALD_MAGIC);
    }

    /// Property: Version is always (VERSION_MAJOR, VERSION_MINOR) for new headers
    #[test]
    fn header_version_constant(
        metadata_len in length_strategy(),
        payload_len in payload_length_strategy()
    ) {
        let header = Header::new(metadata_len, 0, payload_len, FormatFlags::default());
        prop_assert_eq!(header.version_major, VERSION_MAJOR);
        prop_assert_eq!(header.version_minor, VERSION_MINOR);
    }

    // ========================================================================
    // Metadata Properties
    // ========================================================================

    /// Property: Metadata roundtrip via MessagePack preserves all fields
    #[test]
    fn metadata_roundtrip(
        rows in row_count_strategy(),
        cols in column_count_strategy(),
        dtype in dataset_type_strategy()
    ) {
        let metadata = Metadata::new(dtype, rows, cols);
        let bytes = metadata.to_msgpack().expect("serialization");
        let parsed = Metadata::from_msgpack(&bytes).expect("deserialization");

        prop_assert_eq!(metadata.num_rows, parsed.num_rows);
        prop_assert_eq!(metadata.num_columns, parsed.num_columns);
        prop_assert_eq!(metadata.dataset_type, parsed.dataset_type);
    }

    /// Property: Metadata with name roundtrips correctly
    #[test]
    fn metadata_with_name_roundtrip(
        name in dataset_name_strategy(),
        rows in row_count_strategy()
    ) {
        let metadata = Metadata::new(DatasetType::Tabular, rows, 1)
            .with_name(&name);
        let bytes = metadata.to_msgpack().expect("serialization");
        let parsed = Metadata::from_msgpack(&bytes).expect("deserialization");

        prop_assert_eq!(metadata.name, parsed.name);
    }

    /// Property: Metadata with description roundtrips correctly
    #[test]
    fn metadata_with_description_roundtrip(
        desc in description_strategy(),
        rows in row_count_strategy()
    ) {
        let metadata = Metadata::new(DatasetType::Tabular, rows, 1)
            .with_description(&desc);
        let bytes = metadata.to_msgpack().expect("serialization");
        let parsed = Metadata::from_msgpack(&bytes).expect("deserialization");

        prop_assert_eq!(metadata.description, parsed.description);
    }

    /// Property: Row count is preserved exactly
    #[test]
    fn metadata_row_count_preserved(rows in row_count_strategy()) {
        let metadata = Metadata::new(DatasetType::Tabular, rows, 1);
        prop_assert_eq!(metadata.num_rows, rows);
    }

    /// Property: Column count is preserved exactly
    #[test]
    fn metadata_column_count_preserved(cols in column_count_strategy()) {
        let metadata = Metadata::new(DatasetType::Tabular, 1, cols);
        prop_assert_eq!(metadata.num_columns, cols);
    }

    // ========================================================================
    // SaveOptions Properties
    // ========================================================================

    /// Property: SaveOptions preserves compression level
    #[test]
    fn save_options_compression_level(compression in compression_level_strategy()) {
        let options = SaveOptions::new().with_compression_level(compression);
        prop_assert_eq!(options.compression_level, compression);
    }

    /// Property: Default compression level is 3
    #[test]
    fn save_options_default_compression(_dummy in Just(())) {
        let options = SaveOptions::new();
        prop_assert_eq!(options.compression_level, 3);
    }

    /// Property: Default has compression enabled
    #[test]
    fn save_options_default_compress_enabled(_dummy in Just(())) {
        let options = SaveOptions::new();
        prop_assert!(options.compress);
    }

    /// Property: without_compression disables compression
    #[test]
    fn save_options_without_compression(_dummy in Just(())) {
        let options = SaveOptions::new().without_compression();
        prop_assert!(!options.compress);
    }

    /// Property: with_name sets name
    #[test]
    fn save_options_with_name(name in dataset_name_strategy()) {
        let options = SaveOptions::new().with_name(&name);
        prop_assert_eq!(options.name, Some(name));
    }

    // ========================================================================
    // FormatFlags Properties
    // ========================================================================

    /// Property: FormatFlags roundtrip via u32
    #[test]
    fn format_flags_roundtrip(
        encrypted in any::<bool>(),
        signed in any::<bool>(),
        streaming in any::<bool>(),
        compressed in any::<bool>()
    ) {
        let flags = FormatFlags {
            encrypted,
            signed,
            streaming,
            compressed,
        };
        let bits = flags.to_bits();
        let parsed = FormatFlags::from_bits(bits);

        prop_assert_eq!(flags.encrypted, parsed.encrypted);
        prop_assert_eq!(flags.signed, parsed.signed);
        prop_assert_eq!(flags.streaming, parsed.streaming);
        prop_assert_eq!(flags.compressed, parsed.compressed);
    }

    /// Property: Default flags have compression disabled
    #[test]
    fn format_flags_default(_dummy in Just(())) {
        let flags = FormatFlags::default();
        prop_assert!(!flags.encrypted);
        prop_assert!(!flags.signed);
        prop_assert!(!flags.streaming);
        prop_assert!(!flags.compressed);
    }

    // ========================================================================
    // Checksum Properties
    // ========================================================================

    /// Property: CRC32 checksum changes when data changes
    #[test]
    fn checksum_detects_changes(
        data in prop::collection::vec(any::<u8>(), 100..10000),
        position in 0usize..100
    ) {
        let checksum1 = crc32fast::hash(&data);

        // Flip one bit
        let mut modified = data.clone();
        let pos = position % modified.len();
        modified[pos] ^= 1;

        let checksum2 = crc32fast::hash(&modified);
        prop_assert_ne!(checksum1, checksum2);
    }

    /// Property: CRC32 is deterministic
    #[test]
    fn checksum_deterministic(
        data in prop::collection::vec(any::<u8>(), 100..10000)
    ) {
        let checksum1 = crc32fast::hash(&data);
        let checksum2 = crc32fast::hash(&data);
        prop_assert_eq!(checksum1, checksum2);
    }
}

#[cfg(test)]
mod adversarial_tests {
    use super::*;

    #[test]
    fn test_corrupted_magic_bytes() {
        let header = Header::new(100, 100, 1000, FormatFlags::default());
        let mut buffer = Vec::new();
        header.write(&mut buffer).expect("write should succeed");

        // Corrupt magic bytes
        buffer[0] = b'X';
        buffer[1] = b'X';

        let mut cursor = Cursor::new(buffer);
        let result = Header::read(&mut cursor);
        assert!(result.is_err());
    }

    #[test]
    fn test_truncated_header() {
        let header = Header::new(100, 100, 1000, FormatFlags::default());
        let mut buffer = Vec::new();
        header.write(&mut buffer).expect("write should succeed");

        // Truncate to less than HEADER_SIZE
        let truncated = &buffer[..HEADER_SIZE / 2];

        let mut cursor = Cursor::new(truncated);
        let result = Header::read(&mut cursor);
        assert!(result.is_err());
    }

    #[test]
    fn test_max_lengths() {
        let header = Header::new(u32::MAX, u32::MAX, u64::MAX, FormatFlags::default());
        let mut buffer = Vec::new();
        header.write(&mut buffer).expect("write should succeed");

        let mut cursor = Cursor::new(buffer);
        let parsed = Header::read(&mut cursor).expect("read should succeed");
        assert_eq!(parsed.metadata_len, u32::MAX);
        assert_eq!(parsed.schema_len, u32::MAX);
        assert_eq!(parsed.payload_len, u64::MAX);
    }

    #[test]
    fn test_all_flags_set() {
        let flags = FormatFlags {
            encrypted: true,
            signed: true,
            streaming: true,
            compressed: true,
        };
        let bits = flags.to_bits();
        assert_eq!(bits, 0b1111);
    }
}
