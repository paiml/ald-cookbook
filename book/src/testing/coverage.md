# Coverage Requirements

## Thresholds

| Metric | Minimum | Target |
|--------|---------|--------|
| Line Coverage | 95% | 98% |
| Branch Coverage | 90% | 95% |
| Function Coverage | 100% | 100% |

## Setup

```bash
# Install coverage tool
cargo install cargo-llvm-cov

# Requires LLVM/Clang
rustup component add llvm-tools-preview
```

## Running Coverage

```bash
# Basic coverage report
cargo llvm-cov

# With HTML report
cargo llvm-cov --html
open target/llvm-cov/html/index.html

# Fail if below threshold
cargo llvm-cov --fail-under 95

# With all features
cargo llvm-cov --all-features --fail-under 95
```

## Coverage Output

```
Filename                      Regions    Missed  Cover   Lines  Missed   Cover
----------------------------  -------    ------  -----   -----  ------   -----
src/format.rs                    156        8   94.87%    776      35   95.49%
src/transforms.rs                 89        4   95.51%    532      22   95.86%
src/drift.rs                     102        5   95.10%    600      28   95.33%
src/federated.rs                  95        4   95.79%    628      25   96.02%
src/quality.rs                   110        5   95.45%    695      30   95.68%
src/convert.rs                    88        4   95.45%    614      28   95.44%
src/registry.rs                   92        4   95.65%    573      26   95.46%
src/context.rs                    45        2   95.56%    273      12   95.60%
src/error.rs                      35        1   97.14%    217       8   96.31%
----------------------------  -------    ------  -----   -----  ------   -----
TOTAL                            892       39   95.63%   5979     245   95.90%
```

## Improving Coverage

### 1. Identify Uncovered Lines

```bash
cargo llvm-cov --html
# Look for red-highlighted lines in report
```

### 2. Add Missing Tests

```rust
// Uncovered: error path when file doesn't exist
#[test]
fn test_load_nonexistent_file() {
    let result = load("nonexistent.ald");
    assert!(matches!(result, Err(Error::Io(_))));
}
```

### 3. Test All Branches

```rust
// Branch coverage: test both paths
#[test]
fn test_optional_field() {
    let metadata = Metadata::new("test", "");  // empty description
    assert!(metadata.description.is_empty());

    let metadata = Metadata::new("test", "desc");  // with description
    assert!(!metadata.description.is_empty());
}
```

## CI Integration

```yaml
coverage:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - name: Install coverage tool
      run: cargo install cargo-llvm-cov
    - name: Run coverage
      run: cargo llvm-cov --all-features --lcov --output-path lcov.info --fail-under 95
    - name: Upload coverage
      uses: codecov/codecov-action@v4
      with:
        files: lcov.info
```

## Exclusions

Some code is intentionally excluded from coverage:

```rust
#[cfg(not(tarpaulin_include))]
fn debug_only_function() {
    // Not counted in coverage
}
```

Use sparingly for:
- Debug-only code
- Platform-specific code
- Unreachable error handlers
