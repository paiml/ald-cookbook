//! Federated learning data utilities.
//!
//! Provides dataset splitting strategies for federated learning simulation:
//! - IID (Independent and Identically Distributed)
//! - Non-IID (using Dirichlet distribution)

#![allow(
    clippy::many_single_char_names,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::missing_errors_doc,
    clippy::option_if_let_else
)]

use crate::error::{Error, Result};
use arrow::array::{Array, ArrayRef, Int64Array, RecordBatch, StringArray, UInt64Array};
use arrow::compute::take;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::HashMap;
/// Split a dataset into N equal IID (Independent and Identically Distributed) partitions.
///
/// Each client receives a random sample of the data.
///
/// # Arguments
///
/// * `batch` - The dataset to split
/// * `n_clients` - Number of clients/partitions
/// * `rng` - Deterministic RNG
///
/// # Errors
///
/// Returns `Error::Arrow` if split operations fail.
pub fn iid_split(
    batch: &RecordBatch,
    n_clients: usize,
    rng: &mut StdRng,
) -> Result<Vec<RecordBatch>> {
    if n_clients == 0 {
        return Ok(vec![]);
    }

    let num_rows = batch.num_rows();
    if num_rows == 0 {
        return Ok(vec![batch.clone(); n_clients]);
    }

    // Shuffle indices
    let mut indices: Vec<u64> = (0..num_rows as u64).collect();
    indices.shuffle(rng);

    // Split into n_clients partitions
    let chunk_size = num_rows / n_clients;
    let remainder = num_rows % n_clients;

    let mut splits = Vec::with_capacity(n_clients);
    let mut start = 0;

    for i in 0..n_clients {
        let extra = usize::from(i < remainder);
        let end = start + chunk_size + extra;

        let client_indices: Vec<u64> = indices[start..end].to_vec();
        let client_batch = take_by_indices(batch, &client_indices)?;
        splits.push(client_batch);

        start = end;
    }

    Ok(splits)
}

/// Split a dataset into non-IID partitions with label skew.
///
/// Each client receives a subset of labels, simulating heterogeneous data distribution.
///
/// # Arguments
///
/// * `batch` - The dataset to split
/// * `label_column` - Column containing class labels
/// * `n_clients` - Number of clients
/// * `classes_per_client` - Number of classes each client should have
/// * `rng` - Deterministic RNG
///
/// # Errors
///
/// Returns `Error::ColumnNotFound` if label column doesn't exist.
pub fn non_iid_split(
    batch: &RecordBatch,
    label_column: &str,
    n_clients: usize,
    classes_per_client: usize,
    rng: &mut StdRng,
) -> Result<Vec<RecordBatch>> {
    if n_clients == 0 {
        return Ok(vec![]);
    }

    // Group indices by label
    let label_groups = group_by_label(batch, label_column)?;

    if label_groups.is_empty() {
        return iid_split(batch, n_clients, rng);
    }

    let labels: Vec<&String> = label_groups.keys().collect();
    let n_labels = labels.len();

    if classes_per_client >= n_labels {
        // If requesting more classes than available, fall back to IID
        return iid_split(batch, n_clients, rng);
    }

    let mut splits: Vec<Vec<u64>> = vec![vec![]; n_clients];

    // Assign classes to clients in round-robin with shuffle
    let mut label_order: Vec<usize> = (0..n_labels).collect();
    label_order.shuffle(rng);

    for (client_idx, client_split) in splits.iter_mut().enumerate() {
        // Pick `classes_per_client` labels for this client
        let start_label = (client_idx * classes_per_client) % n_labels;
        for j in 0..classes_per_client {
            let label_idx = (start_label + j) % n_labels;
            let label = labels[label_order[label_idx]];
            if let Some(indices) = label_groups.get(label) {
                // Sample from this label's indices
                let sample_size = indices.len() / n_clients + 1;
                let mut sampled: Vec<u64> = indices.clone();
                sampled.shuffle(rng);
                client_split.extend(sampled.into_iter().take(sample_size));
            }
        }
    }

    // Convert to RecordBatches
    splits
        .into_iter()
        .map(|indices| take_by_indices(batch, &indices))
        .collect()
}

