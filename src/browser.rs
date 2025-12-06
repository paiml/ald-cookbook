//! Browser/WASM utilities for dataset viewing and manipulation.
//!
//! Provides WASM-compatible APIs for use in web browsers.
//!
//! Requires the `browser` feature.

use crate::error::Result;
use crate::format;
use arrow::array::RecordBatch;
use wasm_bindgen::prelude::*;

/// Dataset viewer for browser use.
#[wasm_bindgen]
pub struct DatasetViewer {
    batch: RecordBatch,
}

#[wasm_bindgen]
impl DatasetViewer {
    /// Create a new viewer from ALD bytes.
    ///
    /// # Errors
    ///
    /// Returns error if the data is invalid.
    #[wasm_bindgen(constructor)]
    pub fn from_bytes(bytes: &[u8]) -> std::result::Result<DatasetViewer, JsValue> {
        let batch =
            format::load_from_bytes(bytes).map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(Self { batch })
    }

    /// Get the number of rows.
    #[wasm_bindgen]
    pub fn num_rows(&self) -> usize {
        self.batch.num_rows()
    }

    /// Get the number of columns.
    #[wasm_bindgen]
    pub fn num_columns(&self) -> usize {
        self.batch.num_columns()
    }

    /// Get the schema as JSON.
    #[wasm_bindgen]
    pub fn schema_json(&self) -> String {
        let schema = self.batch.schema();
        let fields: Vec<FieldInfo> = schema
            .fields()
            .iter()
            .map(|f| FieldInfo {
                name: f.name().clone(),
                data_type: format!("{:?}", f.data_type()),
                nullable: f.is_nullable(),
            })
            .collect();

        serde_json::to_string(&fields).unwrap_or_else(|_| "[]".to_string())
    }

    /// Get column names as JSON array.
    #[wasm_bindgen]
    pub fn column_names(&self) -> String {
        let schema = self.batch.schema();
        let names: Vec<&str> = schema.fields().iter().map(|f| f.name().as_str()).collect();

        serde_json::to_string(&names).unwrap_or_else(|_| "[]".to_string())
    }

    /// Get a preview of the data (first N rows) as JSON.
    #[wasm_bindgen]
    pub fn preview(&self, max_rows: usize) -> String {
        let rows_to_show = max_rows.min(self.batch.num_rows());
        let mut rows: Vec<serde_json::Value> = Vec::with_capacity(rows_to_show);

        for row_idx in 0..rows_to_show {
            let mut row_obj = serde_json::Map::new();
            for col_idx in 0..self.batch.num_columns() {
                let schema = self.batch.schema();
                let col_name = schema.field(col_idx).name();
                let value = column_value_to_json(&self.batch, col_idx, row_idx);
                row_obj.insert(col_name.clone(), value);
            }
            rows.push(serde_json::Value::Object(row_obj));
        }

        serde_json::to_string(&rows).unwrap_or_else(|_| "[]".to_string())
    }

    /// Get summary statistics as JSON.
    #[wasm_bindgen]
    pub fn summary(&self) -> String {
        let summary = DatasetSummary {
            num_rows: self.batch.num_rows(),
            num_columns: self.batch.num_columns(),
            memory_usage_bytes: estimate_memory_usage(&self.batch),
        };

        serde_json::to_string(&summary).unwrap_or_else(|_| "{}".to_string())
    }
}

/// Field information for JSON serialization.
#[derive(serde::Serialize)]
struct FieldInfo {
    name: String,
    data_type: String,
    nullable: bool,
}

/// Dataset summary for JSON serialization.
#[derive(serde::Serialize)]
struct DatasetSummary {
    num_rows: usize,
    num_columns: usize,
    memory_usage_bytes: usize,
}

