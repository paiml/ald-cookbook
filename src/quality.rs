//! Data quality assurance utilities.
//!
//! Provides checks for data quality issues:
//! - Null values
//! - Duplicates
//! - Outliers
//! - Schema validation

#![allow(clippy::cast_precision_loss)]

use crate::error::{Error, Result};use arrow::array::{Array, Float64Array, Int64Array, RecordBatch, StringArray};
use arrow::datatypes::{DataType, Schema};
use std::collections::{HashMap, HashSet};

/// Statistics for null values in a column.
#[derive(Debug, Clone)]
pub struct NullStats {
    /// Total count of values.
    pub total_count: usize,
    /// Count of null values.
    pub null_count: usize,
}

impl NullStats {
    /// Calculate null percentage.
    #[must_use]
    pub fn null_percentage(&self) -> f64 {
        if self.total_count == 0 {
            0.0
        } else {
            (self.null_count as f64 / self.total_count as f64) * 100.0
        }
    }
}

/// Report of null values across all columns.
#[derive(Debug, Clone)]
pub struct NullReport {
    /// Statistics per column.
    pub columns: HashMap<String, NullStats>,
    /// Total rows in dataset.
    pub total_rows: usize,
}

impl NullReport {
    /// Calculate overall null percentage across all columns.
    #[must_use]
    pub fn overall_null_percentage(&self) -> f64 {
        let total_cells: usize = self.columns.values().map(|s| s.total_count).sum();
        let total_nulls: usize = self.columns.values().map(|s| s.null_count).sum();

        if total_cells == 0 {
            0.0
        } else {
            (total_nulls as f64 / total_cells as f64) * 100.0
        }
    }
}

impl std::fmt::Display for NullReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Null Detection Report")?;
        writeln!(f, "{:-<50}", "")?;
        for (column, stats) in &self.columns {
            writeln!(
                f,
                "  {}: {}/{} nulls ({:.1}%)",
                column,
                stats.null_count,
                stats.total_count,
                stats.null_percentage()
            )?;
        }
        writeln!(f, "{:-<50}", "")?;
        writeln!(
            f,
            "Total: {:.1}% null values",
            self.overall_null_percentage()
        )?;
        Ok(())
    }
}

/// Analyze null values in a dataset.
///
/// # Errors
///
/// Returns `Error::EmptyDataset` if the batch is empty.
pub fn null_report(batch: &RecordBatch) -> Result<NullReport> {
    let mut columns = HashMap::new();

    for (i, field) in batch.schema().fields().iter().enumerate() {
        let col = batch.column(i);
        let stats = NullStats {
            total_count: col.len(),
            null_count: col.null_count(),
        };
        columns.insert(field.name().clone(), stats);
    }

    Ok(NullReport {
        columns,
        total_rows: batch.num_rows(),
    })
}

/// Result of duplicate detection.
#[derive(Debug, Clone)]
pub struct DuplicateReport {
    /// Number of duplicate rows.
    pub duplicate_count: usize,
    /// Total rows.
    pub total_rows: usize,
    /// Indices of duplicate rows.
    pub duplicate_indices: Vec<usize>,
}

impl DuplicateReport {
    /// Calculate duplicate percentage.
    #[must_use]
    pub fn duplicate_percentage(&self) -> f64 {
        if self.total_rows == 0 {
            0.0
        } else {
            (self.duplicate_count as f64 / self.total_rows as f64) * 100.0
        }
    }
}

impl std::fmt::Display for DuplicateReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Duplicate Detection Report")?;
        writeln!(f, "{:-<50}", "")?;
        writeln!(
            f,
            "  Duplicates: {}/{} ({:.1}%)",
            self.duplicate_count,
            self.total_rows,
            self.duplicate_percentage()
        )?;
        if !self.duplicate_indices.is_empty() && self.duplicate_indices.len() <= 10 {
            writeln!(f, "  Duplicate indices: {:?}", self.duplicate_indices)?;
        }
        Ok(())
    }
}

/// Detect duplicate rows based on specified columns.
///
/// # Arguments
///
/// * `batch` - The record batch to check
/// * `columns` - Column names to use for duplicate detection (empty = all columns)
///
/// # Errors
///
/// Returns `Error::ColumnNotFound` if a specified column doesn't exist.
pub fn find_duplicates(batch: &RecordBatch, columns: &[&str]) -> Result<DuplicateReport> {
    let cols_to_check: Vec<usize> = if columns.is_empty() {
        (0..batch.num_columns()).collect()
    } else {
        columns
            .iter()
            .map(|name| {
                batch
                    .schema()
                    .index_of(name)
                    .map_err(|_| Error::ColumnNotFound((*name).to_string()))
            })
            .collect::<Result<Vec<_>>>()?
    };

    let mut seen: HashSet<Vec<u8>> = HashSet::new();
    let mut duplicate_indices = Vec::new();

    for row_idx in 0..batch.num_rows() {
        let row_key = compute_row_key(batch, row_idx, &cols_to_check);
        if !seen.insert(row_key) {
            duplicate_indices.push(row_idx);
        }
    }

    Ok(DuplicateReport {
        duplicate_count: duplicate_indices.len(),
        total_rows: batch.num_rows(),
        duplicate_indices,
    })
}

