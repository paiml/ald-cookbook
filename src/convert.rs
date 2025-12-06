//! Format conversion utilities.
//!
//! Provides conversions between ALD and other common data formats:
//! - Parquet
//! - CSV
//! - JSON Lines

#![allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]

use crate::error::{Error, Result};
use crate::format::{self, DatasetType, SaveOptions};
use arrow::array::RecordBatch;
use arrow::csv as arrow_csv;
use arrow::datatypes::Schema;
use arrow::json as arrow_json;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use parquet::arrow::ArrowWriter;
use parquet::basic::ZstdLevel;
use parquet::file::properties::WriterProperties;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::sync::Arc;

/// Options for CSV conversion.
#[derive(Debug, Clone, Copy)]
pub struct CsvOptions {
    /// Whether the first row is a header.
    pub has_header: bool,
    /// Field delimiter.
    pub delimiter: u8,
    /// Maximum records to infer schema from.
    pub schema_infer_max_records: usize,
    /// Batch size for reading.
    pub batch_size: usize,
}

impl Default for CsvOptions {
    fn default() -> Self {
        Self {
            has_header: true,
            delimiter: b',',
            schema_infer_max_records: 1000,
            batch_size: 8192,
        }
    }
}

/// Options for Parquet conversion.
#[derive(Debug, Clone, Copy)]
pub struct ParquetOptions {
    /// Batch size for reading.
    pub batch_size: usize,
    /// Compression for writing.
    pub compression: ParquetCompression,
}

impl Default for ParquetOptions {
    fn default() -> Self {
        Self {
            batch_size: 8192,
            compression: ParquetCompression::Zstd,
        }
    }
}

/// Parquet compression types.
#[derive(Debug, Clone, Copy)]
pub enum ParquetCompression {
    /// No compression.
    Uncompressed,
    /// Snappy compression.
    Snappy,
    /// Zstd compression.
    Zstd,
    /// LZ4 compression.
    Lz4,
}

impl From<ParquetCompression> for parquet::basic::Compression {
    fn from(c: ParquetCompression) -> Self {
        match c {
            ParquetCompression::Uncompressed => Self::UNCOMPRESSED,
            ParquetCompression::Snappy => Self::SNAPPY,
            ParquetCompression::Zstd => Self::ZSTD(ZstdLevel::default()),
            ParquetCompression::Lz4 => Self::LZ4,
        }
    }
}

/// Convert Parquet file to ALD format.
///
/// # Arguments
///
/// * `parquet_path` - Path to source Parquet file
/// * `ald_path` - Path for output ALD file
/// * `options` - Parquet reading options
///
/// # Errors
///
/// Returns errors if file operations or conversion fails.
pub fn parquet_to_ald(
    parquet_path: impl AsRef<Path>,
    ald_path: impl AsRef<Path>,
    options: ParquetOptions,
) -> Result<ConversionStats> {
    let parquet_path = parquet_path.as_ref();
    let ald_path = ald_path.as_ref();

    if !parquet_path.exists() {
        return Err(Error::DatasetNotFound(parquet_path.to_path_buf()));
    }

    let file = File::open(parquet_path)?;
    let builder = ParquetRecordBatchReaderBuilder::try_new(file)?;
    let mut reader = builder.with_batch_size(options.batch_size).build()?;

    // Collect all batches
    let batches: Vec<RecordBatch> = reader
        .by_ref()
        .collect::<std::result::Result<Vec<_>, _>>()?;

    if batches.is_empty() {
        return Err(Error::EmptyDataset);
    }

    // Concatenate batches
    let schema = batches[0].schema();
    let batch = arrow::compute::concat_batches(&schema, &batches)?;

    let source_rows = batch.num_rows();
    let source_size = std::fs::metadata(parquet_path)?.len();

    // Save to ALD
    format::save(&batch, DatasetType::Tabular, ald_path, SaveOptions::new())?;

    let dest_size = std::fs::metadata(ald_path)?.len();

    Ok(ConversionStats {
        source_format: "Parquet".to_string(),
        dest_format: "ALD".to_string(),
        rows: source_rows,
        columns: batch.num_columns(),
        source_size,
        dest_size,
    })
}

