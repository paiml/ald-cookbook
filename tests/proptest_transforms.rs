//! Property-based tests for the transforms module.
//!
//! These tests verify transformation invariants across randomly generated data.

use ald_cookbook::transforms::{filter_gt_f64, sample, shuffle};
use arrow::array::{Float64Array, Int64Array};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use proptest::prelude::*;
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::sync::Arc;

/// Create a test RecordBatch with the given values
fn create_test_batch(ids: Vec<i64>, values: Vec<f64>) -> RecordBatch {
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("value", DataType::Float64, false),
    ]));

    RecordBatch::try_new(
        schema,
        vec![
            Arc::new(Int64Array::from(ids)),
            Arc::new(Float64Array::from(values)),
        ],
    )
    .expect("valid batch")
}

/// Strategy for generating valid float values (avoiding NaN/Inf)
fn valid_float_strategy() -> impl Strategy<Value = f64> {
    -1000.0..1000.0f64
}

/// Strategy for generating test batches
fn batch_strategy() -> impl Strategy<Value = RecordBatch> {
    (10usize..500).prop_flat_map(|rows| {
        prop::collection::vec(valid_float_strategy(), rows).prop_map(move |values| {
            let ids: Vec<i64> = (0..values.len() as i64).collect();
            create_test_batch(ids, values)
        })
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    // ========================================================================
    // Filter Properties
    // ========================================================================

    /// Property: Filtering reduces or maintains row count
    #[test]
    fn filter_reduces_count(
        batch in batch_strategy(),
        threshold in -1000.0..1000.0f64
    ) {
        let filtered = filter_gt_f64(&batch, "value", threshold)
            .expect("filter should work");
        prop_assert!(filtered.num_rows() <= batch.num_rows());
    }

    /// Property: Filtering with very low threshold keeps all rows
    #[test]
    fn filter_low_threshold_keeps_all(batch in batch_strategy()) {
        let filtered = filter_gt_f64(&batch, "value", -10000.0)
            .expect("filter should work");
        prop_assert_eq!(filtered.num_rows(), batch.num_rows());
    }

    /// Property: Filtering with very high threshold removes all rows
    #[test]
    fn filter_high_threshold_removes_all(batch in batch_strategy()) {
        let filtered = filter_gt_f64(&batch, "value", 10000.0)
            .expect("filter should work");
        prop_assert_eq!(filtered.num_rows(), 0);
    }

    /// Property: Filter preserves column count
    #[test]
    fn filter_preserves_columns(
        batch in batch_strategy(),
        threshold in -1000.0..1000.0f64
    ) {
        let filtered = filter_gt_f64(&batch, "value", threshold)
            .expect("filter should work");
        prop_assert_eq!(filtered.num_columns(), batch.num_columns());
    }

    /// Property: Filter preserves schema
    #[test]
    fn filter_preserves_schema(
        batch in batch_strategy(),
        threshold in -1000.0..1000.0f64
    ) {
        let filtered = filter_gt_f64(&batch, "value", threshold)
            .expect("filter should work");
        prop_assert_eq!(filtered.schema(), batch.schema());
    }

    // ========================================================================
    // Shuffle Properties
    // ========================================================================

    /// Property: Shuffle preserves row count exactly
    #[test]
    fn shuffle_preserves_count(batch in batch_strategy(), seed in any::<u64>()) {
        let mut rng = StdRng::seed_from_u64(seed);
        let shuffled = shuffle(&batch, &mut rng).expect("shuffle should work");
        prop_assert_eq!(shuffled.num_rows(), batch.num_rows());
    }

    /// Property: Shuffle preserves column count
    #[test]
    fn shuffle_preserves_columns(batch in batch_strategy(), seed in any::<u64>()) {
        let mut rng = StdRng::seed_from_u64(seed);
        let shuffled = shuffle(&batch, &mut rng).expect("shuffle should work");
        prop_assert_eq!(shuffled.num_columns(), batch.num_columns());
    }

    /// Property: Shuffle with same seed is deterministic
    #[test]
    fn shuffle_deterministic(batch in batch_strategy(), seed in any::<u64>()) {
        let mut rng1 = StdRng::seed_from_u64(seed);
        let mut rng2 = StdRng::seed_from_u64(seed);

        let shuffled1 = shuffle(&batch, &mut rng1).expect("shuffle should work");
        let shuffled2 = shuffle(&batch, &mut rng2).expect("shuffle should work");

        // Check first column values match
        let col1 = shuffled1.column(0).as_any().downcast_ref::<Int64Array>().unwrap();
        let col2 = shuffled2.column(0).as_any().downcast_ref::<Int64Array>().unwrap();

        for i in 0..col1.len() {
            prop_assert_eq!(col1.value(i), col2.value(i));
        }
    }

    /// Property: Shuffle preserves schema
    #[test]
    fn shuffle_preserves_schema(batch in batch_strategy(), seed in any::<u64>()) {
        let mut rng = StdRng::seed_from_u64(seed);
        let shuffled = shuffle(&batch, &mut rng).expect("shuffle should work");
        prop_assert_eq!(shuffled.schema(), batch.schema());
    }

    /// Property: Different seeds produce different orderings (with high probability)
    #[test]
    fn shuffle_different_seeds_differ(
        rows in 100usize..300,
        seed1 in any::<u64>(),
        seed2 in any::<u64>()
    ) {
        prop_assume!(seed1 != seed2);

        let ids: Vec<i64> = (0..rows as i64).collect();
        let values: Vec<f64> = (0..rows).map(|i| i as f64).collect();
        let batch = create_test_batch(ids, values);

        let mut rng1 = StdRng::seed_from_u64(seed1);
        let mut rng2 = StdRng::seed_from_u64(seed2);

        let shuffled1 = shuffle(&batch, &mut rng1).expect("shuffle should work");
        let shuffled2 = shuffle(&batch, &mut rng2).expect("shuffle should work");

        let col1 = shuffled1.column(0).as_any().downcast_ref::<Int64Array>().unwrap();
        let col2 = shuffled2.column(0).as_any().downcast_ref::<Int64Array>().unwrap();

        // Count differences - with different seeds, most values should differ in position
        let differences: usize = (0..col1.len())
            .filter(|&i| col1.value(i) != col2.value(i))
            .count();

        // At least 10% should be different (very conservative)
        prop_assert!(differences > rows / 10);
    }

    // ========================================================================
    // Sample Properties
    // ========================================================================

    /// Property: Sample returns requested number of rows (without replacement)
    #[test]
    fn sample_without_replacement_count(
        batch in batch_strategy(),
        seed in any::<u64>()
    ) {
        let n = batch.num_rows() / 2;  // Sample half
        let mut rng = StdRng::seed_from_u64(seed);
        let sampled = sample(&batch, n, &mut rng, false).expect("sample should work");
        prop_assert_eq!(sampled.num_rows(), n);
    }

    /// Property: Sample with replacement can return more rows
    #[test]
    fn sample_with_replacement_count(
        batch in batch_strategy(),
        seed in any::<u64>()
    ) {
        let n = batch.num_rows() * 2;  // Sample double
        let mut rng = StdRng::seed_from_u64(seed);
        let sampled = sample(&batch, n, &mut rng, true).expect("sample should work");
        prop_assert_eq!(sampled.num_rows(), n);
    }

    /// Property: Sample with n=0 returns empty
    #[test]
    fn sample_zero_empty(batch in batch_strategy(), seed in any::<u64>()) {
        let mut rng = StdRng::seed_from_u64(seed);
        let sampled = sample(&batch, 0, &mut rng, false).expect("sample should work");
        prop_assert_eq!(sampled.num_rows(), 0);
    }

    /// Property: Sample preserves column structure
    #[test]
    fn sample_preserves_columns(
        batch in batch_strategy(),
        seed in any::<u64>()
    ) {
        let n = batch.num_rows() / 2;
        let mut rng = StdRng::seed_from_u64(seed);
        let sampled = sample(&batch, n, &mut rng, false).expect("sample should work");
        prop_assert_eq!(sampled.num_columns(), batch.num_columns());
        prop_assert_eq!(sampled.schema(), batch.schema());
    }

    /// Property: Sample is deterministic with same seed
    #[test]
    fn sample_deterministic(
        batch in batch_strategy(),
        seed in any::<u64>()
    ) {
        let n = batch.num_rows() / 2;
        let mut rng1 = StdRng::seed_from_u64(seed);
        let mut rng2 = StdRng::seed_from_u64(seed);

        let sampled1 = sample(&batch, n, &mut rng1, false).expect("sample should work");
        let sampled2 = sample(&batch, n, &mut rng2, false).expect("sample should work");

        prop_assert_eq!(sampled1.num_rows(), sampled2.num_rows());

        // Check values match
        let col1 = sampled1.column(0).as_any().downcast_ref::<Int64Array>().unwrap();
        let col2 = sampled2.column(0).as_any().downcast_ref::<Int64Array>().unwrap();
        for i in 0..col1.len() {
            prop_assert_eq!(col1.value(i), col2.value(i));
        }
    }
}

#[cfg(test)]
mod adversarial_tests {
    use super::*;

    #[test]
    fn test_filter_nonexistent_column() {
        let batch = create_test_batch(vec![1, 2, 3], vec![1.0, 2.0, 3.0]);
        let result = filter_gt_f64(&batch, "nonexistent", 0.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_shuffle_empty_batch() {
        let batch = create_test_batch(vec![], vec![]);
        let mut rng = StdRng::seed_from_u64(42);
        let result = shuffle(&batch, &mut rng);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().num_rows(), 0);
    }

    #[test]
    fn test_sample_more_than_available_without_replacement() {
        let batch = create_test_batch(vec![1, 2, 3], vec![1.0, 2.0, 3.0]);
        let mut rng = StdRng::seed_from_u64(42);
        // Sample more than available without replacement should cap at batch size
        let result = sample(&batch, 10, &mut rng, false);
        assert!(result.is_ok());
        // Should be capped at batch size
        assert!(result.unwrap().num_rows() <= 3);
    }

    #[test]
    fn test_sample_empty_batch() {
        let batch = create_test_batch(vec![], vec![]);
        let mut rng = StdRng::seed_from_u64(42);
        let result = sample(&batch, 5, &mut rng, false);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().num_rows(), 0);
    }
}