/// Compute a hash key for a row.
fn compute_row_key(batch: &RecordBatch, row_idx: usize, col_indices: &[usize]) -> Vec<u8> {
    let mut key = Vec::new();

    for &col_idx in col_indices {
        let col = batch.column(col_idx);

        // Handle nulls
        if col.is_null(row_idx) {
            key.extend_from_slice(b"\x00NULL\x00");
            continue;
        }

        // Serialize value based on type
        match col.data_type() {
            DataType::Int64 => {
                if let Some(arr) = col.as_any().downcast_ref::<Int64Array>() {
                    key.extend_from_slice(&arr.value(row_idx).to_le_bytes());
                }
            }
            DataType::Float64 => {
                if let Some(arr) = col.as_any().downcast_ref::<Float64Array>() {
                    key.extend_from_slice(&arr.value(row_idx).to_le_bytes());
                }
            }
            DataType::Utf8 => {
                if let Some(arr) = col.as_any().downcast_ref::<StringArray>() {
                    key.extend_from_slice(arr.value(row_idx).as_bytes());
                }
            }
            _ => {
                // Generic: use debug representation
                key.extend_from_slice(format!("{col:?}").as_bytes());
            }
        }
        key.push(0xFF); // Separator
    }

    key
}

/// Result of outlier detection.
#[derive(Debug, Clone)]
pub struct OutlierReport {
    /// Column name.
    pub column: String,
    /// Number of outliers detected.
    pub outlier_count: usize,
    /// Total values.
    pub total_count: usize,
    /// Indices of outlier rows.
    pub outlier_indices: Vec<usize>,
    /// Lower bound used.
    pub lower_bound: f64,
    /// Upper bound used.
    pub upper_bound: f64,
    /// Method used.
    pub method: String,
}

impl std::fmt::Display for OutlierReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Outlier Detection Report ({})", self.method)?;
        writeln!(f, "{:-<50}", "")?;
        writeln!(f, "  Column: {}", self.column)?;
        writeln!(
            f,
            "  Outliers: {}/{} ({:.1}%)",
            self.outlier_count,
            self.total_count,
            if self.total_count == 0 {
                0.0
            } else {
                (self.outlier_count as f64 / self.total_count as f64) * 100.0
            }
        )?;
        writeln!(
            f,
            "  Bounds: [{:.2}, {:.2}]",
            self.lower_bound, self.upper_bound
        )?;
        Ok(())
    }
}

/// Detect outliers using the IQR (Interquartile Range) method.
///
/// Values outside [Q1 - k*IQR, Q3 + k*IQR] are considered outliers.
/// Default k=1.5 for regular outliers.
///
/// # Errors
///
/// Returns `Error::ColumnNotFound` if the column doesn't exist.
/// Returns `Error::InvalidColumnType` if the column is not Float64.
pub fn detect_outliers_iqr(batch: &RecordBatch, column: &str, k: f64) -> Result<OutlierReport> {
    let col_idx = batch
        .schema()
        .index_of(column)
        .map_err(|_| Error::ColumnNotFound(column.to_string()))?;

    let col = batch.column(col_idx);
    let f64_col =
        col.as_any()
            .downcast_ref::<Float64Array>()
            .ok_or_else(|| Error::InvalidColumnType {
                expected: "Float64".to_string(),
                actual: format!("{:?}", col.data_type()),
            })?;

    // Get non-null values and sort
    let mut values: Vec<(usize, f64)> = f64_col
        .iter()
        .enumerate()
        .filter_map(|(i, v)| v.map(|x| (i, x)))
        .collect();

    values.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    if values.is_empty() {
        return Ok(OutlierReport {
            column: column.to_string(),
            outlier_count: 0,
            total_count: 0,
            outlier_indices: vec![],
            lower_bound: 0.0,
            upper_bound: 0.0,
            method: "IQR".to_string(),
        });
    }

    // Calculate quartiles
    let n = values.len();
    let q1_idx = n / 4;
    let q3_idx = (3 * n) / 4;

    let q1 = values[q1_idx].1;
    let q3 = values[q3_idx].1;
    let iqr = q3 - q1;

    let lower_bound = k.mul_add(-iqr, q1);
    let upper_bound = k.mul_add(iqr, q3);

    let outlier_indices: Vec<usize> = values
        .iter()
        .filter(|(_, v)| *v < lower_bound || *v > upper_bound)
        .map(|(i, _)| *i)
        .collect();

    Ok(OutlierReport {
        column: column.to_string(),
        outlier_count: outlier_indices.len(),
        total_count: n,
        outlier_indices,
        lower_bound,
        upper_bound,
        method: format!("IQR (k={k})"),
    })
}