/// Convert ALD file to Parquet format.
///
/// # Errors
///
/// Returns errors if file operations or conversion fails.
pub fn ald_to_parquet(
    ald_path: impl AsRef<Path>,
    parquet_path: impl AsRef<Path>,
    options: ParquetOptions,
) -> Result<ConversionStats> {
    let ald_path = ald_path.as_ref();
    let parquet_path = parquet_path.as_ref();

    let batch = format::load(ald_path)?;
    let source_size = std::fs::metadata(ald_path)?.len();

    let file = File::create(parquet_path)?;
    let props = WriterProperties::builder()
        .set_compression(options.compression.into())
        .build();

    let mut writer = ArrowWriter::try_new(file, batch.schema(), Some(props))?;
    writer.write(&batch)?;
    writer.close()?;

    let dest_size = std::fs::metadata(parquet_path)?.len();

    Ok(ConversionStats {
        source_format: "ALD".to_string(),
        dest_format: "Parquet".to_string(),
        rows: batch.num_rows(),
        columns: batch.num_columns(),
        source_size,
        dest_size,
    })
}

/// Convert CSV file to ALD format.
///
/// # Errors
///
/// Returns errors if file operations or conversion fails.
pub fn csv_to_ald(
    csv_path: impl AsRef<Path>,
    ald_path: impl AsRef<Path>,
    options: CsvOptions,
) -> Result<ConversionStats> {
    let csv_path = csv_path.as_ref();
    let ald_path = ald_path.as_ref();

    if !csv_path.exists() {
        return Err(Error::DatasetNotFound(csv_path.to_path_buf()));
    }

    let file = File::open(csv_path)?;
    let source_size = file.metadata()?.len();

    let format = arrow_csv::reader::Format::default()
        .with_header(options.has_header)
        .with_delimiter(options.delimiter);

    // Infer schema
    let (schema, _) = format.infer_schema(
        BufReader::new(File::open(csv_path)?),
        Some(options.schema_infer_max_records),
    )?;

    let schema: Arc<Schema> = Arc::new(schema);

    // Read CSV
    let reader = arrow_csv::ReaderBuilder::new(schema.clone())
        .with_format(format)
        .with_batch_size(options.batch_size)
        .build(File::open(csv_path)?)?;

    let batches: Vec<RecordBatch> = reader.collect::<std::result::Result<Vec<_>, _>>()?;

    if batches.is_empty() {
        return Err(Error::EmptyDataset);
    }

    let batch = arrow::compute::concat_batches(&schema, &batches)?;

    // Save to ALD
    format::save(&batch, DatasetType::Tabular, ald_path, SaveOptions::new())?;

    let dest_size = std::fs::metadata(ald_path)?.len();

    Ok(ConversionStats {
        source_format: "CSV".to_string(),
        dest_format: "ALD".to_string(),
        rows: batch.num_rows(),
        columns: batch.num_columns(),
        source_size,
        dest_size,
    })
}

/// Convert ALD file to CSV format.
///
/// # Errors
///
/// Returns errors if file operations or conversion fails.
pub fn ald_to_csv(
    ald_path: impl AsRef<Path>,
    csv_path: impl AsRef<Path>,
    options: CsvOptions,
) -> Result<ConversionStats> {
    let ald_path = ald_path.as_ref();
    let csv_path = csv_path.as_ref();

    let batch = format::load(ald_path)?;
    let source_size = std::fs::metadata(ald_path)?.len();

    let file = File::create(csv_path)?;
    let mut writer = arrow_csv::WriterBuilder::new()
        .with_header(options.has_header)
        .with_delimiter(options.delimiter)
        .build(file);

    writer.write(&batch)?;

    let dest_size = std::fs::metadata(csv_path)?.len();

    Ok(ConversionStats {
        source_format: "ALD".to_string(),
        dest_format: "CSV".to_string(),
        rows: batch.num_rows(),
        columns: batch.num_columns(),
        source_size,
        dest_size,
    })
}

