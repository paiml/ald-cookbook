# API Documentation

## Core Types

### RecipeContext

Provides isolation and determinism for recipes:

```rust
pub struct RecipeContext {
    name: String,
    temp_dir: TempDir,
    rng: StdRng,
    seed: u64,
}

impl RecipeContext {
    pub fn new(name: &str) -> Result<Self>;
    pub fn temp_path(&self, filename: &str) -> PathBuf;
    pub fn rng(&mut self) -> &mut StdRng;
    pub fn seed(&self) -> u64;
    pub fn report<T: Display>(&self, result: &T) -> Result<()>;
}
```

### Error

Domain-specific error type:

```rust
pub enum Error {
    Io(std::io::Error),
    Arrow(arrow::error::ArrowError),
    Parquet(parquet::errors::ParquetError),
    InvalidFormat(String),
    ChecksumMismatch { expected: u32, actual: u32 },
    UnsupportedVersion { major: u8, minor: u8 },
    InvalidMetadata(String),
}
```

### Result

Type alias for convenience:

```rust
pub type Result<T> = std::result::Result<T, Error>;
```

## Format Module

### load

Load an ALD file:

```rust
pub fn load<P: AsRef<Path>>(path: P) -> Result<(RecordBatch, Metadata)>;
```

### save

Save a RecordBatch as ALD:

```rust
pub fn save<P: AsRef<Path>>(
    batch: &RecordBatch,
    path: P,
    options: SaveOptions,
) -> Result<()>;
```

### load_metadata

Load only metadata (fast):

```rust
pub fn load_metadata<P: AsRef<Path>>(path: P) -> Result<Metadata>;
```

### SaveOptions

Configuration for saving:

```rust
pub struct SaveOptions {
    pub name: String,
    pub description: String,
    pub dataset_type: DatasetType,
    pub compression_level: i32,
}

impl SaveOptions {
    pub fn new(name: &str) -> Self;
    pub fn with_description(self, desc: &str) -> Self;
    pub fn with_dataset_type(self, dtype: DatasetType) -> Self;
    pub fn with_compression_level(self, level: i32) -> Self;
}
```

### DatasetType

Dataset type enumeration:

```rust
pub enum DatasetType {
    Tabular,
    TimeSeries,
    Text,
    Image,
}
```

### Metadata

Dataset metadata:

```rust
pub struct Metadata {
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub row_count: u64,
    pub schema_hash: String,
    pub dataset_type: DatasetType,
}
```

## Transforms Module

### filter

Filter rows by predicate:

```rust
pub fn filter<F>(batch: &RecordBatch, predicate: F) -> Result<RecordBatch>
where
    F: Fn(usize) -> bool;
```

### shuffle

Deterministic shuffle:

```rust
pub fn shuffle(batch: &RecordBatch, seed: u64) -> Result<RecordBatch>;
```

### sample

Random sampling:

```rust
pub fn sample(batch: &RecordBatch, ratio: f64, seed: u64) -> Result<RecordBatch>;
```

### normalize

Normalize numeric columns:

```rust
pub fn normalize(batch: &RecordBatch, columns: &[&str]) -> Result<RecordBatch>;
```

## Drift Module

### ks_test

Kolmogorov-Smirnov test:

```rust
pub fn ks_test(baseline: &[f64], current: &[f64]) -> Result<KsResult>;

pub struct KsResult {
    pub statistic: f64,
    pub p_value: f64,
    pub drift_detected: bool,
}
```

### calculate_psi

Population Stability Index:

```rust
pub fn calculate_psi(
    baseline: &[f64],
    current: &[f64],
    bins: usize,
) -> Result<f64>;
```

## Quality Module

### null_count

Count null values:

```rust
pub fn null_count(batch: &RecordBatch) -> HashMap<String, usize>;
```

### detect_outliers

Detect outliers using IQR:

```rust
pub fn detect_outliers(values: &[f64]) -> Vec<usize>;
```

## Federated Module

### iid_split

IID data partitioning:

```rust
pub fn iid_split(
    batch: &RecordBatch,
    num_clients: usize,
    seed: u64,
) -> Result<Vec<RecordBatch>>;
```

### dirichlet_split

Non-IID Dirichlet partitioning:

```rust
pub fn dirichlet_split(
    batch: &RecordBatch,
    num_clients: usize,
    alpha: f64,
    seed: u64,
) -> Result<Vec<RecordBatch>>;
```

## Convert Module

### parquet_to_ald

Convert Parquet to ALD:

```rust
pub fn parquet_to_ald<P: AsRef<Path>>(
    input: P,
    output: P,
    options: SaveOptions,
) -> Result<()>;
```

### csv_to_ald

Convert CSV to ALD:

```rust
pub fn csv_to_ald<P: AsRef<Path>>(
    input: P,
    output: P,
    options: SaveOptions,
) -> Result<()>;
```

## Registry Module

### LocalRegistry

File-based dataset registry:

```rust
pub struct LocalRegistry {
    root: PathBuf,
}

impl LocalRegistry {
    pub fn new<P: AsRef<Path>>(root: P) -> Result<Self>;
    pub fn publish(&self, batch: &RecordBatch, name: &str, version: &str) -> Result<PathBuf>;
    pub fn pull(&self, name: &str, version: &str) -> Result<(RecordBatch, Metadata)>;
    pub fn list(&self) -> Result<Vec<DatasetInfo>>;
}
```
