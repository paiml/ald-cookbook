# IIUR Principles

Every recipe in the ALD Cookbook adheres to **IIUR Principles**: Isolated, Idempotent, Useful, and Reproducible.

## The Four Principles

### 1. Isolated

**Definition**: Recipes use temporary directories for all file I/O and avoid global/static mutable state.

**Implementation**:

```rust
use ald_cookbook::RecipeContext;

fn main() -> Result<()> {
    // Creates isolated temp directory
    let ctx = RecipeContext::new("my_recipe")?;

    // All paths are within the temp dir
    let output_path = ctx.temp_path("output.ald");

    // Directory cleaned up when ctx drops
    Ok(())
}
```

**Benefits**:
- No file system pollution
- Parallel execution safe
- No test interference
- Predictable cleanup

### 2. Idempotent

**Definition**: Running a recipe twice with the same seed produces identical output.

**Implementation**:

```rust
use ald_cookbook::RecipeContext;

fn main() -> Result<()> {
    // Deterministic RNG from recipe name hash
    let ctx = RecipeContext::new("my_recipe")?;

    // Always produces the same sequence
    let value = ctx.rng().gen::<f64>();

    // Same seed = same output, every time
    Ok(())
}
```

**Benefits**:
- Reproducible results
- Debugging possible
- CI stability
- No flaky tests

### 3. Useful

**Definition**: Recipes solve real production problems with copy-paste ready code.

**Criteria**:
- Addresses actual data engineering use case
- Code can be used in production with minimal modification
- Demonstrates best practices
- Includes error handling

**Example Use Cases**:
- Creating ML training datasets
- Converting legacy formats
- Detecting data drift in production
- Partitioning data for federated learning

### 4. Reproducible

**Definition**: Recipes work consistently across platforms with pinned dependencies.

**Implementation**:

```toml
# Cargo.lock committed to repository
# Pinned dependency versions
[dependencies]
arrow = "=53.3.0"
zstd = "=0.13.0"
```

**Supported Platforms**:
- Linux (x86_64, aarch64)
- macOS (x86_64, aarch64)
- WASM (wasm32-unknown-unknown)

**CI Verification**:
```yaml
strategy:
  matrix:
    os: [ubuntu-latest, macos-latest]
    rust: [1.75, stable]
```

## RecipeContext API

The `RecipeContext` struct enforces IIUR principles:

```rust
pub struct RecipeContext {
    name: String,
    temp_dir: TempDir,
    rng: StdRng,
    seed: u64,
    start_time: Instant,
}

impl RecipeContext {
    /// Create new isolated context
    pub fn new(name: &str) -> Result<Self>;

    /// Get path within temp directory
    pub fn temp_path(&self, filename: &str) -> PathBuf;

    /// Get deterministic RNG
    pub fn rng(&mut self) -> &mut StdRng;

    /// Get the seed used
    pub fn seed(&self) -> u64;

    /// Report recipe results
    pub fn report<T: Display>(&self, result: &T) -> Result<()>;
}
```

## Recipe Template

```rust
//! # Recipe: [Name]
//!
//! **Category**: [A-L]
//! **Isolation Level**: Full
//! **Idempotency**: Guaranteed
//!
//! ## QA Checklist (10 points)
//! 1. [x] cargo run succeeds (Exit Code 0)
//! 2. [x] cargo test passes
//! 3. [x] Deterministic output
//! 4. [x] No temp files leaked
//! 5. [x] Memory usage stable
//! 6. [x] Platform independent
//! 7. [x] Clippy clean
//! 8. [x] Rustfmt standard
//! 9. [x] No unwrap() in logic
//! 10. [x] Property tests pass

use ald_cookbook::prelude::*;
use ald_cookbook::{RecipeContext, Result};

fn main() -> Result<()> {
    let ctx = RecipeContext::new("recipe_name")?;

    // Recipe logic here
    let result = execute(&ctx)?;

    ctx.report(&result)?;
    Ok(())
}

fn execute(ctx: &RecipeContext) -> Result<String> {
    // Isolated: use ctx.temp_path()
    // Idempotent: use ctx.rng()
    // Useful: solve real problem
    // Reproducible: no platform-specific code
    Ok("Recipe completed".to_string())
}
```

## Testing IIUR Compliance

### Isolation Test

```rust
#[test]
fn test_isolation() {
    let ctx1 = RecipeContext::new("test").unwrap();
    let ctx2 = RecipeContext::new("test").unwrap();

    // Different temp directories
    assert_ne!(ctx1.temp_path("file"), ctx2.temp_path("file"));
}
```

### Idempotency Test

```rust
#[test]
fn test_idempotency() {
    let mut ctx1 = RecipeContext::new("test").unwrap();
    let mut ctx2 = RecipeContext::new("test").unwrap();

    // Same RNG sequence
    assert_eq!(ctx1.rng().gen::<u64>(), ctx2.rng().gen::<u64>());
}
```

### Reproducibility Test (CI)

```yaml
idempotency:
  runs-on: ubuntu-latest
  steps:
    - run: cargo run --example create_ald_from_arrow > run1.txt
    - run: cargo run --example create_ald_from_arrow > run2.txt
    - run: diff run1.txt run2.txt
```

## Common Violations

### Violation: Global State

```rust
// BAD: Global mutable state
static mut COUNTER: u32 = 0;

fn bad_recipe() {
    unsafe { COUNTER += 1; }  // Not isolated!
}
```

### Violation: System Time

```rust
// BAD: Non-deterministic
fn bad_recipe() {
    let now = SystemTime::now();  // Different each run!
}
```

### Violation: Random without Seed

```rust
// BAD: Non-deterministic
fn bad_recipe() {
    let value = rand::random::<f64>();  // Different each run!
}
```

## Next Steps

- [Zero-Copy Loading](./zero-copy-loading.md) - Memory-efficient data access
- [Property-Based Testing](./property-based-testing.md) - Testing invariants
