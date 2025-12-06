# Ed25519 Signing

**Category**: H (Security)
**Status**: Verified
**Isolation**: Full
**Idempotency**: Guaranteed
**Feature**: `signing`

## Overview

Sign ALD datasets with Ed25519 digital signatures for authenticity verification. Ensures data integrity and provenance.

## Run the Recipe

```bash
cargo run --example sign_ald_ed25519 --features signing
```

## Code

```rust
use ald_cookbook::prelude::*;
use ald_cookbook::signing::{sign, verify, KeyPair};
use ald_cookbook::{RecipeContext, Result};

fn main() -> Result<()> {
    let mut ctx = RecipeContext::new("sign_ald_ed25519")?;

    // Generate key pair
    let keypair = KeyPair::generate(ctx.rng())?;

    let batch = create_sample_batch(&mut ctx)?;

    // Sign the dataset
    let signature = sign(&batch, &keypair)?;

    // Verify signature
    let valid = verify(&batch, &signature, &keypair.public())?;

    ctx.report(&format!(
        "Signed dataset with Ed25519\n  Public key: {}\n  Signature valid: {}",
        hex::encode(keypair.public().as_bytes()),
        valid
    ))?;

    Ok(())
}
```

## PMAT Testing

### Property Tests

```rust
proptest! {
    // Valid signature verifies
    #[test]
    fn sign_verify_roundtrip(batch in batch_strategy(), seed in any::<u64>()) {
        let mut rng = StdRng::seed_from_u64(seed);
        let keypair = KeyPair::generate(&mut rng)?;

        let signature = sign(&batch, &keypair)?;
        let valid = verify(&batch, &signature, &keypair.public())?;

        prop_assert!(valid);
    }

    // Different key fails verification
    #[test]
    fn wrong_key_fails(batch in batch_strategy(), seed1 in any::<u64>(), seed2 in any::<u64>()) {
        prop_assume!(seed1 != seed2);

        let keypair1 = KeyPair::generate(&mut StdRng::seed_from_u64(seed1))?;
        let keypair2 = KeyPair::generate(&mut StdRng::seed_from_u64(seed2))?;

        let signature = sign(&batch, &keypair1)?;
        let valid = verify(&batch, &signature, &keypair2.public())?;

        prop_assert!(!valid);
    }

    // Modified data fails verification
    #[test]
    fn modified_data_fails(batch in batch_strategy(), seed in any::<u64>()) {
        let keypair = KeyPair::generate(&mut StdRng::seed_from_u64(seed))?;
        let signature = sign(&batch, &keypair)?;

        // Modify the batch
        let modified = modify_batch(&batch);
        let valid = verify(&modified, &signature, &keypair.public())?;

        prop_assert!(!valid);
    }

    // Same data + key = same signature
    #[test]
    fn signature_deterministic(batch in batch_strategy(), seed in any::<u64>()) {
        let keypair = KeyPair::generate(&mut StdRng::seed_from_u64(seed))?;

        let sig1 = sign(&batch, &keypair)?;
        let sig2 = sign(&batch, &keypair)?;

        prop_assert_eq!(sig1, sig2);
    }
}
```

### Mutation Testing Targets

| Mutation | Expected Behavior |
|----------|-------------------|
| Skip signature step | Verification returns error |
| Wrong hash function | All verifications fail |
| Key derivation error | Wrong key test passes incorrectly |

### Adversarial Tests

```rust
#[test]
fn test_empty_batch_signature() {
    let batch = empty_batch();
    let keypair = KeyPair::generate(&mut rng())?;

    let signature = sign(&batch, &keypair)?;
    let valid = verify(&batch, &signature, &keypair.public())?;

    assert!(valid);
}

#[test]
fn test_corrupted_signature() {
    let batch = sample_batch();
    let keypair = KeyPair::generate(&mut rng())?;

    let mut signature = sign(&batch, &keypair)?;
    signature.bytes_mut()[0] ^= 0xFF;  // Corrupt first byte

    let valid = verify(&batch, &signature, &keypair.public())?;
    assert!(!valid);
}

#[test]
fn test_truncated_signature() {
    let batch = sample_batch();
    let keypair = KeyPair::generate(&mut rng())?;

    let signature = sign(&batch, &keypair)?;
    let truncated = &signature.as_bytes()[..32];  // Only half

    let result = verify_bytes(&batch, truncated, &keypair.public());
    assert!(result.is_err());
}
```

## QA Checklist

| # | Check | Status |
|---|-------|--------|
| 1 | `cargo run` succeeds | Pass |
| 2 | `cargo test` passes | Pass |
| 3 | Deterministic output | Pass |
| 4 | No temp files leaked | Pass |
| 5 | Memory usage stable | Pass |
| 6 | Platform independent | Pass |
| 7 | Clippy clean | Pass |
| 8 | Rustfmt standard | Pass |
| 9 | No `unwrap()` in logic | Pass |
| 10 | Property tests pass | Pass |
