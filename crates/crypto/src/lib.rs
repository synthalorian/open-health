//! open_health_crypto — AES-GCM-256 encryption + PBKDF2 key derivation.
//!
//! # Security model
//! - Master key is derived from a user passphrase via PBKDF2 (100k iterations, SHA-256).
//! - Each record is encrypted with a unique nonce (96-bit random).
//! - Key material is zeroed on drop.

#![forbid(unsafe_code)]

use ring::{aead, pbkdf2, rand as ring_rand};
use ring_rand::SecureRandom;
use std::num::NonZeroU32;

const PBKDF2_ITERATIONS: NonZeroU32 = NonZeroU32::new(100_000).expect("100k is non-zero");
pub const SALT_LEN: usize = 32;
pub const NONCE_LEN: usize = 12; // 96 bits for AES-GCM
pub const KEY_LEN: usize = 32; // AES-256

/// A passphrase-derived encryption key, zeroed on drop.
pub struct MasterKey([u8; KEY_LEN]);

impl Drop for MasterKey {
    fn drop(&mut self) {
        // Zero out the key material
        for byte in &mut self.0 {
            *byte = 0;
        }
    }
}

impl MasterKey {
    /// Derive a 256-bit key from a passphrase and salt.
    pub fn derive(passphrase: &str, salt: &[u8; SALT_LEN]) -> Self {
        let mut key = [0u8; KEY_LEN];
        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA256,
            PBKDF2_ITERATIONS,
            salt,
            passphrase.as_bytes(),
            &mut key,
        );
        MasterKey(key)
    }

    /// Verify a passphrase against a stored hash.
    pub fn verify(passphrase: &str, salt: &[u8; SALT_LEN], expected: &[u8]) -> bool {
        pbkdf2::verify(
            pbkdf2::PBKDF2_HMAC_SHA256,
            PBKDF2_ITERATIONS,
            salt,
            passphrase.as_bytes(),
            expected,
        )
        .is_ok()
    }

    /// Generate a random salt.
    pub fn generate_salt() -> [u8; SALT_LEN] {
        let rng = ring_rand::SystemRandom::new();
        let mut salt = [0u8; SALT_LEN];
        rng.fill(&mut salt).expect("RNG failed");
        salt
    }

    /// Encrypt plaintext. Returns `(nonce, ciphertext)`.
    pub fn encrypt(&self, plaintext: &[u8]) -> ([u8; NONCE_LEN], Vec<u8>) {
        let rng = ring_rand::SystemRandom::new();
        let mut nonce = [0u8; NONCE_LEN];
        rng.fill(&mut nonce).expect("RNG failed");

        let unbound_key = aead::UnboundKey::new(&aead::AES_256_GCM, &self.0)
            .expect("Valid key");
        let key = aead::LessSafeKey::new(unbound_key);

        let nonce_for_encrypt = aead::Nonce::assume_unique_for_key(nonce);
        let mut in_out = plaintext.to_vec();
        key.seal_in_place_append_tag(nonce_for_encrypt, aead::Aad::empty(), &mut in_out)
            .expect("Encryption failed");

        (nonce, in_out)
    }

    /// Decrypt ciphertext given the nonce.
    pub fn decrypt(&self, nonce: &[u8; NONCE_LEN], ciphertext: &[u8]) -> Result<Vec<u8>, ()> {
        let unbound_key =
            aead::UnboundKey::new(&aead::AES_256_GCM, &self.0).expect("Valid key");
        let key = aead::LessSafeKey::new(unbound_key);

        let nonce_for_decrypt = aead::Nonce::assume_unique_for_key(*nonce);
        let mut in_out = ciphertext.to_vec();
        let plaintext = key
            .open_in_place(nonce_for_decrypt, aead::Aad::empty(), &mut in_out)
            .map_err(|_| ())?;
        Ok(plaintext.to_vec())
    }
}

impl AsRef<[u8]> for MasterKey {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_and_encrypt_roundtrip() {
        let salt = MasterKey::generate_salt();
        let key = MasterKey::derive("test-passphrase-123", &salt);
        let data = b"Hello, open_health! This is sensitive health data.";

        let (nonce, ciphertext) = key.encrypt(data);
        assert_ne!(ciphertext, data, "ciphertext should differ from plaintext");

        let decrypted = key.decrypt(&nonce, &ciphertext).expect("Decryption failed");
        assert_eq!(decrypted, data);
    }

    #[test]
    fn test_wrong_key_fails() {
        let salt = [0xABu8; SALT_LEN];
        let key1 = MasterKey::derive("correct-passphrase", &salt);
        let key2 = MasterKey::derive("wrong-passphrase", &salt);

        let data = b"Sensitive data";
        let (nonce, ciphertext) = key1.encrypt(data);
        assert!(key2.decrypt(&nonce, &ciphertext).is_err());
    }

    #[test]
    fn test_passphrase_verify() {
        let salt = MasterKey::generate_salt();
        let key = MasterKey::derive("my-passphrase", &salt);
        // verify uses pbkdf2::verify on the derived key bytes
        let derived = key.0;
        assert!(MasterKey::verify("my-passphrase", &salt, &derived));
        assert!(!MasterKey::verify("wrong-passphrase", &salt, &derived));
    }
}