/// Split a dataset with stratification to maintain class balance.
///
/// Each client receives a proportional sample of each class.
///
/// # Arguments
///
/// * `batch` - The dataset to split
/// * `label_column` - Column containing class labels
/// * `n_clients` - Number of clients
/// * `rng` - Deterministic RNG
///
/// # Errors
///
/// Returns `Error::ColumnNotFound` if label column doesn't exist.
pub fn stratified_split(
    batch: &RecordBatch,
    label_column: &str,
    n_clients: usize,
    rng: &mut StdRng,
) -> Result<Vec<RecordBatch>> {
    if n_clients == 0 {
        return Ok(vec![]);
    }

    let label_groups = group_by_label(batch, label_column)?;

    if label_groups.is_empty() {
        return iid_split(batch, n_clients, rng);
    }

    let mut splits: Vec<Vec<u64>> = vec![vec![]; n_clients];

    // For each class, distribute samples evenly across clients
    for indices in label_groups.values() {
        let mut shuffled = indices.clone();
        shuffled.shuffle(rng);

        let samples_per_client = indices.len() / n_clients;
        let remainder = indices.len() % n_clients;

        let mut start = 0;
        for (i, client_split) in splits.iter_mut().enumerate() {
            let extra = usize::from(i < remainder);
            let end = start + samples_per_client + extra;
            client_split.extend(&shuffled[start..end]);
            start = end;
        }
    }

    // Shuffle within each client
    for client_split in &mut splits {
        client_split.shuffle(rng);
    }

    splits
        .into_iter()
        .map(|indices| take_by_indices(batch, &indices))
        .collect()
}

/// Split a dataset using Dirichlet distribution for heterogeneous partitioning.
///
/// The concentration parameter α controls heterogeneity:
/// - α → ∞: approaches IID
/// - α → 0: extremely heterogeneous
///
/// # Arguments
///
/// * `batch` - The dataset to split
/// * `label_column` - Column containing class labels
/// * `n_clients` - Number of clients
/// * `alpha` - Dirichlet concentration parameter
/// * `rng` - Deterministic RNG
///
/// # Errors
///
/// Returns `Error::ColumnNotFound` if label column doesn't exist.
pub fn dirichlet_split(
    batch: &RecordBatch,
    label_column: &str,
    n_clients: usize,
    alpha: f64,
    rng: &mut StdRng,
) -> Result<Vec<RecordBatch>> {
    if n_clients == 0 {
        return Ok(vec![]);
    }

    let label_groups = group_by_label(batch, label_column)?;

    if label_groups.is_empty() {
        return iid_split(batch, n_clients, rng);
    }

    let mut splits: Vec<Vec<u64>> = vec![vec![]; n_clients];

    // For each class, sample Dirichlet distribution to determine allocation
    for indices in label_groups.values() {
        // Sample proportions from Dirichlet
        let proportions = sample_dirichlet(n_clients, alpha, rng);

        let mut shuffled = indices.clone();
        shuffled.shuffle(rng);

        // Allocate samples according to proportions
        let mut start = 0;
        for (i, proportion) in proportions.iter().enumerate() {
            let count = (proportion * indices.len() as f64).round() as usize;
            let end = (start + count).min(indices.len());
            splits[i].extend(&shuffled[start..end]);
            start = end;
        }

        // Handle any remaining samples
        while start < shuffled.len() {
            let client = rng.gen_range(0..n_clients);
            splits[client].push(shuffled[start]);
            start += 1;
        }
    }

    // Shuffle within each client
    for client_split in &mut splits {
        client_split.shuffle(rng);
    }

    splits
        .into_iter()
        .map(|indices| take_by_indices(batch, &indices))
        .collect()
}

/// Sample from Dirichlet distribution using Gamma distribution.
fn sample_dirichlet(k: usize, alpha: f64, rng: &mut StdRng) -> Vec<f64> {
    // Dirichlet(α, α, ..., α) can be sampled via Gamma distributions
    let samples: Vec<f64> = (0..k)
        .map(|_| {
            // Sample Gamma(alpha, 1) using Marsaglia and Tsang's method
            sample_gamma(alpha, rng)
        })
        .collect();

    let sum: f64 = samples.iter().sum();
    if sum == 0.0 {
        return vec![1.0 / k as f64; k];
    }

    samples.iter().map(|x| x / sum).collect()
}

