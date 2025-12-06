//! Prelude module for convenient imports.
//!
//! ```
//! use ald_cookbook::prelude::*;
//! ```

pub use crate::context::{RecipeContext, RecipeMetadata};
pub use crate::error::{Error, Result};
pub use crate::format::{
    load, load_from_bytes, load_metadata, save, DatasetType, FormatFlags, Header, Metadata,
    SaveOptions,
};

// Re-export commonly used types from dependencies
pub use arrow::array::RecordBatch;
pub use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
pub use rand::rngs::StdRng;
pub use rand::{Rng, SeedableRng};
pub use std::sync::Arc;
