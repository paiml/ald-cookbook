# QA Team Validation Prompt

## Instructions for QA Team

You are validating the **ALD Cookbook** implementation against a 100-point quality checklist. The checklist is located at `docs/qa/qa-checklist-100.md`.

---

## Your Task

1. **Clone and setup the repository**:
   ```bash
   cd /path/to/ald-cookbook
   ```

2. **Execute the Quick Validation Script** at the bottom of the checklist, OR manually verify each of the 100 criteria.

3. **For each criterion**:
   - Mark `[x]` if PASS
   - Mark `[ ]` if FAIL
   - Add notes for any failures

4. **Calculate final score** (each `[x]` = 1 point)

5. **Report findings** with:
   - Total score out of 100
   - PASS (95+) or FAIL (<95)
   - List of failed criteria with explanations
   - Recommendations for fixes

---

## Key Validation Commands

```bash
# Full build verification
cargo build && cargo build --release && cargo build --all-features && cargo build --examples

# Run all tests
cargo test

# Run all tests including feature-gated
cargo test --all-features

# Check code quality
cargo fmt --check
cargo clippy --examples

# Run example recipes (spot check)
cargo run --example create_ald_from_arrow
cargo run --example drift_ks_test
cargo run --example federated_iid_split
cargo run --example sign_ald_ed25519 --features signing
cargo run --example registry_publish
cargo run --example cli_ald_info

# Idempotency test
cargo run --example create_ald_from_arrow > /tmp/out1.txt 2>&1
cargo run --example create_ald_from_arrow > /tmp/out2.txt 2>&1
diff /tmp/out1.txt /tmp/out2.txt  # Should show no differences

# Run benchmarks
cargo bench

# Generate documentation
cargo doc --no-deps --open
```

---

## Critical Acceptance Criteria

The following are **MUST PASS** criteria (failure = automatic rejection):

1. **A1.1**: `cargo build` succeeds (exit code 0)
2. **B1.1**: All unit tests pass
3. **C2.1**: Idempotent output (deterministic recipes)
4. **D3.1**: `#![forbid(unsafe_code)]` enforced
5. **E3.1**: Data survives save/load roundtrip
6. **F1.1 - F10.1**: At least one recipe per category executes

---

## Expected Results

Based on the implementation, you should expect:

| Metric | Expected |
|--------|----------|
| Unit tests | 89 passing |
| Doc tests | 3 passing |
| Examples | 22 compilable |
| Clippy | 0 errors, ~61 warnings (precision loss - expected) |
| Format | Passes `cargo fmt --check` |

---

## Reporting Template

```markdown
# ALD Cookbook QA Validation Report

**Date**: YYYY-MM-DD
**Validator**: [Your Name]
**Repository**: ald-cookbook
**Commit**: [git commit hash]

## Summary

| Category | Points | Max |
|----------|--------|-----|
| A. Build & Compilation | /10 | 10 |
| B. Test Suite | /10 | 10 |
| C. IIUR Compliance | /10 | 10 |
| D. Code Quality | /10 | 10 |
| E. ALD Format | /10 | 10 |
| F. Recipe Categories | /10 | 10 |
| G. API Correctness | /10 | 10 |
| H. Documentation | /10 | 10 |
| I. Security & Features | /10 | 10 |
| J. Performance & Edge Cases | /10 | 10 |
| **TOTAL** | **/100** | **100** |

**Status**: [ ] PASS (95+) / [ ] FAIL (<95)

## Failed Criteria

| ID | Criterion | Reason | Severity |
|----|-----------|--------|----------|
| | | | |

## Recommendations

1.
2.
3.

## Sign-Off

- [ ] QA validation complete
- [ ] Report reviewed
- [ ] Ready for release: YES / NO
```

---

## Contact

For questions about the checklist or implementation:
- Specification: `docs/specifications/ald-cookbook-spec.md`
- Implementation: `src/` directory
- Examples: `examples/` directory

---

*Please complete validation within 2 business days and submit the filled report.*
