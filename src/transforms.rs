//! Data transformation utilities.
//!
//! Provides filter, map, shuffle, sample, and normalize operations
//! following the IIUR principles with deterministic behavior.

#![allow(clippy::cast_precision_loss)]

use crate::error::{Error, Result};
use arrow::array::{
    Array, ArrayRef, BooleanArray, Float64Array, Int64Array, RecordBatch, UInt64Array,
};
use arrow::compute::{filter_record_batch, take};
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::Rng;
use std::sync::Arc;

/// Filter a `RecordBatch` using a boolean mask.
///
/// # Errors
///
/// Returns `Error::Arrow` if the filter operation fails.
pub fn filter(batch: &RecordBatch, mask: &BooleanArray) -> Result<RecordBatch> {
    filter_record_batch(batch, mask).map_err(Error::from)
}

/// Filter a `RecordBatch` by column value predicate.
///
/// # Errors
///
/// Returns `Error::ColumnNotFound` if the column doesn't exist.
/// Returns `Error::InvalidColumnType` if the column type doesn't support comparison.
pub fn filter_by_column<F>(batch: &RecordBatch, column: &str, predicate: F) -> Result<RecordBatch>
where
    F: Fn(&dyn Array, usize) -> bool,
{
    let col_idx = batch
        .schema()
        .index_of(column)
        .map_err(|_| Error::ColumnNotFound(column.to_string()))?;

    let col = batch.column(col_idx);
    let mask: BooleanArray = (0..batch.num_rows())
        .map(|i| Some(predicate(col.as_ref(), i)))
        .collect();

    filter(batch, &mask)
}

/// Filter rows where a Float64 column exceeds a threshold.
///
/// # Errors
///
/// Returns `Error::ColumnNotFound` if the column doesn't exist.
/// Returns `Error::InvalidColumnType` if the column is not Float64.
pub fn filter_gt_f64(batch: &RecordBatch, column: &str, threshold: f64) -> Result<RecordBatch> {
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

    let mask: BooleanArray = f64_col.iter().map(|v| v.map(|x| x > threshold)).collect();

    filter(batch, &mask)
}

/// Filter rows where an Int64 column exceeds a threshold.
///
/// # Errors
///
/// Returns `Error::ColumnNotFound` if the column doesn't exist.
/// Returns `Error::InvalidColumnType` if the column is not Int64.
pub fn filter_gt_i64(batch: &RecordBatch, column: &str, threshold: i64) -> Result<RecordBatch> {
    let col_idx = batch
        .schema()
        .index_of(column)
        .map_err(|_| Error::ColumnNotFound(column.to_string()))?;

    let col = batch.column(col_idx);
    let i64_col =
        col.as_any()
            .downcast_ref::<Int64Array>()
            .ok_or_else(|| Error::InvalidColumnType {
                expected: "Int64".to_string(),
                actual: format!("{:?}", col.data_type()),
            })?;

    let mask: BooleanArray = i64_col.iter().map(|v| v.map(|x| x > threshold)).collect();

    filter(batch, &mask)
}

/// Deterministically shuffle a `RecordBatch`.
///
/// # Errors
///
/// Returns `Error::Arrow` if the shuffle operation fails.
pub fn shuffle(batch: &RecordBatch, rng: &mut StdRng) -> Result<RecordBatch> {
    let num_rows = batch.num_rows();
    if num_rows == 0 {
        return Ok(batch.clone());
    }

    let mut indices: Vec<u64> = (0..num_rows as u64).collect();
    indices.shuffle(rng);

    let indices_array = UInt64Array::from(indices);
    take_by_indices(batch, &indices_array)
}

/// Take rows by index array.
fn take_by_indices(batch: &RecordBatch, indices: &UInt64Array) -> Result<RecordBatch> {
    let columns: Vec<ArrayRef> = batch
        .columns()
        .iter()
        .map(|col| take(col.as_ref(), indices, None).map_err(Error::from))
        .collect::<Result<Vec<_>>>()?;

    RecordBatch::try_new(batch.schema(), columns).map_err(Error::from)
}

/// Deterministically sample rows from a `RecordBatch`.
///
/// # Arguments
///
/// * `batch` - The source batch
/// * `n` - Number of samples to take
/// * `rng` - Deterministic RNG
/// * `replace` - Whether to sample with replacement
///
/// # Errors
///
/// Returns `Error::Arrow` if sampling fails.
pub fn sample(
    batch: &RecordBatch,
    n: usize,
    rng: &mut StdRng,
    replace: bool,
) -> Result<RecordBatch> {
    let num_rows = batch.num_rows();
    if num_rows == 0 {
        return Ok(batch.clone());
    }

    let indices: Vec<u64> = if replace {
        (0..n).map(|_| rng.gen_range(0..num_rows) as u64).collect()
    } else {
        let mut all_indices: Vec<u64> = (0..num_rows as u64).collect();
        all_indices.shuffle(rng);
        all_indices.into_iter().take(n).collect()
    };

    let indices_array = UInt64Array::from(indices);
    take_by_indices(batch, &indices_array)
}

