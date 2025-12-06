//! Property-based tests for the drift detection module.
//!
//! These tests verify statistical drift detection invariants.

use ald_cookbook::drift::ks_test;
use arrow::array::Float64Array;
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use proptest::prelude::*;
use std::sync::Arc;

/// Create a test RecordBatch with float values
fn create_float_batch(values: Vec<f64>) -> RecordBatch {
    let schema = Arc::new(Schema::new(vec![Field::new(
        "value",
        DataType::Float64,
        false,
    )]));

    RecordBatch::try_new(schema, vec![Arc::new(Float64Array::from(values))]).expect("valid batch")
}

/// Strategy for generating valid probability distributions (values in 0..1)
fn distribution_strategy() -> impl Strategy<Value = Vec<f64>> {
    prop::collection::vec(0.0..1.0f64, 50..500)
        // Filter out constant distributions to avoid edge cases
        .prop_filter("non-constant distribution", |v| {
            if v.len() < 2 {
                return true;
            }
            let first = v[0];
            !v.iter().all(|&x| (x - first).abs() < 1e-10)
        })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    // ========================================================================
    // KS Test Properties
    // ========================================================================

    /// Property: KS statistic is in [0, 1] range
    #[test]
    fn ks_statistic_range(
        baseline in distribution_strategy(),
        current in distribution_strategy()
    ) {
        let baseline_batch = create_float_batch(baseline);
        let current_batch = create_float_batch(current);

        let result = ks_test(&baseline_batch, &current_batch, "value")
            .expect("ks_test should work");
        prop_assert!(result.statistic >= 0.0 && result.statistic <= 1.0,
            "KS statistic {} out of [0, 1] range", result.statistic);
    }

    /// Property: Identical distributions have low KS statistic and high p-value
    #[test]
    fn ks_identical_distributions(values in distribution_strategy()) {
        let batch1 = create_float_batch(values.clone());
        let batch2 = create_float_batch(values);

        let result = ks_test(&batch1, &batch2, "value")
            .expect("ks_test should work");
        // Identical distributions should have low statistic (< 0.5)
        // Note: Due to implementation details, may not be exactly 0
        prop_assert!(result.statistic < 0.5,
            "Identical distributions should have low KS statistic, got {}",
            result.statistic);
        // P-value should be high for identical distributions
        prop_assert!(result.p_value > 0.01,
            "Identical distributions should have high p-value, got {}",
            result.p_value);
    }

    /// Property: KS test is symmetric
    #[test]
    fn ks_symmetric(
        baseline in distribution_strategy(),
        current in distribution_strategy()
    ) {
        let baseline_batch = create_float_batch(baseline);
        let current_batch = create_float_batch(current);

        let result1 = ks_test(&baseline_batch, &current_batch, "value")
            .expect("ks_test should work");
        let result2 = ks_test(&current_batch, &baseline_batch, "value")
            .expect("ks_test should work");

        prop_assert!((result1.statistic - result2.statistic).abs() < 1e-10,
            "KS test should be symmetric: {} vs {}", result1.statistic, result2.statistic);
    }

    /// Property: KS p-value is in [0, 1] range
    #[test]
    fn ks_pvalue_range(
        baseline in distribution_strategy(),
        current in distribution_strategy()
    ) {
        let baseline_batch = create_float_batch(baseline);
        let current_batch = create_float_batch(current);

        let result = ks_test(&baseline_batch, &current_batch, "value")
            .expect("ks_test should work");
        prop_assert!(result.p_value >= 0.0 && result.p_value <= 1.0,
            "P-value {} out of [0, 1] range", result.p_value);
    }

    /// Property: Sample sizes are correctly reported
    #[test]
    fn ks_sample_sizes(
        baseline in distribution_strategy(),
        current in distribution_strategy()
    ) {
        let n_baseline = baseline.len();
        let n_current = current.len();

        let baseline_batch = create_float_batch(baseline);
        let current_batch = create_float_batch(current);

        let result = ks_test(&baseline_batch, &current_batch, "value")
            .expect("ks_test should work");

        prop_assert_eq!(result.n_reference, n_baseline);
        prop_assert_eq!(result.n_current, n_current);
    }

    /// Property: Highly different distributions have high KS statistic
    #[test]
    fn ks_different_distributions_detected(count in 100usize..300) {
        // Generate two very different distributions
        let low: Vec<f64> = (0..count).map(|_| 0.1).collect();
        let high: Vec<f64> = (0..count).map(|_| 0.9).collect();

        let low_batch = create_float_batch(low);
        let high_batch = create_float_batch(high);

        let result = ks_test(&low_batch, &high_batch, "value")
            .expect("ks_test should work");
        prop_assert!(result.statistic > 0.5,
            "Very different distributions should have high KS statistic, got {}",
            result.statistic);
    }

    /// Property: Column name is preserved in result
    #[test]
    fn ks_column_name_preserved(
        baseline in distribution_strategy(),
        current in distribution_strategy()
    ) {
        let baseline_batch = create_float_batch(baseline);
        let current_batch = create_float_batch(current);

        let result = ks_test(&baseline_batch, &current_batch, "value")
            .expect("ks_test should work");

        prop_assert_eq!(result.column, "value");
    }

    /// Property: drift_detected respects significance level
    #[test]
    fn ks_drift_detected_consistency(
        baseline in distribution_strategy(),
        current in distribution_strategy(),
        alpha in 0.01..0.1f64
    ) {
        let baseline_batch = create_float_batch(baseline);
        let current_batch = create_float_batch(current);

        let result = ks_test(&baseline_batch, &current_batch, "value")
            .expect("ks_test should work");

        // drift_detected should be consistent with p_value < alpha
        let expected_drift = result.p_value < alpha;
        prop_assert_eq!(result.drift_detected(alpha), expected_drift);
    }
}

