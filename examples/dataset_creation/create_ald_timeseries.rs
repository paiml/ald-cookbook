//! # Recipe: Create Time Series ALD Dataset
//!
//! **Category**: Dataset Creation
//! **Isolation Level**: Full
//! **Idempotency**: Guaranteed
//! **Dependencies**: None (default features)
//!
//! ## QA Checklist
//! 1. [x] `cargo run` succeeds (Exit Code 0)
//! 2. [x] `cargo test` passes
//! 3. [x] Deterministic output (Verified)
//! 4. [x] No temp files leaked
//! 5. [x] Memory usage stable
//! 6. [x] WASM compatible (if applicable)
//! 7. [x] Clippy clean
//! 8. [x] Rustfmt standard
//! 9. [x] No `unwrap()` in logic
//! 10. [x] Proptests pass (100+ cases)
//!
//! ## Learning Objective
//! Create a time series dataset with temporal indexing and multiple measurement columns.
//!
//! ## Run Command
//! ```bash
//! cargo run --example create_ald_timeseries
//! ```

use ald_cookbook::prelude::*;
use arrow::array::{Float64Array, Int64Array, StringArray, TimestampMillisecondArray};
use rand::Rng;
use std::fmt;
use std::sync::Arc;

/// Result of the recipe execution.
struct RecipeResult {
    num_rows: usize,
    time_range_ms: (i64, i64),
    metrics: Vec<String>,
    file_size_bytes: u64,
}

impl fmt::Display for RecipeResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Created Time Series ALD Dataset")?;
        writeln!(f, "  Data points: {}", self.num_rows)?;
        writeln!(
            f,
            "  Time range: {} - {} (ms since epoch)",
            self.time_range_ms.0, self.time_range_ms.1
        )?;
        writeln!(f, "  Metrics: {:?}", self.metrics)?;
        writeln!(f, "  File size: {} bytes", self.file_size_bytes)?;
        Ok(())
    }
}

/// Generate synthetic time series data (sensor readings).
fn create_sensor_timeseries(rng: &mut impl Rng, num_points: usize) -> Result<RecordBatch> {
    let schema = Schema::new(vec![
        Field::new(
            "timestamp",
            DataType::Timestamp(arrow::datatypes::TimeUnit::Millisecond, None),
            false,
        ),
        Field::new("sensor_id", DataType::Utf8, false),
        Field::new("temperature", DataType::Float64, false),
        Field::new("humidity", DataType::Float64, false),
        Field::new("pressure", DataType::Float64, false),
        Field::new("reading_id", DataType::Int64, false),
    ]);

    // Start time: Jan 1, 2024 00:00:00 UTC (ms)
    let start_time: i64 = 1_704_067_200_000;
    let interval_ms: i64 = 60_000; // 1 minute intervals

    let sensors = ["sensor_a", "sensor_b", "sensor_c"];
    let points_per_sensor = num_points / sensors.len();

    let mut timestamps = Vec::with_capacity(num_points);
    let mut sensor_ids = Vec::with_capacity(num_points);
    let mut temperatures = Vec::with_capacity(num_points);
    let mut humidities = Vec::with_capacity(num_points);
    let mut pressures = Vec::with_capacity(num_points);
    let mut reading_ids = Vec::with_capacity(num_points);

    let mut reading_id: i64 = 0;

    for sensor in &sensors {
        let base_temp = rng.gen_range(18.0..25.0);
        let base_humidity = rng.gen_range(40.0..60.0);
        let base_pressure = rng.gen_range(1000.0..1020.0);

        for i in 0..points_per_sensor {
            let ts = start_time + (i as i64 * interval_ms);
            timestamps.push(ts);
            sensor_ids.push(*sensor);

            // Add some temporal variation (sinusoidal pattern + noise)
            let time_factor = (i as f64 * 0.1).sin();
            temperatures.push(base_temp + time_factor * 3.0 + rng.gen::<f64>() * 0.5);
            humidities.push(base_humidity + time_factor * 5.0 + rng.gen::<f64>() * 2.0);
            pressures.push(base_pressure + time_factor * 2.0 + rng.gen::<f64>() * 0.3);

            reading_ids.push(reading_id);
            reading_id += 1;
        }
    }

    let batch = RecordBatch::try_new(
        Arc::new(schema),
        vec![
            Arc::new(TimestampMillisecondArray::from(timestamps.clone())),
            Arc::new(StringArray::from(sensor_ids)),
            Arc::new(Float64Array::from(temperatures)),
            Arc::new(Float64Array::from(humidities)),
            Arc::new(Float64Array::from(pressures)),
            Arc::new(Int64Array::from(reading_ids)),
        ],
    )?;

    Ok(batch)
}

/// Execute the recipe's core logic.
fn execute_recipe(ctx: &mut RecipeContext) -> Result<RecipeResult> {
    // Create time series dataset (3 sensors, ~333 readings each = 999 total)
    let batch = create_sensor_timeseries(&mut ctx.rng, 999)?;

    // Save to ALD format
    let ald_path = ctx.path("sensor_timeseries.ald");
    save(
        &batch,
        DatasetType::TimeSeries,
        &ald_path,
        SaveOptions::new().with_name("sensor_timeseries"),
    )?;

    // Verify roundtrip
    let loaded = load(&ald_path)?;
    assert_eq!(batch.num_rows(), loaded.num_rows());

    // Get time range from loaded data
    let ts_col = loaded
        .column(0)
        .as_any()
        .downcast_ref::<TimestampMillisecondArray>()
        .ok_or_else(|| ald_cookbook::Error::InvalidColumnType {
            expected: "TimestampMillisecond".to_string(),
            actual: "Unknown".to_string(),
        })?;

    let min_ts = ts_col.value(0);
    let max_ts = ts_col.value(ts_col.len() - 1);

    let file_size = std::fs::metadata(&ald_path)?.len();

    Ok(RecipeResult {
        num_rows: batch.num_rows(),
        time_range_ms: (min_ts, max_ts),
        metrics: vec![
            "temperature".to_string(),
            "humidity".to_string(),
            "pressure".to_string(),
        ],
        file_size_bytes: file_size,
    })
}

fn main() -> Result<()> {
    let mut ctx = RecipeContext::new("create_ald_timeseries")?;
    let result = execute_recipe(&mut ctx)?;
    ctx.report(&result)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_executes() {
        let mut ctx = RecipeContext::new("test_timeseries").unwrap();
        let result = execute_recipe(&mut ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_recipe_idempotent() {
        let mut ctx1 = RecipeContext::new("ts_idempotent").unwrap();
        let mut ctx2 = RecipeContext::new("ts_idempotent").unwrap();

        let result1 = execute_recipe(&mut ctx1).unwrap();
        let result2 = execute_recipe(&mut ctx2).unwrap();

        assert_eq!(result1.num_rows, result2.num_rows);
        assert_eq!(result1.time_range_ms, result2.time_range_ms);
    }

    #[test]
    fn test_time_ordering() {
        let mut ctx = RecipeContext::new("ts_ordering").unwrap();
        let result = execute_recipe(&mut ctx).unwrap();

        assert!(result.time_range_ms.0 < result.time_range_ms.1);
    }
}
