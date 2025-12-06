//! Dataset signing utilities (Ed25519).
//!
//! Provides digital signatures for datasets using Ed25519.
//!
//! Requires the `signing` feature.

use crate::error::{Error, Result};
use ed25519_dalek::{
    Signature, Signer, SigningKey, Verifier, VerifyingKey, PUBLIC_KEY_LENGTH, SECRET_KEY_LENGTH,
    SIGNATURE_LENGTH,
};
use rand::rngs::OsRng;

/// A keypair for signing and verification.
#[derive(Clone)]
pub struct Keypair {
    /// Private signing key.
    signing_key: SigningKey,
}

impl Keypair {
    /// Generate a new random keypair.
    #[must_use]
    pub fn generate() -> Self {
        use rand::RngCore;
        let mut bytes = [0u8; SECRET_KEY_LENGTH];
        OsRng.fill_bytes(&mut bytes);
        let signing_key = SigningKey::from_bytes(&bytes);
        Self { signing_key }
    }

    /// Create a keypair from secret key bytes.
    ///
    /// # Errors
    ///
    /// Returns `Error::Signing` if the key is invalid.
    pub fn from_bytes(secret_key: &[u8; SECRET_KEY_LENGTH]) -> Result<Self> {
        let signing_key = SigningKey::from_bytes(secret_key);
        Ok(Self { signing_key })
    }

    /// Get the secret key bytes.
    #[must_use]
    pub fn secret_key_bytes(&self) -> [u8; SECRET_KEY_LENGTH] {
        self.signing_key.to_bytes()
    }

    /// Get the public key bytes.
    #[must_use]
    pub fn public_key_bytes(&self) -> [u8; PUBLIC_KEY_LENGTH] {
        self.signing_key.verifying_key().to_bytes()
    }

    /// Get the verifying (public) key.
    #[must_use]
    pub fn verifying_key(&self) -> VerifyingKey {
        self.signing_key.verifying_key()
    }

    /// Sign data.
    #[must_use]
    pub fn sign(&self, data: &[u8]) -> SignatureData {
        let signature = self.signing_key.sign(data);
        SignatureData {
            signature: signature.to_bytes(),
            public_key: self.public_key_bytes(),
        }
    }
}

impl std::fmt::Debug for Keypair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Keypair")
            .field("public_key", &hex::encode(&self.public_key_bytes()))
            .finish()
    }
}

/// Signature data containing signature and public key.
#[derive(Debug, Clone)]
pub struct SignatureData {
    /// The signature bytes.
    pub signature: [u8; SIGNATURE_LENGTH],
    /// The public key bytes.
    pub public_key: [u8; PUBLIC_KEY_LENGTH],
}

impl SignatureData {
    /// Serialize to bytes.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(SIGNATURE_LENGTH + PUBLIC_KEY_LENGTH);
        bytes.extend_from_slice(&self.signature);
        bytes.extend_from_slice(&self.public_key);
        bytes
    }

    /// Deserialize from bytes.
    ///
    /// # Errors
    ///
    /// Returns `Error::SignatureVerification` if data is invalid.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != SIGNATURE_LENGTH + PUBLIC_KEY_LENGTH {
            return Err(Error::SignatureVerification(format!(
                "Invalid signature data length: expected {}, got {}",
                SIGNATURE_LENGTH + PUBLIC_KEY_LENGTH,
                bytes.len()
            )));
        }

        let mut signature = [0u8; SIGNATURE_LENGTH];
        let mut public_key = [0u8; PUBLIC_KEY_LENGTH];

        signature.copy_from_slice(&bytes[..SIGNATURE_LENGTH]);
        public_key.copy_from_slice(&bytes[SIGNATURE_LENGTH..]);

        Ok(Self {
            signature,
            public_key,
        })
    }

    /// Verify the signature against data.
    ///
    /// # Errors
    ///
    /// Returns `Error::SignatureVerification` if verification fails.
    pub fn verify(&self, data: &[u8]) -> Result<()> {
        let verifying_key = VerifyingKey::from_bytes(&self.public_key)
            .map_err(|e| Error::SignatureVerification(format!("Invalid public key: {e}")))?;

        let signature = Signature::from_bytes(&self.signature);

        verifying_key.verify(data, &signature).map_err(|e| {
            Error::SignatureVerification(format!("Signature verification failed: {e}"))
        })
    }
}

/// Sign data with a keypair.
#[must_use]
pub fn sign(data: &[u8], keypair: &Keypair) -> SignatureData {
    keypair.sign(data)
}

/// Verify a signature.
///
/// # Errors
///
/// Returns `Error::SignatureVerification` if verification fails.
pub fn verify(data: &[u8], signature_data: &SignatureData) -> Result<()> {
    signature_data.verify(data)
}

