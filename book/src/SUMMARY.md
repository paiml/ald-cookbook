# Summary

[Introduction](./introduction.md)

# Getting Started

- [Installation](./getting-started/installation.md)
- [Quick Start](./getting-started/quick-start.md)
- [Project Structure](./getting-started/project-structure.md)

# Core Concepts

- [The ALD Format](./concepts/ald-format.md)
- [IIUR Principles](./concepts/iiur-principles.md)
- [Zero-Copy Loading](./concepts/zero-copy-loading.md)
- [Property-Based Testing](./concepts/property-based-testing.md)

# Recipe Categories

- [A: Dataset Creation](./recipes/a-dataset-creation/index.md)
    - [Create ALD from Arrow](./recipes/a-dataset-creation/create-ald-from-arrow.md)
    - [Create Tabular Dataset](./recipes/a-dataset-creation/create-ald-tabular.md)
    - [Create TimeSeries Dataset](./recipes/a-dataset-creation/create-ald-timeseries.md)
    - [Create Text Corpus](./recipes/a-dataset-creation/create-ald-text-corpus.md)
    - [Create Image Dataset](./recipes/a-dataset-creation/create-ald-image-dataset.md)

- [B: Loading & Streaming](./recipes/b-loading/index.md)
    - [Load ALD Basic](./recipes/b-loading/load-ald-basic.md)
    - [Load ALD Metadata](./recipes/b-loading/load-ald-metadata.md)

- [C: Format Conversion](./recipes/c-conversion/index.md)
    - [Convert Parquet to ALD](./recipes/c-conversion/convert-parquet-to-ald.md)
    - [Convert CSV to ALD](./recipes/c-conversion/convert-csv-to-ald.md)

- [D: Data Transforms](./recipes/d-transforms/index.md)
    - [Filter Transform](./recipes/d-transforms/transform-filter.md)
    - [Shuffle Transform](./recipes/d-transforms/transform-shuffle.md)
    - [Sample Transform](./recipes/d-transforms/transform-sample.md)

- [E: Data Quality](./recipes/e-quality/index.md)
    - [Null Detection](./recipes/e-quality/quality-null-detection.md)

- [F: Drift Detection](./recipes/f-drift/index.md)
    - [KS Test](./recipes/f-drift/drift-ks-test.md)
    - [PSI Analysis](./recipes/f-drift/drift-psi.md)

- [G: Federated Learning](./recipes/g-federated/index.md)
    - [IID Split](./recipes/g-federated/federated-iid-split.md)
    - [Dirichlet Split](./recipes/g-federated/federated-dirichlet-split.md)

- [H: Security & Encryption](./recipes/h-security/index.md)
    - [Ed25519 Signing](./recipes/h-security/sign-ald-ed25519.md)

- [I: Registry & Distribution](./recipes/i-registry/index.md)
    - [Publish Dataset](./recipes/i-registry/registry-publish.md)
    - [Pull Dataset](./recipes/i-registry/registry-pull.md)

- [L: CLI Tools](./recipes/l-cli/index.md)
    - [ALD Info](./recipes/l-cli/cli-ald-info.md)

# Testing & Quality

- [Extreme TDD Methodology](./testing/extreme-tdd.md)
- [Property-Based Testing](./testing/property-testing.md)
- [Mutation Testing](./testing/mutation-testing.md)
- [Coverage Requirements](./testing/coverage.md)

# Reference

- [API Documentation](./reference/api.md)
- [Error Handling](./reference/errors.md)
- [Feature Flags](./reference/features.md)

# Appendix

- [Toyota Way Principles](./appendix/toyota-way.md)
- [Recipe QA Checklist](./appendix/qa-checklist.md)
- [PMAT Testing Strategy](./appendix/pmat-strategy.md)