/// Convert a column value to JSON.
fn column_value_to_json(batch: &RecordBatch, col_idx: usize, row_idx: usize) -> serde_json::Value {
    use arrow::array::*;
    use arrow::datatypes::DataType;

    let col = batch.column(col_idx);

    if col.is_null(row_idx) {
        return serde_json::Value::Null;
    }

    match col.data_type() {
        DataType::Int8 => {
            let arr = col.as_any().downcast_ref::<Int8Array>().unwrap();
            serde_json::json!(arr.value(row_idx))
        }
        DataType::Int16 => {
            let arr = col.as_any().downcast_ref::<Int16Array>().unwrap();
            serde_json::json!(arr.value(row_idx))
        }
        DataType::Int32 => {
            let arr = col.as_any().downcast_ref::<Int32Array>().unwrap();
            serde_json::json!(arr.value(row_idx))
        }
        DataType::Int64 => {
            let arr = col.as_any().downcast_ref::<Int64Array>().unwrap();
            serde_json::json!(arr.value(row_idx))
        }
        DataType::UInt8 => {
            let arr = col.as_any().downcast_ref::<UInt8Array>().unwrap();
            serde_json::json!(arr.value(row_idx))
        }
        DataType::UInt16 => {
            let arr = col.as_any().downcast_ref::<UInt16Array>().unwrap();
            serde_json::json!(arr.value(row_idx))
        }
        DataType::UInt32 => {
            let arr = col.as_any().downcast_ref::<UInt32Array>().unwrap();
            serde_json::json!(arr.value(row_idx))
        }
        DataType::UInt64 => {
            let arr = col.as_any().downcast_ref::<UInt64Array>().unwrap();
            serde_json::json!(arr.value(row_idx))
        }
        DataType::Float32 => {
            let arr = col.as_any().downcast_ref::<Float32Array>().unwrap();
            serde_json::json!(arr.value(row_idx))
        }
        DataType::Float64 => {
            let arr = col.as_any().downcast_ref::<Float64Array>().unwrap();
            serde_json::json!(arr.value(row_idx))
        }
        DataType::Utf8 => {
            let arr = col.as_any().downcast_ref::<StringArray>().unwrap();
            serde_json::json!(arr.value(row_idx))
        }
        DataType::Boolean => {
            let arr = col.as_any().downcast_ref::<BooleanArray>().unwrap();
            serde_json::json!(arr.value(row_idx))
        }
        _ => serde_json::json!(format!("<{:?}>", col.data_type())),
    }
}

/// Estimate memory usage of a batch.
fn estimate_memory_usage(batch: &RecordBatch) -> usize {
    batch
        .columns()
        .iter()
        .map(|col| col.get_array_memory_size())
        .sum()
}

/// Utility functions exposed to JavaScript.
#[wasm_bindgen]
pub fn ald_version() -> String {
    format!("{}.{}", format::VERSION_MAJOR, format::VERSION_MINOR)
}

/// Check if bytes are valid ALD format.
#[wasm_bindgen]
pub fn is_valid_ald(bytes: &[u8]) -> bool {
    if bytes.len() < 4 {
        return false;
    }
    let magic = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    magic == format::ALD_MAGIC
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

        let id_array = Int64Array::from(vec![1, 2, 3]);
        let value_array = Float64Array::from(vec![1.1, 2.2, 3.3]);

        RecordBatch::try_new(
            Arc::new(schema),
            vec![Arc::new(id_array), Arc::new(value_array)],
        )
        .unwrap()
    }

    #[test]
    fn test_estimate_memory() {
        let batch = create_test_batch();
        let size = estimate_memory_usage(&batch);
        assert!(size > 0);
    }

    #[test]
    fn test_column_value_to_json() {
        let batch = create_test_batch();

        let val = column_value_to_json(&batch, 0, 0);
        assert_eq!(val, serde_json::json!(1));

        let val = column_value_to_json(&batch, 1, 1);
        assert_eq!(val, serde_json::json!(2.2));
    }

    #[test]
    fn test_is_valid_ald() {
        // Valid magic
        let mut bytes = vec![0x41, 0x44, 0x4C, 0x46]; // "ALDF" little-endian
        assert!(is_valid_ald(&bytes));

        // Invalid magic
        bytes[0] = 0x00;
        assert!(!is_valid_ald(&bytes));

        // Too short
        assert!(!is_valid_ald(&[0x41, 0x44]));
    }

    #[test]
    fn test_ald_version() {
        let version = ald_version();
        assert!(version.contains('.'));
    }
}
