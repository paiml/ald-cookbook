# Installation

## Prerequisites

- **Rust 1.75+** (MSRV)
- **Cargo** (included with Rust)
- Optional: `cargo-llvm-cov` for coverage
- Optional: `cargo-mutants` for mutation testing

## Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update stable
```

## Clone the Repository

```bash
git clone https://github.com/paiml/ald-cookbook.git
cd ald-cookbook
```

## Verify Installation

```bash
# Run tests
cargo test

# Run a recipe
cargo run --example create_ald_from_arrow
```

## Install Development Tools

For the full TDD experience:

```bash
# Coverage tool
cargo install cargo-llvm-cov

# Mutation testing
cargo install cargo-mutants

# Fast test runner
cargo install cargo-nextest

# mdBook for documentation
cargo install mdbook
```

## Feature Flags

Enable specific features based on your needs:

```bash
# Core functionality (default)
cargo build

# With encryption support
cargo build --features encryption

# With Ed25519 signing
cargo build --features signing

# All features
cargo build --features full
```

## Verify Quality Tools

```bash
# Coverage check (95% minimum)
cargo llvm-cov --fail-under 95

# Mutation testing (80% minimum)
cargo mutants --timeout 60

# Clippy (zero warnings)
cargo clippy -- -D warnings
```

## Building the Book

```bash
cd book
mdbook build
mdbook serve --open
```

## IDE Setup

### VS Code

Install the `rust-analyzer` extension for:
- Inline type hints
- Go to definition
- Auto-completion
- Inline test running

### Settings

Add to `.vscode/settings.json`:

```json
{
    "rust-analyzer.cargo.features": "all",
    "rust-analyzer.checkOnSave.command": "clippy"
}
```

## Next Steps

- [Quick Start](./quick-start.md) - Run your first recipe
- [Project Structure](./project-structure.md) - Understand the codebase