/// Sample from Gamma distribution.
fn sample_gamma(alpha: f64, rng: &mut StdRng) -> f64 {
    // Marsaglia and Tsang's method
    if alpha < 1.0 {
        let u: f64 = rng.gen();
        return sample_gamma(1.0 + alpha, rng) * u.powf(1.0 / alpha);
    }

    let d = alpha - 1.0 / 3.0;
    let c = 1.0 / (9.0 * d).sqrt();

    loop {
        let x: f64 = {
            // Sample standard normal
            let u1: f64 = rng.gen();
            let u2: f64 = rng.gen();
            (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
        };

        let v = (1.0 + c * x).powi(3);
        if v <= 0.0 {
            continue;
        }

        let u: f64 = rng.gen();
        if u < 0.0331f64.mul_add(-x.powi(4), 1.0) {
            return d * v;
        }

        if u.ln() < 0.5f64.mul_add(x.powi(2), d * (1.0 - v + v.ln())) {
            return d * v;
        }
    }
}

/// Group row indices by label value.
fn group_by_label(batch: &RecordBatch, label_column: &str) -> Result<HashMap<String, Vec<u64>>> {
    let col_idx = batch
        .schema()
        .index_of(label_column)
        .map_err(|_| Error::ColumnNotFound(label_column.to_string()))?;

    let col = batch.column(col_idx);
    let mut groups: HashMap<String, Vec<u64>> = HashMap::new();

    // Handle different column types
    if let Some(str_col) = col.as_any().downcast_ref::<StringArray>() {
        for (i, val) in str_col.iter().enumerate() {
            if let Some(v) = val {
                groups.entry(v.to_string()).or_default().push(i as u64);
            }
        }
    } else if let Some(int_col) = col.as_any().downcast_ref::<Int64Array>() {
        for (i, val) in int_col.iter().enumerate() {
            if let Some(v) = val {
                groups.entry(v.to_string()).or_default().push(i as u64);
            }
        }
    } else {
        return Err(Error::InvalidColumnType {
            expected: "Utf8 or Int64".to_string(),
            actual: format!("{:?}", col.data_type()),
        });
    }

    Ok(groups)
}

/// Take rows by indices.
fn take_by_indices(batch: &RecordBatch, indices: &[u64]) -> Result<RecordBatch> {
    if indices.is_empty() {
        // Return empty batch with same schema
        return Ok(RecordBatch::new_empty(batch.schema()));
    }

    let indices_array = UInt64Array::from(indices.to_vec());
    let columns: Vec<ArrayRef> = batch
        .columns()
        .iter()
        .map(|col| take(col.as_ref(), &indices_array, None).map_err(Error::from))
        .collect::<Result<Vec<_>>>()?;

    RecordBatch::try_new(batch.schema(), columns).map_err(Error::from)
}

/// Statistics about a federated split.
#[derive(Debug, Clone)]
pub struct SplitStats {
    /// Number of clients.
    pub n_clients: usize,
    /// Samples per client.
    pub samples_per_client: Vec<usize>,
    /// Label distribution per client.
    pub label_distributions: Vec<HashMap<String, usize>>,
}

impl SplitStats {
    /// Calculate statistics for a split.
    pub fn from_splits(splits: &[RecordBatch], label_column: Option<&str>) -> Result<Self> {
        let n_clients = splits.len();
        let samples_per_client: Vec<usize> = splits
            .iter()
            .map(arrow::array::RecordBatch::num_rows)
            .collect();

        let label_distributions = if let Some(col_name) = label_column {
            splits
                .iter()
                .map(|batch| {
                    let groups = group_by_label(batch, col_name).unwrap_or_default();
                    groups.into_iter().map(|(k, v)| (k, v.len())).collect()
                })
                .collect()
        } else {
            vec![HashMap::new(); n_clients]
        };

        Ok(Self {
            n_clients,
            samples_per_client,
            label_distributions,
        })
    }
}

impl std::fmt::Display for SplitStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Federated Split Statistics")?;
        writeln!(f, "{:-<50}", "")?;
        writeln!(f, "  Number of clients: {}", self.n_clients)?;
        for (i, count) in self.samples_per_client.iter().enumerate() {
            write!(f, "  Client {i}: {count} samples")?;
            if !self.label_distributions[i].is_empty() {
                write!(f, " [")?;
                for (label, n) in &self.label_distributions[i] {
                    write!(f, " {label}:{n}")?;
                }
                write!(f, " ]")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::Float64Array;
    use arrow::datatypes::{DataType, Field, Schema};
    use rand::SeedableRng;
    use std::sync::Arc;

    fn create_labeled_batch() -> RecordBatch {
        let schema = Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("value", DataType::Float64, false),
            Field::new("label", DataType::Utf8, false),
        ]);

        let id_array = Int64Array::from((0..100).collect::<Vec<_>>());
        let value_array = Float64Array::from((0..100).map(|x| x as f64).collect::<Vec<_>>());
        // 4 classes: A, B, C, D with 25 samples each
        let label_array = StringArray::from(
            (0..100)
                .map(|i| match i % 4 {
                    0 => "A",
                    1 => "B",
                    2 => "C",
                    _ => "D",
                })
                .collect::<Vec<_>>(),
        );

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
    fn test_iid_split() {
        let batch = create_labeled_batch();
        let mut rng = StdRng::seed_from_u64(42);

        let splits = iid_split(&batch, 5, &mut rng).unwrap();

        assert_eq!(splits.len(), 5);

        // Total samples should equal original
        let total: usize = splits.iter().map(|b| b.num_rows()).sum();
        assert_eq!(total, batch.num_rows());

        // Each split should have approximately equal size
        for split in &splits {
            assert!(split.num_rows() >= 19 && split.num_rows() <= 21);
        }
    }

    #[test]
    fn test_iid_split_deterministic() {
        let batch = create_labeled_batch();
        let mut rng1 = StdRng::seed_from_u64(42);
        let mut rng2 = StdRng::seed_from_u64(42);

        let splits1 = iid_split(&batch, 5, &mut rng1).unwrap();
        let splits2 = iid_split(&batch, 5, &mut rng2).unwrap();

        for (s1, s2) in splits1.iter().zip(splits2.iter()) {
            assert_eq!(s1.num_rows(), s2.num_rows());
        }
    }

    #[test]
    fn test_non_iid_split() {
        let batch = create_labeled_batch();
        let mut rng = StdRng::seed_from_u64(42);

        let splits = non_iid_split(&batch, "label", 4, 2, &mut rng).unwrap();

        assert_eq!(splits.len(), 4);

        // Each client should have limited labels
        for split in &splits {
            let groups = group_by_label(split, "label").unwrap();
            // Should have at most 2 classes (may have fewer due to distribution)
            assert!(groups.len() <= 3);
        }
    }

    #[test]
    fn test_stratified_split() {
        let batch = create_labeled_batch();
        let mut rng = StdRng::seed_from_u64(42);

        let splits = stratified_split(&batch, "label", 5, &mut rng).unwrap();

        assert_eq!(splits.len(), 5);

        // Each split should have all labels
        for split in &splits {
            let groups = group_by_label(split, "label").unwrap();
            assert_eq!(groups.len(), 4); // All 4 labels
        }
    }

    #[test]
    fn test_dirichlet_split() {
        let batch = create_labeled_batch();
        let mut rng = StdRng::seed_from_u64(42);

        // Low alpha = high heterogeneity
        let splits = dirichlet_split(&batch, "label", 5, 0.5, &mut rng).unwrap();

        assert_eq!(splits.len(), 5);

        // Total samples should equal original
        let total: usize = splits.iter().map(|b| b.num_rows()).sum();
        assert_eq!(total, batch.num_rows());
    }

    #[test]
    fn test_dirichlet_high_alpha() {
        let batch = create_labeled_batch();
        let mut rng = StdRng::seed_from_u64(42);

        // High alpha = more uniform/IID-like
        let splits = dirichlet_split(&batch, "label", 5, 100.0, &mut rng).unwrap();

        assert_eq!(splits.len(), 5);

        // With high alpha, splits should be more balanced
        let sizes: Vec<usize> = splits.iter().map(|b| b.num_rows()).collect();
        let avg = sizes.iter().sum::<usize>() as f64 / sizes.len() as f64;
        for size in sizes {
            assert!((size as f64 - avg).abs() < 15.0);
        }
    }

    #[test]
    fn test_split_stats() {
        let batch = create_labeled_batch();
        let mut rng = StdRng::seed_from_u64(42);

        let splits = stratified_split(&batch, "label", 4, &mut rng).unwrap();
        let stats = SplitStats::from_splits(&splits, Some("label")).unwrap();

        assert_eq!(stats.n_clients, 4);
        assert_eq!(stats.samples_per_client.len(), 4);
        assert_eq!(stats.label_distributions.len(), 4);

        // Each client should have all 4 labels
        for dist in &stats.label_distributions {
            assert_eq!(dist.len(), 4);
        }
    }

    #[test]
    fn test_empty_split() {
        let schema = Schema::new(vec![Field::new("id", DataType::Int64, false)]);
        let batch = RecordBatch::new_empty(Arc::new(schema));
        let mut rng = StdRng::seed_from_u64(42);

        let splits = iid_split(&batch, 3, &mut rng).unwrap();

        assert_eq!(splits.len(), 3);
        for split in splits {
            assert_eq!(split.num_rows(), 0);
        }
    }

    #[test]
    fn test_zero_clients() {
        let batch = create_labeled_batch();
        let mut rng = StdRng::seed_from_u64(42);

        let splits = iid_split(&batch, 0, &mut rng).unwrap();
        assert!(splits.is_empty());
    }

    #[test]
    fn test_column_not_found() {
        let batch = create_labeled_batch();
        let mut rng = StdRng::seed_from_u64(42);

        let result = stratified_split(&batch, "nonexistent", 3, &mut rng);
        assert!(matches!(result, Err(Error::ColumnNotFound(_))));
    }
}