/// Statistics for normalization.
#[derive(Debug, Clone)]
pub struct ColumnStats {
    /// Column name.
    pub name: String,
    /// Mean value.
    pub mean: f64,
    /// Standard deviation.
    pub std_dev: f64,
    /// Minimum value.
    pub min: f64,
    /// Maximum value.
    pub max: f64,
}

/// Compute statistics for a Float64 column.
///
/// # Errors
///
/// Returns `Error::ColumnNotFound` if the column doesn't exist.
/// Returns `Error::InvalidColumnType` if the column is not Float64.
pub fn compute_stats(batch: &RecordBatch, column: &str) -> Result<ColumnStats> {
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

    let values: Vec<f64> = f64_col.iter().flatten().collect();
    if values.is_empty() {
        return Ok(ColumnStats {
            name: column.to_string(),
            mean: 0.0,
            std_dev: 0.0,
            min: 0.0,
            max: 0.0,
        });
    }

    let sum: f64 = values.iter().sum();
    let mean = sum / values.len() as f64;

    let variance: f64 =
        values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
    let std_dev = variance.sqrt();

    let min = values.iter().copied().fold(f64::INFINITY, f64::min);
    let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);

    Ok(ColumnStats {
        name: column.to_string(),
        mean,
        std_dev,
        min,
        max,
    })
}

/// Z-score normalize a Float64 column.
///
/// # Errors
///
/// Returns errors if column not found or wrong type.
pub fn normalize_zscore(batch: &RecordBatch, column: &str) -> Result<RecordBatch> {
    let stats = compute_stats(batch, column)?;
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

    let normalized: Float64Array = f64_col
        .iter()
        .map(|v| {
            v.map(|x| {
                if stats.std_dev == 0.0 {
                    0.0
                } else {
                    (x - stats.mean) / stats.std_dev
                }
            })
        })
        .collect();

    let mut columns: Vec<ArrayRef> = batch.columns().to_vec();
    columns[col_idx] = Arc::new(normalized);

    RecordBatch::try_new(batch.schema(), columns).map_err(Error::from)
}

/// Min-max normalize a Float64 column to [0, 1].
///
/// # Errors
///
/// Returns errors if column not found or wrong type.
pub fn normalize_minmax(batch: &RecordBatch, column: &str) -> Result<RecordBatch> {
    let stats = compute_stats(batch, column)?;
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

    let range = stats.max - stats.min;
    let normalized: Float64Array = f64_col
        .iter()
        .map(|v| {
            v.map(|x| {
                if range == 0.0 {
                    0.5
                } else {
                    (x - stats.min) / range
                }
            })
        })
        .collect();

    let mut columns: Vec<ArrayRef> = batch.columns().to_vec();
    columns[col_idx] = Arc::new(normalized);

    RecordBatch::try_new(batch.schema(), columns).map_err(Error::from)
}