/// Detect outliers using Z-score method.
///
/// Values with |z-score| > threshold are considered outliers.
/// Default threshold=3.0.
///
/// # Errors
///
/// Returns errors if column not found or wrong type.
pub fn detect_outliers_zscore(
    batch: &RecordBatch,
    column: &str,
    threshold: f64,
) -> Result<OutlierReport> {
    let col_idx = batch
        .schema()
        .index_of(column)
        .map_err(|_| Error::ColumnNotFound(column.to_string()))?;

    let col = batch.column(col_idx);
    let f64_col =
        col.as_any()
            .downcast_ref::<Float64Array>()
            .ok_or_else(|| Error::InvalidColumnType {
                expected: "Float64".to_string(),
                actual: format!("{:?}", col.data_type()),
            })?;

    let values: Vec<(usize, f64)> = f64_col
        .iter()
        .enumerate()
        .filter_map(|(i, v)| v.map(|x| (i, x)))
        .collect();

    if values.is_empty() {
        return Ok(OutlierReport {
            column: column.to_string(),
            outlier_count: 0,
            total_count: 0,
            outlier_indices: vec![],
            lower_bound: 0.0,
            upper_bound: 0.0,
            method: "Z-score".to_string(),
        });
    }

    // Calculate mean and std
    let n = values.len() as f64;
    let mean: f64 = values.iter().map(|(_, v)| v).sum::<f64>() / n;
    let variance: f64 = values.iter().map(|(_, v)| (v - mean).powi(2)).sum::<f64>() / n;
    let std_dev = variance.sqrt();

    if std_dev == 0.0 {
        return Ok(OutlierReport {
            column: column.to_string(),
            outlier_count: 0,
            total_count: values.len(),
            outlier_indices: vec![],
            lower_bound: mean,
            upper_bound: mean,
            method: format!("Z-score (threshold={threshold})"),
        });
    }

    let lower_bound = threshold.mul_add(-std_dev, mean);
    let upper_bound = threshold.mul_add(std_dev, mean);

    let outlier_indices: Vec<usize> = values
        .iter()
        .filter(|(_, v)| {
            let z = (v - mean) / std_dev;
            z.abs() > threshold
        })
        .map(|(i, _)| *i)
        .collect();

    Ok(OutlierReport {
        column: column.to_string(),
        outlier_count: outlier_indices.len(),
        total_count: values.len(),
        outlier_indices,
        lower_bound,
        upper_bound,
        method: format!("Z-score (threshold={threshold})"),
    })
}

/// Schema validation result.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether validation passed.
    pub valid: bool,
    /// List of validation errors.
    pub errors: Vec<String>,
}

impl std::fmt::Display for ValidationResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.valid {
            writeln!(f, "Schema validation: PASSED")?;
        } else {
            writeln!(f, "Schema validation: FAILED")?;
            for err in &self.errors {
                writeln!(f, "  - {err}")?;
            }
        }
        Ok(())
    }
}

