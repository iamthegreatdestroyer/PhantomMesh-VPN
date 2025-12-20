//! Crypto Manager Module
//! =====================
//! Quantum-resistant cryptographic operations.
//!
//! Copyright © 2025 Stephen Bilodeau. All rights reserved.
//! Licensed under GPL-3.0 with proprietary agent clauses.

use pqcrypto_dilithium::*;
use pqcrypto_kyber::*;
use pqcrypto_traits::kem::{Ciphertext, SharedSecret};
use pqcrypto_traits::sign::SignedMessage;
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey};
use ring::rand::SecureRandom;

/// Quantum-resistant cryptographic manager
pub struct CryptoManager {
    rng: ring::rand::SystemRandom,
}

impl CryptoManager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Self {
            rng: ring::rand::SystemRandom::new(),
        })
    }

    /// Generate Kyber keypair for quantum-resistant key exchange
    pub fn generate_kyber_keypair(
        &self,
    ) -> Result<(kyber768::PublicKey, kyber768::SecretKey), Box<dyn std::error::Error + Send + Sync>>
    {
        let (pk, sk) = kyber768::keypair();
        Ok((pk, sk))
    }

    /// Perform Kyber key exchange
    pub fn kyber_encapsulate(
        &self,
        pk: &kyber768::PublicKey,
    ) -> Result<(Vec<u8>, Vec<u8>), Box<dyn std::error::Error + Send + Sync>> {
        let (ct, ss) = kyber768::encapsulate(pk);
        Ok((ct.as_bytes().to_vec(), ss.as_bytes().to_vec()))
    }

    pub fn kyber_decapsulate(
        &self,
        ct: &[u8],
        sk: &kyber768::SecretKey,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let ct = kyber768::Ciphertext::from_bytes(ct)?;
        let ss = kyber768::decapsulate(&ct, sk);
        Ok(ss.as_bytes().to_vec())
    }

    /// Generate Dilithium keypair for signatures
    pub fn generate_dilithium_keypair(
        &self,
    ) -> Result<
        (dilithium2::PublicKey, dilithium2::SecretKey),
        Box<dyn std::error::Error + Send + Sync>,
    > {
        let (pk, sk) = dilithium2::keypair();
        Ok((pk, sk))
    }

    /// Sign with Dilithium
    pub fn dilithium_sign(
        &self,
        message: &[u8],
        sk: &dilithium2::SecretKey,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let signed = dilithium2::sign(message, sk);
        Ok(signed.as_bytes().to_vec())
    }

    /// Verify Dilithium signature
    pub fn dilithium_verify(
        &self,
        message: &[u8],
        sig: &[u8],
        pk: &dilithium2::PublicKey,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let signed = dilithium2::SignedMessage::from_bytes(sig)?;
        match dilithium2::open(&signed, pk) {
            Ok(opened_message) => Ok(opened_message == message),
            Err(_) => Ok(false),
        }
    }

    /// ChaCha20-Poly1305 encryption
    pub fn encrypt_chacha(
        &self,
        plaintext: &[u8],
        key: &[u8],
        nonce: &[u8],
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        use ring::aead::CHACHA20_POLY1305;
        let key = UnboundKey::new(&CHACHA20_POLY1305, key).map_err(|_| "Invalid key")?;
        let key = LessSafeKey::new(key);
        let nonce = Nonce::try_assume_unique_for_key(nonce).map_err(|_| "Invalid nonce")?;
        let mut in_out = plaintext.to_vec();
        key.seal_in_place_append_tag(nonce, Aad::empty(), &mut in_out)
            .map_err(|_| "Encryption failed")?;
        Ok(in_out)
    }

    /// ChaCha20-Poly1305 decryption
    pub fn decrypt_chacha(
        &self,
        ciphertext: &[u8],
        key: &[u8],
        nonce: &[u8],
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        use ring::aead::CHACHA20_POLY1305;
        let key = UnboundKey::new(&CHACHA20_POLY1305, key).map_err(|_| "Invalid key")?;
        let key = LessSafeKey::new(key);
        let nonce = Nonce::try_assume_unique_for_key(nonce).map_err(|_| "Invalid nonce")?;
        let mut in_out = ciphertext.to_vec();
        let plaintext = key
            .open_in_place(nonce, Aad::empty(), &mut in_out)
            .map_err(|_| "Decryption failed")?;
        Ok(plaintext.to_vec())
    }

    /// Generate random bytes
    pub fn random_bytes(
        &self,
        len: usize,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = vec![0u8; len];
        self.rng
            .fill(&mut bytes)
            .map_err(|_| "Random generation failed")?;
        Ok(bytes)
    }

    // Legacy methods for compatibility
    pub fn generate_keypair(&self) -> Result<([u8; 32], [u8; 32]), Box<dyn std::error::Error>> {
        let mut public = [0u8; 32];
        let mut secret = [0u8; 32];
        self.rng
            .fill(&mut public)
            .map_err(|_| "Random generation failed")?;
        self.rng
            .fill(&mut secret)
            .map_err(|_| "Random generation failed")?;
        Ok((public, secret))
    }

    pub fn encrypt(
        &self,
        plaintext: &[u8],
        key: &[u8],
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let nonce = self.random_bytes(12)?;
        self.encrypt_chacha(plaintext, key, &nonce)
    }

    pub fn decrypt(
        &self,
        ciphertext: &[u8],
        key: &[u8],
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let nonce = &ciphertext[ciphertext.len() - 12..];
        let ciphertext = &ciphertext[..ciphertext.len() - 12];
        self.decrypt_chacha(ciphertext, key, nonce)
    }
}
