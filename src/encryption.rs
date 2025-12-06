//! Dataset encryption utilities (AES-256-GCM).
//!
//! Provides secure encryption for datasets using:
//! - AES-256-GCM for authenticated encryption
//! - Argon2id for password-based key derivation
//!
//! Requires the `encryption` feature.

use crate::error::{Error, Result};
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{
    password_hash::{rand_core::RngCore, SaltString},
    Argon2,
};

/// Nonce size for AES-256-GCM (96 bits = 12 bytes).
const NONCE_SIZE: usize = 12;
/// Salt size for Argon2 key derivation.
const SALT_SIZE: usize = 16;
/// Key size for AES-256 (256 bits = 32 bytes).
const KEY_SIZE: usize = 32;

/// Encryption options.
#[derive(Debug, Clone)]
pub struct EncryptionOptions {
    /// Password for key derivation.
    password: String,
    /// Optional custom salt (for deterministic testing).
    salt: Option<[u8; SALT_SIZE]>,
}

impl EncryptionOptions {
    /// Create encryption options from a password.
    #[must_use]
    pub fn password(password: impl Into<String>) -> Self {
        Self {
            password: password.into(),
            salt: None,
        }
    }

    /// Set a custom salt (for testing/determinism).
    #[must_use]
    pub fn with_salt(mut self, salt: [u8; SALT_SIZE]) -> Self {
        self.salt = Some(salt);
        self
    }
}

/// Encrypted data structure.
#[derive(Debug, Clone)]
pub struct EncryptedData {
    /// Salt used for key derivation.
    pub salt: [u8; SALT_SIZE],
    /// Nonce used for encryption.
    pub nonce: [u8; NONCE_SIZE],
    /// Encrypted ciphertext.
    pub ciphertext: Vec<u8>,
}

impl EncryptedData {
    /// Serialize to bytes.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(SALT_SIZE + NONCE_SIZE + self.ciphertext.len());
        bytes.extend_from_slice(&self.salt);
        bytes.extend_from_slice(&self.nonce);
        bytes.extend_from_slice(&self.ciphertext);
        bytes
    }

    /// Deserialize from bytes.
    ///
    /// # Errors
    ///
    /// Returns `Error::Decryption` if data is too short.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < SALT_SIZE + NONCE_SIZE {
            return Err(Error::Decryption("Data too short".to_string()));
        }

        let mut salt = [0u8; SALT_SIZE];
        let mut nonce = [0u8; NONCE_SIZE];

        salt.copy_from_slice(&bytes[..SALT_SIZE]);
        nonce.copy_from_slice(&bytes[SALT_SIZE..SALT_SIZE + NONCE_SIZE]);
        let ciphertext = bytes[SALT_SIZE + NONCE_SIZE..].to_vec();

        Ok(Self {
            salt,
            nonce,
            ciphertext,
        })
    }
}

/// Encrypt data using AES-256-GCM.
///
/// # Arguments
///
/// * `plaintext` - Data to encrypt
/// * `options` - Encryption options (password, etc.)
///
/// # Errors
///
/// Returns `Error::Encryption` if encryption fails.
pub fn encrypt(plaintext: &[u8], options: &EncryptionOptions) -> Result<EncryptedData> {
    // Generate or use provided salt
    let salt = options.salt.unwrap_or_else(|| {
        let mut salt = [0u8; SALT_SIZE];
        OsRng.fill_bytes(&mut salt);
        salt
    });

    // Derive key using Argon2id
    let key = derive_key(&options.password, &salt)?;

    // Generate nonce
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);

    // Encrypt
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| Error::Encryption(e.to_string()))?;

    Ok(EncryptedData {
        salt,
        nonce: nonce_bytes,
        ciphertext,
    })
}

/// Decrypt data using AES-256-GCM.
///
/// # Arguments
///
/// * `encrypted` - Encrypted data structure
/// * `password` - Password for key derivation
///
/// # Errors
///
/// Returns `Error::Decryption` if decryption fails.
pub fn decrypt(encrypted: &EncryptedData, password: &str) -> Result<Vec<u8>> {
    // Derive key using same salt
    let key = derive_key(password, &encrypted.salt)?;

    // Decrypt
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&encrypted.nonce);

    cipher
        .decrypt(nonce, encrypted.ciphertext.as_ref())
        .map_err(|e| Error::Decryption(e.to_string()))
}

/// Encrypt bytes directly to bytes (convenience wrapper).
///
/// # Errors
///
/// Returns `Error::Encryption` if encryption fails.
pub fn encrypt_bytes(plaintext: &[u8], password: &str) -> Result<Vec<u8>> {
    let options = EncryptionOptions::password(password);
    let encrypted = encrypt(plaintext, &options)?;
    Ok(encrypted.to_bytes())
}