/// Validate a `RecordBatch` against an expected schema.
///
/// # Errors
///
/// This function doesn't error - it returns validation results.
#[must_use]
pub fn validate_schema(batch: &RecordBatch, expected: &Schema) -> ValidationResult {
    let mut errors = Vec::new();
    let actual = batch.schema();

    // Check number of columns
    if actual.fields().len() != expected.fields().len() {
        errors.push(format!(
            "Column count mismatch: expected {}, got {}",
            expected.fields().len(),
            actual.fields().len()
        ));
    }

    // Check each field
    for (i, expected_field) in expected.fields().iter().enumerate() {
        if let Some(actual_field) = actual.fields().get(i) {
            if actual_field.name() != expected_field.name() {
                errors.push(format!(
                    "Column {} name mismatch: expected '{}', got '{}'",
                    i,
                    expected_field.name(),
                    actual_field.name()
                ));
            }
            if actual_field.data_type() != expected_field.data_type() {
                errors.push(format!(
                    "Column '{}' type mismatch: expected {:?}, got {:?}",
                    expected_field.name(),
                    expected_field.data_type(),
                    actual_field.data_type()
                ));
            }
            if actual_field.is_nullable() != expected_field.is_nullable() {
                errors.push(format!(
                    "Column '{}' nullability mismatch: expected {}, got {}",
                    expected_field.name(),
                    expected_field.is_nullable(),
                    actual_field.is_nullable()
                ));
            }
        } else {
            errors.push(format!("Missing column: '{}'", expected_field.name()));
        }
    }

    ValidationResult {
        valid: errors.is_empty(),
        errors,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::datatypes::Field;
    use std::sync::Arc;

    fn create_test_batch() -> RecordBatch {
        let schema = Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("value", DataType::Float64, true),
            Field::new("label", DataType::Utf8, true),
        ]);

        // Values designed to have clear outliers: [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, None, 100.0]
        // With more values, 100.0 will be a clear outlier via IQR and Z-score
        let id_array = Int64Array::from(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let value_array = Float64Array::from(vec![
            Some(1.0),
            Some(2.0),
            Some(3.0),
            Some(4.0),
            Some(5.0),
            Some(6.0),
            Some(7.0),
            Some(8.0),
            None,
            Some(100.0),
        ]);
        let label_array = StringArray::from(vec![
            Some("a"),
            Some("b"),
            None,
            Some("d"),
            Some("e"),
            Some("a"),
            Some("b"),
            Some("c"),
            Some("d"),
            Some("e"),
        ]);

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

    fn create_batch_with_duplicates() -> RecordBatch {
        let schema = Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("value", DataType::Float64, false),
        ]);

        let id_array = Int64Array::from(vec![1, 2, 1, 3, 2]); // 1 and 2 are duplicated
        let value_array = Float64Array::from(vec![1.0, 2.0, 1.0, 3.0, 2.0]);

        RecordBatch::try_new(
            Arc::new(schema),
            vec![Arc::new(id_array), Arc::new(value_array)],
        )
        .unwrap()
    }

    #[test]
    fn test_null_report() {
        let batch = create_test_batch();
        let report = null_report(&batch).unwrap();

        assert_eq!(report.columns["id"].null_count, 0);
        assert_eq!(report.columns["value"].null_count, 1);
        assert_eq!(report.columns["label"].null_count, 1);
        assert_eq!(report.total_rows, 10);
    }

    #[test]
    fn test_null_percentage() {
        let stats = NullStats {
            total_count: 100,
            null_count: 25,
        };
        assert!((stats.null_percentage() - 25.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_find_duplicates_all_columns() {
        let batch = create_batch_with_duplicates();
        let report = find_duplicates(&batch, &[]).unwrap();

        assert_eq!(report.duplicate_count, 2);
        assert_eq!(report.total_rows, 5);
    }

    #[test]
    fn test_find_duplicates_specific_column() {
        let batch = create_batch_with_duplicates();
        let report = find_duplicates(&batch, &["id"]).unwrap();

        assert_eq!(report.duplicate_count, 2);
    }

    #[test]
    fn test_detect_outliers_iqr() {
        let batch = create_test_batch();
        let report = detect_outliers_iqr(&batch, "value", 1.5).unwrap();

        // 100.0 should be an outlier (at index 9 in the original array)
        assert_eq!(report.outlier_count, 1);
        assert!(report.outlier_indices.contains(&9));
    }

    #[test]
    fn test_detect_outliers_zscore() {
        let batch = create_test_batch();
        let report = detect_outliers_zscore(&batch, "value", 2.0).unwrap();

        // 100.0 should definitely be an outlier (at index 9)
        assert!(report.outlier_count >= 1);
        assert!(report.outlier_indices.contains(&9));
    }

    #[test]
    fn test_validate_schema_pass() {
        let batch = create_test_batch();
        let expected = Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("value", DataType::Float64, true),
            Field::new("label", DataType::Utf8, true),
        ]);

        let result = validate_schema(&batch, &expected);
        assert!(result.valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_schema_type_mismatch() {
        let batch = create_test_batch();
        let expected = Schema::new(vec![
            Field::new("id", DataType::Int32, false), // Wrong type
            Field::new("value", DataType::Float64, true),
            Field::new("label", DataType::Utf8, true),
        ]);

        let result = validate_schema(&batch, &expected);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("type mismatch")));
    }

    #[test]
    fn test_validate_schema_column_count_mismatch() {
        let batch = create_test_batch();
        let expected = Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("value", DataType::Float64, true),
        ]);

        let result = validate_schema(&batch, &expected);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("Column count")));
    }

    #[test]
    fn test_outlier_column_not_found() {
        let batch = create_test_batch();
        let result = detect_outliers_iqr(&batch, "nonexistent", 1.5);
        assert!(matches!(result, Err(Error::ColumnNotFound(_))));
    }

    #[test]
    fn test_outlier_wrong_type() {
        let batch = create_test_batch();
        let result = detect_outliers_iqr(&batch, "label", 1.5);
        assert!(matches!(result, Err(Error::InvalidColumnType { .. })));
    }
}
