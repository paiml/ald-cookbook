//! Local dataset registry for publishing and versioning.
//!
//! Provides a file-system based registry for managing datasets
//! with versioning support.

use crate::error::{Error, Result};
use crate::format::{self, DatasetType, SaveOptions};
use arrow::array::RecordBatch;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// License types for datasets.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum License {
    /// MIT License.
    MIT,
    /// Apache 2.0 License.
    Apache2,
    /// Creative Commons Attribution.
    CCBY4,
    /// Creative Commons Zero (public domain).
    CC0,
    /// Proprietary/Commercial.
    Proprietary,
    /// Custom license with text.
    Custom(String),
}

impl std::fmt::Display for License {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MIT => write!(f, "MIT"),
            Self::Apache2 => write!(f, "Apache-2.0"),
            Self::CCBY4 => write!(f, "CC-BY-4.0"),
            Self::CC0 => write!(f, "CC0-1.0"),
            Self::Proprietary => write!(f, "Proprietary"),
            Self::Custom(s) => write!(f, "Custom: {s}"),
        }
    }
}

/// Options for publishing a dataset.
#[derive(Debug, Clone)]
pub struct PublishOptions {
    /// Dataset version (semver).
    pub version: String,
    /// Short description.
    pub description: String,
    /// License.
    pub license: License,
    /// Dataset type.
    pub dataset_type: DatasetType,
    /// Additional tags.
    pub tags: Vec<String>,
    /// Author information.
    pub author: Option<String>,
}

impl Default for PublishOptions {
    fn default() -> Self {
        Self {
            version: "0.1.0".to_string(),
            description: String::new(),
            license: License::MIT,
            dataset_type: DatasetType::Tabular,
            tags: Vec::new(),
            author: None,
        }
    }
}

/// Dataset metadata stored in the registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetInfo {
    /// Dataset name.
    pub name: String,
    /// Current version.
    pub version: String,
    /// Description.
    pub description: String,
    /// License identifier.
    pub license: String,
    /// Number of rows.
    pub num_rows: usize,
    /// Number of columns.
    pub num_columns: usize,
    /// File size in bytes.
    pub size_bytes: u64,
    /// Publication timestamp.
    pub published_at: String,
    /// Tags.
    pub tags: Vec<String>,
    /// Author.
    pub author: Option<String>,
    /// All available versions.
    pub versions: Vec<String>,
}

impl std::fmt::Display for DatasetInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Dataset: {}", self.name)?;
        writeln!(f, "  Version: {}", self.version)?;
        writeln!(f, "  Description: {}", self.description)?;
        writeln!(f, "  License: {}", self.license)?;
        writeln!(f, "  Rows: {}", self.num_rows)?;
        writeln!(f, "  Columns: {}", self.num_columns)?;
        writeln!(f, "  Size: {} KB", self.size_bytes / 1024)?;
        if !self.tags.is_empty() {
            writeln!(f, "  Tags: {}", self.tags.join(", "))?;
        }
        if let Some(ref author) = self.author {
            writeln!(f, "  Author: {author}")?;
        }
        if self.versions.len() > 1 {
            writeln!(f, "  Available versions: {}", self.versions.join(", "))?;
        }
        Ok(())
    }
}

/// Local file-system based dataset registry.
pub struct Registry {
    /// Root path of the registry.
    root: PathBuf,
    /// Index of all datasets.
    index: RegistryIndex,
}

/// Registry index tracking all datasets.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct RegistryIndex {
    /// Map of dataset name to info.
    datasets: HashMap<String, DatasetInfo>,
}

impl Registry {
    /// Create or open a registry at the given path.
    ///
    /// # Errors
    ///
    /// Returns `Error::Io` if directory creation fails.
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let root = path.as_ref().to_path_buf();
        fs::create_dir_all(&root)?;

        let index_path = root.join("index.json");
        let index = if index_path.exists() {
            let data = fs::read_to_string(&index_path)?;
            serde_json::from_str(&data).map_err(|e| Error::Deserialization(e.to_string()))?
        } else {
            RegistryIndex::default()
        };