/// Decrypt bytes directly from bytes (convenience wrapper).
///
/// # Errors
///
/// Returns `Error::Decryption` if decryption fails.
pub fn decrypt_bytes(ciphertext: &[u8], password: &str) -> Result<Vec<u8>> {
    let encrypted = EncryptedData::from_bytes(ciphertext)?;
    decrypt(&encrypted, password)
}

/// Derive a key from password using Argon2id.
fn derive_key(password: &str, salt: &[u8; SALT_SIZE]) -> Result<Key<Aes256Gcm>> {
    let argon2 = Argon2::default();

    let mut key_bytes = [0u8; KEY_SIZE];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut key_bytes)
        .map_err(|e| Error::Encryption(format!("Key derivation failed: {e}")))?;

    Ok(*Key::<Aes256Gcm>::from_slice(&key_bytes))
}

/// Information about encryption parameters.
#[derive(Debug, Clone)]
pub struct EncryptionInfo {
    /// Cipher algorithm.
    pub cipher: &'static str,
    /// Key derivation function.
    pub kdf: &'static str,
    /// Key size in bits.
    pub key_size_bits: usize,
    /// Nonce size in bytes.
    pub nonce_size: usize,
}

/// Get information about the encryption scheme.
#[must_use]
pub fn encryption_info() -> EncryptionInfo {
    EncryptionInfo {
        cipher: "AES-256-GCM",
        kdf: "Argon2id",
        key_size_bits: 256,
        nonce_size: NONCE_SIZE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let plaintext = b"Hello, World! This is a test message.";
        let password = "secure-password-123";

        let encrypted = encrypt(plaintext, &EncryptionOptions::password(password)).unwrap();
        let decrypted = decrypt(&encrypted, password).unwrap();

        assert_eq!(plaintext.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn test_encrypt_bytes_roundtrip() {
        let plaintext = b"Test data for encryption";
        let password = "my-password";

        let ciphertext = encrypt_bytes(plaintext, password).unwrap();
        let decrypted = decrypt_bytes(&ciphertext, password).unwrap();

        assert_eq!(plaintext.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn test_wrong_password() {
        let plaintext = b"Secret message";
        let password = "correct-password";
        let wrong_password = "wrong-password";

        let encrypted = encrypt(plaintext, &EncryptionOptions::password(password)).unwrap();
        let result = decrypt(&encrypted, wrong_password);

        assert!(result.is_err());
    }

    #[test]
    fn test_deterministic_with_salt() {
        let plaintext = b"Test data";
        let password = "password";
        let salt = [1u8; SALT_SIZE];

        let options = EncryptionOptions::password(password).with_salt(salt);

        let encrypted1 = encrypt(plaintext, &options).unwrap();
        let encrypted2 = encrypt(plaintext, &options).unwrap();

        // Same salt should produce same key, but different nonces
        // So ciphertexts will differ (due to random nonce)
        // But both should decrypt correctly
        let decrypted1 = decrypt(&encrypted1, password).unwrap();
        let decrypted2 = decrypt(&encrypted2, password).unwrap();

        assert_eq!(decrypted1, decrypted2);
        assert_eq!(decrypted1.as_slice(), plaintext);
    }

    #[test]
    fn test_encrypted_data_serialization() {
        let encrypted = EncryptedData {
            salt: [1u8; SALT_SIZE],
            nonce: [2u8; NONCE_SIZE],
            ciphertext: vec![3, 4, 5, 6],
        };

        let bytes = encrypted.to_bytes();
        let recovered = EncryptedData::from_bytes(&bytes).unwrap();

        assert_eq!(encrypted.salt, recovered.salt);
        assert_eq!(encrypted.nonce, recovered.nonce);
        assert_eq!(encrypted.ciphertext, recovered.ciphertext);
    }

    #[test]
    fn test_encrypted_data_too_short() {
        let result = EncryptedData::from_bytes(&[0u8; 10]);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_plaintext() {
        let plaintext = b"";
        let password = "password";

        let encrypted = encrypt(plaintext, &EncryptionOptions::password(password)).unwrap();
        let decrypted = decrypt(&encrypted, password).unwrap();

        assert!(decrypted.is_empty());
    }

    #[test]
    fn test_large_plaintext() {
        let plaintext: Vec<u8> = (0..100_000).map(|i| (i % 256) as u8).collect();
        let password = "password";

        let encrypted = encrypt(&plaintext, &EncryptionOptions::password(password)).unwrap();
        let decrypted = decrypt(&encrypted, password).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_encryption_info() {
        let info = encryption_info();
        assert_eq!(info.cipher, "AES-256-GCM");
        assert_eq!(info.kdf, "Argon2id");
        assert_eq!(info.key_size_bits, 256);
    }
}
