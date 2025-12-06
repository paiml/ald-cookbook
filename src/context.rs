//! Recipe context for isolated, idempotent execution.
//!
//! Provides standardized isolation primitives following IIUR principles:
//! - **Isolated**: Temporary directory with automatic cleanup
//! - **Idempotent**: Deterministic RNG seeded by recipe name
//! - **Reproducible**: Consistent behavior across runs

use crate::error::{Error, Result};
use rand::{rngs::StdRng, SeedableRng};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use tempfile::TempDir;

/// Metadata about a recipe for reporting.
#[derive(Debug, Clone)]
pub struct RecipeMetadata {
    /// Recipe name (used for seeding).
    pub name: String,
    /// Recipe category (e.g., "`dataset_creation`").
    pub category: Option<String>,
    /// Short description.
    pub description: Option<String>,
    /// Timestamp when context was created.
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl RecipeMetadata {
    /// Create metadata from a recipe name.
    #[must_use]
    pub fn from_name(name: &str) -> Self {
        Self {
            name: name.to_string(),
            category: None,
            description: None,
            created_at: chrono::Utc::now(),
        }
    }

    /// Set the category.
    #[must_use]
    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    /// Set the description.
    #[must_use]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// Recipe execution context providing isolation and determinism.
///
/// # Example
///
/// ```
/// use ald_cookbook::context::RecipeContext;
///
/// let ctx = RecipeContext::new("my_recipe").unwrap();
/// let data_path = ctx.path("data.ald");
/// // Work with files in the isolated temp directory
/// // Cleanup happens automatically when ctx is dropped
/// ```
pub struct RecipeContext {
    /// Isolated temporary directory (auto-cleanup on drop).
    temp_dir: TempDir,
    /// Deterministic RNG seeded by recipe name.
    pub rng: StdRng,
    /// Recipe metadata for reporting.
    pub metadata: RecipeMetadata,
    /// The seed used for RNG.
    seed: u64,
}

impl RecipeContext {
    /// Create a new recipe context with isolated temp directory and deterministic RNG.
    ///
    /// # Errors
    ///
    /// Returns `Error::ContextInit` if the temporary directory cannot be created.
    pub fn new(name: &str) -> Result<Self> {
        let seed = hash_name_to_seed(name);
        let temp_dir =
            TempDir::new().map_err(|e| Error::ContextInit(format!("temp dir creation: {e}")))?;

        Ok(Self {
            temp_dir,
            rng: StdRng::seed_from_u64(seed),
            metadata: RecipeMetadata::from_name(name),
            seed,
        })
    }

    /// Create a new recipe context with custom metadata.
    ///
    /// # Errors
    ///
    /// Returns `Error::ContextInit` if the temporary directory cannot be created.
    pub fn with_metadata(name: &str, metadata: RecipeMetadata) -> Result<Self> {
        let seed = hash_name_to_seed(name);
        let temp_dir =
            TempDir::new().map_err(|e| Error::ContextInit(format!("temp dir creation: {e}")))?;

        Ok(Self {
            temp_dir,
            rng: StdRng::seed_from_u64(seed),
            metadata,
            seed,
        })
    }

    /// Get a path within the isolated temp directory.
    #[must_use]
    pub fn path(&self, filename: &str) -> PathBuf {
        self.temp_dir.path().join(filename)
    }

    /// Get the temp directory path.
    #[must_use]
    pub fn temp_path(&self) -> &std::path::Path {
        self.temp_dir.path()
    }

    /// Get the RNG seed for reproducibility verification.
    #[must_use]
    pub const fn seed(&self) -> u64 {
        self.seed
    }

    /// Reset the RNG to its initial state (for idempotency testing).
    pub fn reset_rng(&mut self) {
        self.rng = StdRng::seed_from_u64(self.seed);
    }

    /// Create a subdirectory within the temp directory.
    ///
    /// # Errors
    ///
    /// Returns `Error::Io` if the directory cannot be created.
    pub fn create_subdir(&self, name: &str) -> Result<PathBuf> {
        let path = self.path(name);
        std::fs::create_dir_all(&path)?;
        Ok(path)
    }

    /// Report results to stdout in a consistent format.
    ///
    /// # Errors
    ///
    /// Returns `Error::Io` if writing to stdout fails.
    pub fn report<T: std::fmt::Display>(&self, result: &T) -> Result<()> {
        println!("Recipe: {}", self.metadata.name);
        if let Some(ref cat) = self.metadata.category {
            println!("Category: {cat}");
        }
        println!("Seed: {}", self.seed);
        println!("{:-<50}", "");
        println!("{result}");
        println!("{:-<50}", "");
        Ok(())
    }
}

/// Hash a recipe name to a deterministic seed.
fn hash_name_to_seed(name: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    name.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn test_context_creation() {
        let ctx = RecipeContext::new("test_recipe").unwrap();
        assert_eq!(ctx.metadata.name, "test_recipe");
        assert!(ctx.temp_path().exists());
    }

    #[test]
    fn test_context_path() {
        let ctx = RecipeContext::new("test_recipe").unwrap();
        let path = ctx.path("data.ald");
        assert!(path.ends_with("data.ald"));
        assert!(path.starts_with(ctx.temp_path()));
    }

    #[test]
    fn test_deterministic_seed() {
        let seed1 = hash_name_to_seed("my_recipe");
        let seed2 = hash_name_to_seed("my_recipe");
        let seed3 = hash_name_to_seed("other_recipe");

        assert_eq!(seed1, seed2);
        assert_ne!(seed1, seed3);
    }

    #[test]
    fn test_deterministic_rng() {
        let mut ctx1 = RecipeContext::new("test_recipe").unwrap();
        let mut ctx2 = RecipeContext::new("test_recipe").unwrap();

        let values1: Vec<u64> = (0..10).map(|_| ctx1.rng.gen()).collect();
        let values2: Vec<u64> = (0..10).map(|_| ctx2.rng.gen()).collect();

        assert_eq!(values1, values2);
    }

    #[test]
    fn test_rng_reset() {
        let mut ctx = RecipeContext::new("test_recipe").unwrap();

        let values1: Vec<u64> = (0..10).map(|_| ctx.rng.gen()).collect();
        ctx.reset_rng();
        let values2: Vec<u64> = (0..10).map(|_| ctx.rng.gen()).collect();

        assert_eq!(values1, values2);
    }

    #[test]
    fn test_create_subdir() {
        let ctx = RecipeContext::new("test_recipe").unwrap();
        let subdir = ctx.create_subdir("nested/path").unwrap();
        assert!(subdir.exists());
        assert!(subdir.is_dir());
    }

    #[test]
    fn test_temp_cleanup() {
        let path: PathBuf;
        {
            let ctx = RecipeContext::new("cleanup_test").unwrap();
            path = ctx.temp_path().to_path_buf();
            assert!(path.exists());
        }
        // After ctx is dropped, temp dir should be cleaned up
        assert!(!path.exists());
    }

    #[test]
    fn test_metadata_builder() {
        let metadata = RecipeMetadata::from_name("my_recipe")
            .with_category("dataset_creation")
            .with_description("Test recipe");

        assert_eq!(metadata.name, "my_recipe");
        assert_eq!(metadata.category, Some("dataset_creation".to_string()));
        assert_eq!(metadata.description, Some("Test recipe".to_string()));
    }

    #[test]
    fn test_context_with_metadata() {
        let metadata = RecipeMetadata::from_name("custom")
            .with_category("testing")
            .with_description("Custom context test");

        let ctx = RecipeContext::with_metadata("custom", metadata).unwrap();
        assert_eq!(ctx.metadata.category, Some("testing".to_string()));
    }

    #[test]
    fn test_seed_getter() {
        let ctx = RecipeContext::new("seed_test").unwrap();
        let expected = hash_name_to_seed("seed_test");
        assert_eq!(ctx.seed(), expected);
    }
}