/// Convert JSON Lines file to ALD format.
///
/// # Errors
///
/// Returns errors if file operations or conversion fails.
pub fn jsonl_to_ald(
    jsonl_path: impl AsRef<Path>,
    ald_path: impl AsRef<Path>,
    batch_size: usize,
) -> Result<ConversionStats> {
    let jsonl_path = jsonl_path.as_ref();
    let ald_path = ald_path.as_ref();

    if !jsonl_path.exists() {
        return Err(Error::DatasetNotFound(jsonl_path.to_path_buf()));
    }

    let file = File::open(jsonl_path)?;
    let source_size = file.metadata()?.len();

    // Infer schema from first few lines
    let reader = BufReader::new(File::open(jsonl_path)?);
    let (schema, _) = arrow_json::reader::infer_json_schema(reader, Some(1000))?;
    let schema: Arc<Schema> = Arc::new(schema);

    // Read JSON Lines
    let file = File::open(jsonl_path)?;
    let reader = arrow_json::ReaderBuilder::new(schema.clone())
        .with_batch_size(batch_size)
        .build(BufReader::new(file))?;

    let batches: Vec<RecordBatch> = reader.collect::<std::result::Result<Vec<_>, _>>()?;

    if batches.is_empty() {
        return Err(Error::EmptyDataset);
    }

    let batch = arrow::compute::concat_batches(&schema, &batches)?;

    // Save to ALD
    format::save(&batch, DatasetType::Tabular, ald_path, SaveOptions::new())?;

    let dest_size = std::fs::metadata(ald_path)?.len();

    Ok(ConversionStats {
        source_format: "JSONL".to_string(),
        dest_format: "ALD".to_string(),
        rows: batch.num_rows(),
        columns: batch.num_columns(),
        source_size,
        dest_size,
    })
}

/// Conversion statistics.
#[derive(Debug, Clone)]
pub struct ConversionStats {
    /// Source format name.
    pub source_format: String,
    /// Destination format name.
    pub dest_format: String,
    /// Number of rows converted.
    pub rows: usize,
    /// Number of columns.
    pub columns: usize,
    /// Source file size in bytes.
    pub source_size: u64,
    /// Destination file size in bytes.
    pub dest_size: u64,
}

impl ConversionStats {
    /// Calculate compression ratio.
    #[must_use]
    pub fn compression_ratio(&self) -> f64 {
        if self.dest_size == 0 {
            0.0
        } else {
            self.source_size as f64 / self.dest_size as f64
        }
    }

    /// Calculate size change percentage.
    #[must_use]
    pub fn size_change_percent(&self) -> f64 {
        if self.source_size == 0 {
            0.0
        } else {
            ((self.dest_size as f64 - self.source_size as f64) / self.source_size as f64) * 100.0
        }
    }
}

impl std::fmt::Display for ConversionStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "Conversion: {} → {}",
            self.source_format, self.dest_format
        )?;
        writeln!(f, "{:-<50}", "")?;
        writeln!(f, "  Rows: {}", self.rows)?;
        writeln!(f, "  Columns: {}", self.columns)?;
        writeln!(f, "  Source size: {} KB", self.source_size / 1024)?;
        writeln!(f, "  Dest size: {} KB", self.dest_size / 1024)?;
        writeln!(f, "  Size change: {:.1}%", self.size_change_percent())?;
        Ok(())
    }
}

/// Count rows in a Parquet file.
///
/// # Errors
///
/// Returns errors if file reading fails.
pub fn count_parquet_rows(path: impl AsRef<Path>) -> Result<usize> {
    let file = File::open(path)?;
    let builder = ParquetRecordBatchReaderBuilder::try_new(file)?;
    let reader = builder.build()?;

    let total: usize = reader.map(|b| b.map(|r| r.num_rows()).unwrap_or(0)).sum();
    Ok(total)
}

