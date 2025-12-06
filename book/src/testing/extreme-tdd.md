# Extreme TDD Methodology

Extreme Test-Driven Development (TDD) goes beyond traditional TDD by combining multiple testing strategies into a comprehensive quality assurance system.

## The Testing Pyramid

```
                    ▲
                   /│\
                  / │ \      E2E Tests
                 /  │  \     (CI Validation)
                /───│───\
               /    │    \   Integration Tests
              /     │     \  (Recipe Execution)
             /──────│──────\
            /       │       \ Property Tests
           /        │        \(Invariant Verification)
          /─────────│─────────\
         /          │          \ Unit Tests
        /───────────│───────────\(Module Behavior)
```

## Four Pillars of Extreme TDD

### 1. Unit Tests (Foundation)

Traditional unit tests verify individual functions and methods:

```rust
#[test]
fn test_header_magic_bytes() {
    let header = Header::new("test", 100);
    assert_eq!(&header.magic, b"ALDF");
}
```

**Requirements:**
- 95% line coverage minimum
- Test all public APIs
- Test edge cases (empty, single, boundary)

### 2. Property Tests (Invariants)

Property-based tests verify invariants across random inputs:

```rust
proptest! {
    #[test]
    fn roundtrip_preserves_data(rows in 1..10000usize) {
        let batch = generate_batch(rows);
        let path = temp_path();
        save(&batch, &path)?;
        let (loaded, _) = load(&path)?;
        prop_assert_eq!(batch.num_rows(), loaded.num_rows());
    }
}
```

**Requirements:**
- 3+ properties per module
- 50 cases for fast tests
- 500 cases for comprehensive tests

### 3. Integration Tests (Recipes)

Recipes serve as integration tests verifying end-to-end functionality:

```rust
fn main() -> Result<()> {
    let ctx = RecipeContext::new("create_ald_from_arrow")?;
    // Full workflow execution
    let result = execute(&ctx)?;
    ctx.report(&result)?;
    Ok(())
}
```

**Requirements:**
- All 22 recipes must pass
- Idempotent execution (same output twice)
- IIUR compliance

### 4. E2E Tests (CI Validation)

CI pipeline verifies the complete system:

```yaml
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - run: cargo test --all-features
      - run: cargo run --example create_ald_from_arrow
```

## TDD Workflow

### Red-Green-Refactor

1. **Red**: Write a failing test
2. **Green**: Write minimal code to pass
3. **Refactor**: Improve while keeping tests green

### Property-First Development

1. Identify invariants before implementation
2. Write property tests first
3. Implement until properties pass
4. Add edge case unit tests

## Quality Gates

| Gate | Threshold | Tool |
|------|-----------|------|
| Line Coverage | 95% | cargo-llvm-cov |
| Branch Coverage | 90% | cargo-llvm-cov |
| Mutation Score | 80% | cargo-mutants |
| Property Cases | 50+ | proptest |
| Clippy Warnings | 0 | cargo clippy |

## Make Targets

```bash
# Fast development cycle
make test-fast     # Unit tests only

# Before commit
make validate      # Full validation

# Comprehensive
make test-all      # All tests + properties
make coverage      # Coverage report
make mutants       # Mutation testing
```

## Best Practices

1. **Test first, always**: Never write code without a test
2. **Property thinking**: Ask "what should always be true?"
3. **Mutation testing**: Verify tests catch bugs
4. **Coverage discipline**: No untested code paths
5. **CI enforcement**: All gates must pass