        Ok(Self { root, index })
    }

    /// Get the registry root path.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Publish a dataset to the registry.
    ///
    /// # Errors
    ///
    /// Returns errors if saving fails.
    pub fn publish(
        &mut self,
        name: &str,
        batch: &RecordBatch,
        options: PublishOptions,
    ) -> Result<DatasetInfo> {
        // Create dataset directory
        let dataset_dir = self.root.join(name);
        fs::create_dir_all(&dataset_dir)?;

        // Create version directory
        let version_dir = dataset_dir.join(&options.version);
        fs::create_dir_all(&version_dir)?;

        // Save dataset
        let ald_path = version_dir.join("data.ald");
        let save_opts = SaveOptions::new().with_name(name.to_string());
        format::save(batch, options.dataset_type, &ald_path, save_opts)?;

        let size_bytes = fs::metadata(&ald_path)?.len();

        // Get existing versions or create new list
        let mut versions = self
            .index
            .datasets
            .get(name)
            .map(|d| d.versions.clone())
            .unwrap_or_default();

        if !versions.contains(&options.version) {
            versions.push(options.version.clone());
            versions.sort();
        }

        // Create dataset info
        let info = DatasetInfo {
            name: name.to_string(),
            version: options.version,
            description: options.description,
            license: options.license.to_string(),
            num_rows: batch.num_rows(),
            num_columns: batch.num_columns(),
            size_bytes,
            published_at: chrono::Utc::now().to_rfc3339(),
            tags: options.tags,
            author: options.author,
            versions,
        };

        // Update index
        self.index.datasets.insert(name.to_string(), info.clone());
        self.save_index()?;

        Ok(info)
    }

    /// Pull a dataset from the registry.
    ///
    /// # Arguments
    ///
    /// * `name` - Dataset name
    /// * `version` - Optional version (defaults to latest)
    ///
    /// # Errors
    ///
    /// Returns `Error::DatasetNotFound` if dataset doesn't exist.
    pub fn pull(&self, name: &str, version: Option<&str>) -> Result<RecordBatch> {
        let info = self
            .index
            .datasets
            .get(name)
            .ok_or_else(|| Error::DatasetNotFound(PathBuf::from(name)))?;

        let version = version.unwrap_or(&info.version);
        let ald_path = self.root.join(name).join(version).join("data.ald");

        if !ald_path.exists() {
            return Err(Error::DatasetNotFound(ald_path));
        }

        format::load(&ald_path)
    }

    /// List all datasets in the registry.
    #[must_use]
    pub fn list(&self) -> Vec<&DatasetInfo> {
        self.index.datasets.values().collect()
    }

    /// Get info for a specific dataset.
    #[must_use]
    pub fn get_info(&self, name: &str) -> Option<&DatasetInfo> {
        self.index.datasets.get(name)
    }

    /// Check if a dataset exists.
    #[must_use]
    pub fn exists(&self, name: &str) -> bool {
        self.index.datasets.contains_key(name)
    }

    /// Get all versions of a dataset.
    #[must_use]
    pub fn versions(&self, name: &str) -> Option<&[String]> {
        self.index.datasets.get(name).map(|d| d.versions.as_slice())
    }

    /// Delete a dataset version from the registry.
    ///
    /// # Errors
    ///
    /// Returns errors if deletion fails.
    pub fn delete(&mut self, name: &str, version: Option<&str>) -> Result<()> {
        let _info = self
            .index
            .datasets
            .get(name)
            .ok_or_else(|| Error::DatasetNotFound(PathBuf::from(name)))?;

        if let Some(version) = version {
            // Delete specific version
            let version_dir = self.root.join(name).join(version);
            if version_dir.exists() {
                fs::remove_dir_all(&version_dir)?;
            }

            // Update versions list
            if let Some(ds) = self.index.datasets.get_mut(name) {
                ds.versions.retain(|v| v != version);
                if ds.versions.is_empty() {
                    self.index.datasets.remove(name);
                    // Remove dataset directory
                    let dataset_dir = self.root.join(name);
                    if dataset_dir.exists() {
                        fs::remove_dir_all(&dataset_dir)?;
                    }
                }
            }
        } else {
            // Delete all versions
            let dataset_dir = self.root.join(name);
            if dataset_dir.exists() {
                fs::remove_dir_all(&dataset_dir)?;
            }
            self.index.datasets.remove(name);
        }

        self.save_index()?;
        Ok(())
    }

    /// Save the index to disk.
    fn save_index(&self) -> Result<()> {
        let index_path = self.root.join("index.json");
        let data = serde_json::to_string_pretty(&self.index)
            .map_err(|e| Error::Serialization(e.to_string()))?;
        fs::write(&index_path, data)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::{Float64Array, Int64Array};
    use arrow::datatypes::{DataType, Field, Schema};
    use std::sync::Arc;

    fn create_test_batch() -> RecordBatch {
        let schema = Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("value", DataType::Float64, false),
        ]);

        let id_array = Int64Array::from(vec![1, 2, 3, 4, 5]);
        let value_array = Float64Array::from(vec![1.1, 2.2, 3.3, 4.4, 5.5]);

        RecordBatch::try_new(
            Arc::new(schema),
            vec![Arc::new(id_array), Arc::new(value_array)],
        )
        .unwrap()
    }

    #[test]
    fn test_registry_creation() {
        let temp = tempfile::tempdir().unwrap();
        let registry = Registry::new(temp.path().join("registry")).unwrap();

        assert!(registry.root().exists());
        assert!(registry.list().is_empty());
    }

    #[test]
    fn test_publish_and_pull() {
        let temp = tempfile::tempdir().unwrap();
        let mut registry = Registry::new(temp.path().join("registry")).unwrap();

        let batch = create_test_batch();
        let info = registry
            .publish(
                "test-dataset",
                &batch,
                PublishOptions {
                    version: "1.0.0".to_string(),
                    description: "Test dataset".to_string(),
                    license: License::MIT,
                    ..Default::default()
                },
            )
            .unwrap();

        assert_eq!(info.name, "test-dataset");
        assert_eq!(info.version, "1.0.0");
        assert_eq!(info.num_rows, 5);

        // Pull dataset
        let pulled = registry.pull("test-dataset", None).unwrap();
        assert_eq!(pulled.num_rows(), 5);
    }

    #[test]
    fn test_publish_multiple_versions() {
        let temp = tempfile::tempdir().unwrap();
        let mut registry = Registry::new(temp.path().join("registry")).unwrap();

        let batch = create_test_batch();

        registry
            .publish(
                "test-dataset",
                &batch,
                PublishOptions {
                    version: "1.0.0".to_string(),
                    ..Default::default()
                },
            )
            .unwrap();

        registry
            .publish(
                "test-dataset",
                &batch,
                PublishOptions {
                    version: "1.1.0".to_string(),
                    ..Default::default()
                },
            )
            .unwrap();

        let versions = registry.versions("test-dataset").unwrap();
        assert_eq!(versions.len(), 2);
        assert!(versions.contains(&"1.0.0".to_string()));
        assert!(versions.contains(&"1.1.0".to_string()));
    }

    #[test]
    fn test_pull_specific_version() {
        let temp = tempfile::tempdir().unwrap();
        let mut registry = Registry::new(temp.path().join("registry")).unwrap();

        let batch = create_test_batch();

        registry
            .publish(
                "test-dataset",
                &batch,
                PublishOptions {
                    version: "1.0.0".to_string(),
                    ..Default::default()
                },
            )
            .unwrap();

        let pulled = registry.pull("test-dataset", Some("1.0.0")).unwrap();
        assert_eq!(pulled.num_rows(), 5);
    }

    #[test]
    fn test_pull_nonexistent() {
        let temp = tempfile::tempdir().unwrap();
        let registry = Registry::new(temp.path().join("registry")).unwrap();

        let result = registry.pull("nonexistent", None);
        assert!(matches!(result, Err(Error::DatasetNotFound(_))));
    }

    #[test]
    fn test_list_datasets() {
        let temp = tempfile::tempdir().unwrap();
        let mut registry = Registry::new(temp.path().join("registry")).unwrap();

        let batch = create_test_batch();

        registry
            .publish("dataset-a", &batch, PublishOptions::default())
            .unwrap();
        registry
            .publish("dataset-b", &batch, PublishOptions::default())
            .unwrap();

        let list = registry.list();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_exists() {
        let temp = tempfile::tempdir().unwrap();
        let mut registry = Registry::new(temp.path().join("registry")).unwrap();

        let batch = create_test_batch();
        registry
            .publish("test-dataset", &batch, PublishOptions::default())
            .unwrap();

        assert!(registry.exists("test-dataset"));
        assert!(!registry.exists("nonexistent"));
    }

    #[test]
    fn test_delete_version() {
        let temp = tempfile::tempdir().unwrap();
        let mut registry = Registry::new(temp.path().join("registry")).unwrap();

        let batch = create_test_batch();

        registry
            .publish(
                "test-dataset",
                &batch,
                PublishOptions {
                    version: "1.0.0".to_string(),
                    ..Default::default()
                },
            )
            .unwrap();
        registry
            .publish(
                "test-dataset",
                &batch,
                PublishOptions {
                    version: "1.1.0".to_string(),
                    ..Default::default()
                },
            )
            .unwrap();

        registry.delete("test-dataset", Some("1.0.0")).unwrap();

        let versions = registry.versions("test-dataset").unwrap();
        assert_eq!(versions.len(), 1);
        assert_eq!(versions[0], "1.1.0");
    }

    #[test]
    fn test_delete_all() {
        let temp = tempfile::tempdir().unwrap();
        let mut registry = Registry::new(temp.path().join("registry")).unwrap();

        let batch = create_test_batch();
        registry
            .publish("test-dataset", &batch, PublishOptions::default())
            .unwrap();

        registry.delete("test-dataset", None).unwrap();

        assert!(!registry.exists("test-dataset"));
    }

    #[test]
    fn test_license_display() {
        assert_eq!(License::MIT.to_string(), "MIT");
        assert_eq!(License::Apache2.to_string(), "Apache-2.0");
        assert_eq!(License::CCBY4.to_string(), "CC-BY-4.0");
        assert_eq!(License::CC0.to_string(), "CC0-1.0");
        assert_eq!(
            License::Custom("My License".to_string()).to_string(),
            "Custom: My License"
        );
    }

    #[test]
    fn test_persistence() {
        let temp = tempfile::tempdir().unwrap();
        let registry_path = temp.path().join("registry");

        // Create and publish
        {
            let mut registry = Registry::new(&registry_path).unwrap();
            let batch = create_test_batch();
            registry
                .publish("test-dataset", &batch, PublishOptions::default())
                .unwrap();
        }

        // Reopen and verify
        {
            let registry = Registry::new(&registry_path).unwrap();
            assert!(registry.exists("test-dataset"));
            let info = registry.get_info("test-dataset").unwrap();
            assert_eq!(info.num_rows, 5);
        }
    }
}