/// Count rows in a CSV file.
///
/// # Errors
///
/// Returns errors if file reading fails.
pub fn count_csv_rows(path: impl AsRef<Path>, has_header: bool) -> Result<usize> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut count = reader.lines().count();
    if has_header && count > 0 {
        count -= 1;
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::{Float64Array, Int64Array, StringArray};
    use arrow::datatypes::{DataType, Field, Schema};
    use std::io::Write;
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

    fn create_test_parquet(path: &Path) -> Result<()> {
        let batch = create_test_batch();
        let file = File::create(path)?;
        let props = WriterProperties::builder().build();
        let mut writer = ArrowWriter::try_new(file, batch.schema(), Some(props))?;
        writer.write(&batch)?;
        writer.close()?;
        Ok(())
    }

    fn create_test_csv(path: &Path) -> Result<()> {
        let mut file = File::create(path)?;
        writeln!(file, "id,value,label")?;
        writeln!(file, "1,1.1,a")?;
        writeln!(file, "2,2.2,b")?;
        writeln!(file, "3,3.3,")?;
        writeln!(file, "4,4.4,d")?;
        writeln!(file, "5,5.5,e")?;
        Ok(())
    }

    fn create_test_jsonl(path: &Path) -> Result<()> {
        let mut file = File::create(path)?;
        writeln!(file, r#"{{"id":1,"value":1.1,"label":"a"}}"#)?;
        writeln!(file, r#"{{"id":2,"value":2.2,"label":"b"}}"#)?;
        writeln!(file, r#"{{"id":3,"value":3.3,"label":null}}"#)?;
        writeln!(file, r#"{{"id":4,"value":4.4,"label":"d"}}"#)?;
        writeln!(file, r#"{{"id":5,"value":5.5,"label":"e"}}"#)?;
        Ok(())
    }

    #[test]
    fn test_parquet_to_ald() {
        let temp = tempfile::tempdir().unwrap();
        let parquet_path = temp.path().join("test.parquet");
        let ald_path = temp.path().join("test.ald");

        create_test_parquet(&parquet_path).unwrap();

        let stats = parquet_to_ald(&parquet_path, &ald_path, ParquetOptions::default()).unwrap();

        assert_eq!(stats.rows, 5);
        assert_eq!(stats.columns, 3);
        assert!(ald_path.exists());

        // Verify roundtrip
        let loaded = format::load(&ald_path).unwrap();
        assert_eq!(loaded.num_rows(), 5);
    }

    #[test]
    fn test_ald_to_parquet() {
        let temp = tempfile::tempdir().unwrap();
        let ald_path = temp.path().join("test.ald");
        let parquet_path = temp.path().join("test.parquet");

        let batch = create_test_batch();
        format::save(&batch, DatasetType::Tabular, &ald_path, SaveOptions::new()).unwrap();

        let stats = ald_to_parquet(&ald_path, &parquet_path, ParquetOptions::default()).unwrap();

        assert_eq!(stats.rows, 5);
        assert!(parquet_path.exists());

        // Verify roundtrip
        let count = count_parquet_rows(&parquet_path).unwrap();
        assert_eq!(count, 5);
    }

    #[test]
    fn test_csv_to_ald() {
        let temp = tempfile::tempdir().unwrap();
        let csv_path = temp.path().join("test.csv");
        let ald_path = temp.path().join("test.ald");

        create_test_csv(&csv_path).unwrap();

        let stats = csv_to_ald(&csv_path, &ald_path, CsvOptions::default()).unwrap();

        assert_eq!(stats.rows, 5);
        assert!(ald_path.exists());

        let loaded = format::load(&ald_path).unwrap();
        assert_eq!(loaded.num_rows(), 5);
    }

    #[test]
    fn test_ald_to_csv() {
        let temp = tempfile::tempdir().unwrap();
        let ald_path = temp.path().join("test.ald");
        let csv_path = temp.path().join("test.csv");

        let batch = create_test_batch();
        format::save(&batch, DatasetType::Tabular, &ald_path, SaveOptions::new()).unwrap();

        let stats = ald_to_csv(&ald_path, &csv_path, CsvOptions::default()).unwrap();

        assert_eq!(stats.rows, 5);
        assert!(csv_path.exists());

        let count = count_csv_rows(&csv_path, true).unwrap();
        assert_eq!(count, 5);
    }

    #[test]
    fn test_jsonl_to_ald() {
        let temp = tempfile::tempdir().unwrap();
        let jsonl_path = temp.path().join("test.jsonl");
        let ald_path = temp.path().join("test.ald");

        create_test_jsonl(&jsonl_path).unwrap();

        let stats = jsonl_to_ald(&jsonl_path, &ald_path, 1024).unwrap();

        assert_eq!(stats.rows, 5);
        assert!(ald_path.exists());

        let loaded = format::load(&ald_path).unwrap();
        assert_eq!(loaded.num_rows(), 5);
    }

    #[test]
    fn test_conversion_stats_display() {
        let stats = ConversionStats {
            source_format: "CSV".to_string(),
            dest_format: "ALD".to_string(),
            rows: 1000,
            columns: 10,
            source_size: 50000,
            dest_size: 25000,
        };

        let display = format!("{stats}");
        assert!(display.contains("CSV → ALD"));
        assert!(display.contains("1000"));
    }

    #[test]
    fn test_compression_ratio() {
        let stats = ConversionStats {
            source_format: "CSV".to_string(),
            dest_format: "ALD".to_string(),
            rows: 1000,
            columns: 10,
            source_size: 100,
            dest_size: 50,
        };

        assert!((stats.compression_ratio() - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_file_not_found() {
        let temp = tempfile::tempdir().unwrap();
        let result = parquet_to_ald(
            temp.path().join("nonexistent.parquet"),
            temp.path().join("out.ald"),
            ParquetOptions::default(),
        );

        assert!(matches!(result, Err(Error::DatasetNotFound(_))));
    }
}
