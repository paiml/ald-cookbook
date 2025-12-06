//! Benchmarks for ALD format operations.

use ald_cookbook::format::{self, DatasetType, SaveOptions};
use arrow::array::{Float64Array, Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::sync::Arc;

fn create_test_batch(num_rows: usize) -> RecordBatch {
    let schema = Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("value", DataType::Float64, false),
        Field::new("label", DataType::Utf8, true),
    ]);

    let id_array: Int64Array = (0..num_rows as i64).collect();
    let value_array: Float64Array = (0..num_rows).map(|i| i as f64 * 0.1).collect();
    let label_array: StringArray = (0..num_rows)
        .map(|i| Some(format!("label_{}", i % 100)))
        .collect();

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

fn bench_save(c: &mut Criterion) {
    let mut group = c.benchmark_group("save");

    for size in [1_000, 10_000, 100_000].iter() {
        let batch = create_test_batch(*size);
        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(BenchmarkId::new("compressed", size), size, |b, _| {
            let temp = tempfile::tempdir().unwrap();
            let path = temp.path().join("bench.ald");
            b.iter(|| {
                format::save(
                    black_box(&batch),
                    DatasetType::Tabular,
                    &path,
                    SaveOptions::new(),
                )
                .unwrap();
            });
        });

        group.bench_with_input(BenchmarkId::new("uncompressed", size), size, |b, _| {
            let temp = tempfile::tempdir().unwrap();
            let path = temp.path().join("bench.ald");
            b.iter(|| {
                format::save(
                    black_box(&batch),
                    DatasetType::Tabular,
                    &path,
                    SaveOptions::new().without_compression(),
                )
                .unwrap();
            });
        });
    }

    group.finish();
}

fn bench_load(c: &mut Criterion) {
    let mut group = c.benchmark_group("load");

    for size in [1_000, 10_000, 100_000].iter() {
        let batch = create_test_batch(*size);
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("bench.ald");
        format::save(&batch, DatasetType::Tabular, &path, SaveOptions::new()).unwrap();

        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(BenchmarkId::new("from_file", size), size, |b, _| {
            b.iter(|| {
                format::load(black_box(&path)).unwrap();
            });
        });

        let bytes = std::fs::read(&path).unwrap();
        group.bench_with_input(BenchmarkId::new("from_bytes", size), size, |b, _| {
            b.iter(|| {
                format::load_from_bytes(black_box(&bytes)).unwrap();
            });
        });
    }

    group.finish();
}

fn bench_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("roundtrip");

    for size in [1_000, 10_000].iter() {
        let batch = create_test_batch(*size);
        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            let temp = tempfile::tempdir().unwrap();
            let path = temp.path().join("bench.ald");
            b.iter(|| {
                format::save(
                    black_box(&batch),
                    DatasetType::Tabular,
                    &path,
                    SaveOptions::new(),
                )
                .unwrap();
                format::load(&path).unwrap();
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_save, bench_load, bench_roundtrip);
criterion_main!(benches);