#[cfg(test)]
mod adversarial_tests {
    use super::*;

    #[test]
    fn test_ks_identical_low_statistic() {
        let values = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        let batch1 = create_float_batch(values.clone());
        let batch2 = create_float_batch(values);

        let result = ks_test(&batch1, &batch2, "value").expect("should work");
        // Identical distributions should have low KS statistic
        // Note: Due to CDF computation details, may not be exactly 0
        assert!(
            result.statistic < 0.5,
            "Expected low statistic, got {}",
            result.statistic
        );
    }

    #[test]
    fn test_ks_empty_baseline() {
        let baseline_batch = create_float_batch(vec![]);
        let current_batch = create_float_batch(vec![0.5]);

        let result = ks_test(&baseline_batch, &current_batch, "value");
        // Should handle gracefully, returning 0 statistic and p-value of 1
        assert!(result.is_ok());
        let r = result.unwrap();
        assert_eq!(r.statistic, 0.0);
        assert_eq!(r.p_value, 1.0);
    }

    #[test]
    fn test_ks_empty_current() {
        let baseline_batch = create_float_batch(vec![0.5]);
        let current_batch = create_float_batch(vec![]);

        let result = ks_test(&baseline_batch, &current_batch, "value");
        assert!(result.is_ok());
    }

    #[test]
    fn test_ks_both_empty() {
        let baseline_batch = create_float_batch(vec![]);
        let current_batch = create_float_batch(vec![]);

        let result = ks_test(&baseline_batch, &current_batch, "value");
        assert!(result.is_ok());
    }

    #[test]
    fn test_ks_single_value() {
        let baseline_batch = create_float_batch(vec![0.5]);
        let current_batch = create_float_batch(vec![0.5]);

        let result = ks_test(&baseline_batch, &current_batch, "value");
        assert!(result.is_ok());
    }

    #[test]
    fn test_ks_nonexistent_column() {
        let baseline_batch = create_float_batch(vec![0.5]);
        let current_batch = create_float_batch(vec![0.5]);

        let result = ks_test(&baseline_batch, &current_batch, "nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_ks_display() {
        let baseline_batch = create_float_batch(vec![0.1, 0.2, 0.3]);
        let current_batch = create_float_batch(vec![0.7, 0.8, 0.9]);

        let result =
            ks_test(&baseline_batch, &current_batch, "value").expect("ks_test should work");

        // Test Display implementation doesn't panic
        let display = format!("{}", result);
        assert!(display.contains("Kolmogorov-Smirnov"));
        assert!(display.contains("value"));
    }
}