/// Map a function over a Float64 column.
///
/// # Errors
///
/// Returns errors if column not found or wrong type.
pub fn map_f64<F>(batch: &RecordBatch, column: &str, f: F) -> Result<RecordBatch>
where
    F: Fn(f64) -> f64,
{
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

    let mapped: Float64Array = f64_col.iter().map(|v| v.map(&f)).collect();

    let mut columns: Vec<ArrayRef> = batch.columns().to_vec();
    columns[col_idx] = Arc::new(mapped);

    RecordBatch::try_new(batch.schema(), columns).map_err(Error::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::datatypes::{DataType, Field, Schema};
    use rand::SeedableRng;

    fn create_test_batch() -> RecordBatch {
        let schema = Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("value", DataType::Float64, false),
        ]);

        let id_array = Int64Array::from(vec![1, 2, 3, 4, 5]);
        let value_array = Float64Array::from(vec![1.0, 2.0, 3.0, 4.0, 5.0]);

        RecordBatch::try_new(
            Arc::new(schema),
            vec![Arc::new(id_array), Arc::new(value_array)],
        )
        .unwrap()
    }

    #[test]
    fn test_filter_by_mask() {
        let batch = create_test_batch();
        let mask = BooleanArray::from(vec![true, false, true, false, true]);

        let filtered = filter(&batch, &mask).unwrap();
        assert_eq!(filtered.num_rows(), 3);
    }

    #[test]
    fn test_filter_gt_f64() {
        let batch = create_test_batch();
        let filtered = filter_gt_f64(&batch, "value", 2.5).unwrap();
        assert_eq!(filtered.num_rows(), 3); // 3.0, 4.0, 5.0
    }

    #[test]
    fn test_filter_gt_i64() {
        let batch = create_test_batch();
        let filtered = filter_gt_i64(&batch, "id", 3).unwrap();
        assert_eq!(filtered.num_rows(), 2); // 4, 5
    }

    #[test]
    fn test_filter_column_not_found() {
        let batch = create_test_batch();
        let result = filter_gt_f64(&batch, "nonexistent", 0.0);
        assert!(matches!(result, Err(Error::ColumnNotFound(_))));
    }

    #[test]
    fn test_shuffle_deterministic() {
        let batch = create_test_batch();
        let mut rng1 = StdRng::seed_from_u64(42);
        let mut rng2 = StdRng::seed_from_u64(42);

        let shuffled1 = shuffle(&batch, &mut rng1).unwrap();
        let shuffled2 = shuffle(&batch, &mut rng2).unwrap();

        // Same seed should produce same shuffle
        let id1 = shuffled1.column(0);
        let id2 = shuffled2.column(0);
        assert_eq!(id1.as_ref(), id2.as_ref());
    }

    #[test]
    fn test_shuffle_changes_order() {
        let batch = create_test_batch();
        let mut rng = StdRng::seed_from_u64(42);

        let shuffled = shuffle(&batch, &mut rng).unwrap();

        // Should have same number of rows
        assert_eq!(shuffled.num_rows(), batch.num_rows());

        // Order should likely be different (not guaranteed but very probable)
        let original_ids: Vec<i64> = batch
            .column(0)
            .as_any()
            .downcast_ref::<Int64Array>()
            .unwrap()
            .iter()
            .map(|v| v.unwrap())
            .collect();

        let shuffled_ids: Vec<i64> = shuffled
            .column(0)
            .as_any()
            .downcast_ref::<Int64Array>()
            .unwrap()
            .iter()
            .map(|v| v.unwrap())
            .collect();

        // Order changed but same elements
        assert_ne!(original_ids, shuffled_ids);
        let mut sorted_shuffled = shuffled_ids.clone();
        sorted_shuffled.sort();
        assert_eq!(original_ids, sorted_shuffled);
    }

    #[test]
    fn test_sample_without_replacement() {
        let batch = create_test_batch();
        let mut rng = StdRng::seed_from_u64(42);

        let sampled = sample(&batch, 3, &mut rng, false).unwrap();
        assert_eq!(sampled.num_rows(), 3);
    }

    #[test]
    fn test_sample_with_replacement() {
        let batch = create_test_batch();
        let mut rng = StdRng::seed_from_u64(42);

        let sampled = sample(&batch, 10, &mut rng, true).unwrap();
        assert_eq!(sampled.num_rows(), 10);
    }

    #[test]
    fn test_compute_stats() {
        let batch = create_test_batch();
        let stats = compute_stats(&batch, "value").unwrap();

        assert_eq!(stats.name, "value");
        assert!((stats.mean - 3.0).abs() < f64::EPSILON);
        assert!((stats.min - 1.0).abs() < f64::EPSILON);
        assert!((stats.max - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_normalize_zscore() {
        let batch = create_test_batch();
        let normalized = normalize_zscore(&batch, "value").unwrap();

        let values = normalized
            .column(1)
            .as_any()
            .downcast_ref::<Float64Array>()
            .unwrap();

        // Mean should be ~0, std should be ~1
        let sum: f64 = values.iter().map(|v| v.unwrap()).sum();
        assert!((sum / 5.0).abs() < 1e-10); // Mean ~= 0
    }

    #[test]
    fn test_normalize_minmax() {
        let batch = create_test_batch();
        let normalized = normalize_minmax(&batch, "value").unwrap();

        let values = normalized
            .column(1)
            .as_any()
            .downcast_ref::<Float64Array>()
            .unwrap();

        let min_val = values
            .iter()
            .map(|v| v.unwrap())
            .fold(f64::INFINITY, f64::min);
        let max_val = values
            .iter()
            .map(|v| v.unwrap())
            .fold(f64::NEG_INFINITY, f64::max);

        assert!((min_val - 0.0).abs() < f64::EPSILON);
        assert!((max_val - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_map_f64() {
        let batch = create_test_batch();
        let mapped = map_f64(&batch, "value", |x| x * 2.0).unwrap();

        let values = mapped
            .column(1)
            .as_any()
            .downcast_ref::<Float64Array>()
            .unwrap();

        let expected = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        for (i, v) in values.iter().enumerate() {
            assert!((v.unwrap() - expected[i]).abs() < f64::EPSILON);
        }
    }
}
