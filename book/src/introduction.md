# Introduction

**ALD Cookbook** is a comprehensive collection of production-ready recipes demonstrating the **Alimentar Dataset Format** (`.ald`) - a secure, verifiable dataset distribution format built on Apache Arrow IPC with zstd compression.

## Philosophy: Extreme TDD with Toyota Production System

This cookbook applies **Extreme Test-Driven Development** principles informed by the Toyota Production System:

| Toyota Principle | Application in ALD Cookbook |
|------------------|----------------------------|
| **Jidoka** (Built-in Quality) | Type system prevents invalid states; property tests verify invariants |
| **Muda** (Eliminate Waste) | Zero-copy loading; no unnecessary allocations |
| **Heijunka** (Level Production) | Consistent recipe structure across all categories |
| **Poka-Yoke** (Error Prevention) | Compile-time checks; deterministic RNG prevents flaky tests |
| **Kaizen** (Continuous Improvement) | 95% coverage minimum; mutation testing |

## IIUR Principles

Every recipe in this cookbook adheres to **IIUR Principles**:

1. **Isolated**: Uses `tempfile::tempdir()` for all file I/O; no global state
2. **Idempotent**: Fixed RNG seeds produce identical output on every run
3. **Useful**: Solves real production problems with copy-paste ready code
4. **Reproducible**: Works on Linux, macOS, and WASM; pinned dependencies

## Recipe Categories

| Category | Count | Focus |
|----------|-------|-------|
| **A**: Dataset Creation | 5 | Create ALD from Arrow, tabular, time series, text, images |
| **B**: Loading & Streaming | 2 | Zero-copy loading, metadata inspection |
| **C**: Format Conversion | 2 | Parquet → ALD, CSV → ALD |
| **D**: Data Transforms | 3 | Filter, shuffle, sample operations |
| **E**: Data Quality | 1 | Null detection, schema validation |
| **F**: Drift Detection | 2 | KS test, PSI analysis |
| **G**: Federated Learning | 2 | IID and Dirichlet splits |
| **H**: Security | 1 | Ed25519 digital signatures |
| **I**: Registry | 2 | Publish and pull datasets |
| **L**: CLI Tools | 1 | Dataset inspection |

## Quality Gates

Before any code is merged, it must pass:

- **95% Line Coverage** - Verified via `cargo llvm-cov`
- **80% Mutation Score** - Verified via `cargo mutants`
- **Property Tests** - 3+ properties per module via `proptest`
- **10-Point QA Checklist** - Every recipe manually verified
- **Zero `unwrap()` Policy** - All errors handled explicitly

## Testing Pyramid

```
                    ▲
                   /│\
                  / │ \
                 /  │  \
                /   │   \  E2E Tests (CI validation)
               /    │    \
              /─────│─────\
             /      │      \
            /       │       \  Integration Tests (recipe execution)
           /────────│────────\
          /         │         \
         /          │          \  Property Tests (invariant verification)
        /───────────│───────────\
       /            │            \
      /             │             \  Unit Tests (module behavior)
     /──────────────│──────────────\
```

## How to Use This Book

1. **Getting Started**: Install dependencies and run your first recipe
2. **Core Concepts**: Understand the ALD format and testing methodology
3. **Recipe Categories**: Browse recipes by use case
4. **Testing & Quality**: Learn the TDD methodology
5. **Reference**: API documentation and feature flags

## Running a Recipe

```bash
# Run any recipe
cargo run --example create_ald_from_arrow

# Run with specific features
cargo run --example sign_ald_ed25519 --features signing

# Run all tests
cargo test --all-features
```

## Contributing

All contributions must:

1. Follow the canonical recipe structure
2. Include property-based tests
3. Pass the 10-point QA checklist
4. Maintain 95% code coverage
5. Include book documentation

Welcome to Extreme TDD for dataset engineering!
