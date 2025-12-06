# Toyota Way Principles

The ALD Cookbook applies Toyota Production System (TPS) principles to dataset engineering.

## Core Principles

### 1. Jidoka (Built-in Quality)

**Definition**: Build quality into the process, stopping to fix problems immediately.

**Application in ALD Cookbook**:
- Rust's type system prevents invalid states at compile time
- Property-based tests verify invariants automatically
- CI fails fast on any quality gate violation

```rust
// Type system enforces valid metadata
pub struct Metadata {
    name: NonEmptyString,     // Cannot be empty
    row_count: NonZeroU64,    // Must be positive
    created_at: DateTime<Utc>,// Valid timestamp
}
```

### 2. Muda (Eliminate Waste)

**Definition**: Identify and eliminate non-value-adding activities.

**Application in ALD Cookbook**:
- Zero-copy loading avoids unnecessary memory allocation
- Zstd compression reduces storage and transfer costs
- IIUR principles prevent flaky tests (time waste)

**Seven Types of Waste**:
| Waste Type | Traditional | ALD Cookbook Solution |
|------------|-------------|----------------------|
| Overproduction | Extra features | YAGNI - minimal features |
| Waiting | Slow tests | Parallel execution, fast feedback |
| Transport | Data copying | Zero-copy Arrow operations |
| Processing | Complex code | Simple, focused functions |
| Inventory | Unused code | Regular dead code removal |
| Motion | Context switching | IIUR isolation |
| Defects | Bugs | 95% coverage, mutation testing |

### 3. Heijunka (Level Production)

**Definition**: Maintain consistent, predictable output.

**Application in ALD Cookbook**:
- Every recipe follows the same structure
- Consistent error handling patterns
- Predictable file format layout

```rust
// Every recipe has this structure
fn main() -> Result<()> {
    let ctx = RecipeContext::new("name")?;
    let result = execute(&ctx)?;
    ctx.report(&result)?;
    Ok(())
}
```

### 4. Poka-Yoke (Error Prevention)

**Definition**: Design systems that make errors impossible or immediately visible.

**Application in ALD Cookbook**:
- Compile-time type checking prevents data type mismatches
- Deterministic RNG prevents flaky tests
- Checksum verification detects corruption

```rust
// Poka-Yoke: Can't create invalid header
impl Header {
    pub fn new(name: &str, rows: u64) -> Self {
        Self {
            magic: *b"ALDF",  // Always correct
            version_major: 1,  // Constant
            version_minor: 2,
            // ...
        }
    }
}
```

### 5. Kaizen (Continuous Improvement)

**Definition**: Continuously improve processes through small, incremental changes.

**Application in ALD Cookbook**:
- 95% coverage threshold with gradual increase
- Mutation testing identifies weak spots
- Regular dependency updates

**Improvement Cycle**:
1. Measure (coverage, mutation score)
2. Identify gaps (uncovered code, missed mutations)
3. Improve (add tests, refactor)
4. Verify (CI passes)
5. Repeat

### 6. Genchi Genbutsu (Go and See)

**Definition**: Go to the source to understand problems firsthand.

**Application in ALD Cookbook**:
- Recipes run actual operations, not mocks
- Integration tests use real file I/O
- Examples demonstrate real-world usage

```rust
// Real file I/O, not mocked
#[test]
fn test_save_load() {
    let path = temp_path();
    save(&batch, &path, options)?;
    let (loaded, _) = load(&path)?;  // Real file read
    assert_eq!(batch.num_rows(), loaded.num_rows());
}
```

## Quality Matrix

| Principle | Metric | Target |
|-----------|--------|--------|
| Jidoka | Compile errors | 0 |
| Muda | Unnecessary copies | 0 |
| Heijunka | Recipe structure variance | 0 |
| Poka-Yoke | Runtime panics | 0 |
| Kaizen | Coverage | 95%+ |
| Genchi Genbutsu | Mock usage | Minimal |

## Implementation Checklist

- [ ] Type system enforces invariants
- [ ] Zero-copy where possible
- [ ] Consistent code structure
- [ ] Deterministic execution
- [ ] Measurable quality metrics
- [ ] Real integration tests
