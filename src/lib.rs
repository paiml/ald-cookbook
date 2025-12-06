//! # ALD Cookbook
//!
//! Cookbook for sharing `.ald` files - Isolated, Idempotent, Useful, and Reproducible dataset recipes.
//!
//! ## IIUR Principles
//!
//! Each recipe in this cookbook follows the IIUR principles:
//!
//! - **Isolated**: No shared mutable state, self-contained dependencies, temp directory isolation
//! - **Idempotent**: Running twice produces identical output, deterministic seeds
//! - **Useful**: Solves real problems, executable demonstrations, copy-paste ready
//! - **Reproducible**: Pinned dependencies, cross-platform, CI-verified
//!
//! ## Quick Start
//!
//! ```
//! use ald_cookbook::prelude::*;
//! use arrow::array::{Int64Array, Float64Array};
//!
//! fn main() -> Result<()> {
//!     // Create isolated context
//!     let ctx = RecipeContext::new("quickstart")?;
//!
//!     // Create a simple dataset
//!     let schema = Schema::new(vec![
//!         Field::new("id", DataType::Int64, false),
//!         Field::new("value", DataType::Float64, false),
//!     ]);
//!
//!     let batch = RecordBatch::try_new(
//!         Arc::new(schema),
//!         vec![
//!             Arc::new(Int64Array::from(vec![1, 2, 3])),
//!             Arc::new(Float64Array::from(vec![1.0, 2.0, 3.0])),
//!         ],
//!     )?;
//!
//!     // Save to ALD format
//!     let path = ctx.path("data.ald");
//!     save(&batch, DatasetType::Tabular, &path, SaveOptions::new())?;
//!
//!     // Load it back
//!     let loaded = load(&path)?;
//!     assert_eq!(batch.num_rows(), loaded.num_rows());
//!
//!     Ok(())
//! }
//! ```
//!
//! ## ALD Format
//!
//! The Alimentar Dataset Format (`.ald`) provides secure, verifiable dataset distribution:
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │ Header (32 bytes, fixed)                │
//! │   Magic: "ALDF" (0x414C4446)            │
//! │   Version: 1.2                          │
//! │   Flags: encryption, signing, streaming │
//! ├─────────────────────────────────────────┤
//! │ Metadata (variable, MessagePack)        │
//! ├─────────────────────────────────────────┤
//! │ Schema (variable, Arrow IPC)            │
//! ├─────────────────────────────────────────┤
//! │ Payload (variable, Arrow IPC + zstd)    │
//! ├─────────────────────────────────────────┤
//! │ Checksum (4 bytes, CRC32)               │
//! └─────────────────────────────────────────┘
//! ```

#![forbid(unsafe_code)]
#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::module_name_repetitions)]

pub mod context;
pub mod convert;
pub mod drift;
pub mod error;
pub mod federated;
pub mod format;
pub mod prelude;
pub mod quality;
pub mod registry;
pub mod transforms;

// Feature-gated modules
#[cfg(feature = "encryption")]
pub mod encryption;

#[cfg(feature = "signing")]
pub mod signing;

#[cfg(feature = "browser")]
pub mod browser;

// Re-export main types at crate root for convenience
pub use crate::context::RecipeContext;
pub use crate::error::{Error, Result};
pub use crate::format::{load, save, DatasetType, Metadata, SaveOptions};
