# Create TimeSeries Dataset

**Category**: A (Dataset Creation)
**Status**: Verified
**Isolation**: Full
**Idempotency**: Guaranteed

## Overview

Create a time series dataset with temporal indices and multiple value columns.

## Run the Recipe

```bash
cargo run --example create_ald_timeseries
```

## Code Highlights

```rust
let schema = Arc::new(Schema::new(vec![
    Field::new("timestamp", DataType::Timestamp(TimeUnit::Millisecond, None), false),
    Field::new("value", DataType::Float64, false),
    Field::new("sensor_id", DataType::Int32, false),
]));
```

## Use Cases

- IoT sensor data collection
- Financial market data
- Monitoring and observability

## QA Checklist

All 10 points verified. See [QA Checklist](../../appendix/qa-checklist.md).