/// Verify signature using public key bytes.
///
/// # Errors
///
/// Returns `Error::SignatureVerification` if verification fails.
pub fn verify_with_public_key(
    data: &[u8],
    signature: &[u8; SIGNATURE_LENGTH],
    public_key: &[u8; PUBLIC_KEY_LENGTH],
) -> Result<()> {
    let verifying_key = VerifyingKey::from_bytes(public_key)
        .map_err(|e| Error::SignatureVerification(format!("Invalid public key: {e}")))?;

    let sig = Signature::from_bytes(signature);

    verifying_key
        .verify(data, &sig)
        .map_err(|e| Error::SignatureVerification(format!("Verification failed: {e}")))
}

/// Information about the signing scheme.
#[derive(Debug, Clone)]
pub struct SigningInfo {
    /// Algorithm name.
    pub algorithm: &'static str,
    /// Signature size in bytes.
    pub signature_size: usize,
    /// Public key size in bytes.
    pub public_key_size: usize,
    /// Secret key size in bytes.
    pub secret_key_size: usize,
}

/// Get information about the signing scheme.
#[must_use]
pub fn signing_info() -> SigningInfo {
    SigningInfo {
        algorithm: "Ed25519",
        signature_size: SIGNATURE_LENGTH,
        public_key_size: PUBLIC_KEY_LENGTH,
        secret_key_size: SECRET_KEY_LENGTH,
    }
}

/// Hex encoding utilities for keys.
pub mod hex {
    /// Encode bytes to hex string.
    #[must_use]
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }

    /// Decode hex string to bytes.
    ///
    /// # Errors
    ///
    /// Returns error if hex is invalid.
    pub fn decode(hex: &str) -> std::result::Result<Vec<u8>, String> {
        if hex.len() % 2 != 0 {
            return Err("Hex string must have even length".to_string());
        }

        (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).map_err(|e| e.to_string()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generation() {
        let kp1 = Keypair::generate();
        let kp2 = Keypair::generate();

        // Different keypairs should have different keys
        assert_ne!(kp1.public_key_bytes(), kp2.public_key_bytes());
        assert_ne!(kp1.secret_key_bytes(), kp2.secret_key_bytes());
    }

    #[test]
    fn test_keypair_from_bytes() {
        let kp = Keypair::generate();
        let secret = kp.secret_key_bytes();

        let kp2 = Keypair::from_bytes(&secret).unwrap();

        assert_eq!(kp.public_key_bytes(), kp2.public_key_bytes());
    }

    #[test]
    fn test_sign_verify() {
        let kp = Keypair::generate();
        let data = b"Hello, World!";

        let signature = sign(data, &kp);
        assert!(verify(data, &signature).is_ok());
    }

    #[test]
    fn test_verify_wrong_data() {
        let kp = Keypair::generate();
        let data = b"Hello, World!";
        let wrong_data = b"Hello, Wrong!";

        let signature = sign(data, &kp);
        assert!(verify(wrong_data, &signature).is_err());
    }

    #[test]
    fn test_verify_wrong_key() {
        let kp1 = Keypair::generate();
        let kp2 = Keypair::generate();
        let data = b"Test data";

        let signature = sign(data, &kp1);
        let wrong_signature = SignatureData {
            signature: signature.signature,
            public_key: kp2.public_key_bytes(),
        };

        assert!(verify(data, &wrong_signature).is_err());
    }

    #[test]
    fn test_signature_data_serialization() {
        let kp = Keypair::generate();
        let data = b"Test data";

        let signature = sign(data, &kp);
        let bytes = signature.to_bytes();
        let recovered = SignatureData::from_bytes(&bytes).unwrap();

        assert_eq!(signature.signature, recovered.signature);
        assert_eq!(signature.public_key, recovered.public_key);
    }

    #[test]
    fn test_signature_data_invalid_length() {
        let result = SignatureData::from_bytes(&[0u8; 10]);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_with_public_key() {
        let kp = Keypair::generate();
        let data = b"Test data";

        let signature = sign(data, &kp);
        let result = verify_with_public_key(data, &signature.signature, &signature.public_key);

        assert!(result.is_ok());
    }

    #[test]
    fn test_hex_encode_decode() {
        let bytes = [0x12, 0x34, 0xab, 0xcd];
        let encoded = hex::encode(&bytes);
        assert_eq!(encoded, "1234abcd");

        let decoded = hex::decode(&encoded).unwrap();
        assert_eq!(decoded, bytes);
    }

    #[test]
    fn test_hex_decode_invalid() {
        assert!(hex::decode("123").is_err()); // Odd length
        assert!(hex::decode("gg").is_err()); // Invalid chars
    }

    #[test]
    fn test_signing_info() {
        let info = signing_info();
        assert_eq!(info.algorithm, "Ed25519");
        assert_eq!(info.signature_size, 64);
        assert_eq!(info.public_key_size, 32);
        assert_eq!(info.secret_key_size, 32);
    }

    #[test]
    fn test_empty_data() {
        let kp = Keypair::generate();
        let data = b"";

        let signature = sign(data, &kp);
        assert!(verify(data, &signature).is_ok());
    }

    #[test]
    fn test_large_data() {
        let kp = Keypair::generate();
        let data: Vec<u8> = (0..100_000).map(|i| (i % 256) as u8).collect();

        let signature = sign(&data, &kp);
        assert!(verify(&data, &signature).is_ok());
    }
}
