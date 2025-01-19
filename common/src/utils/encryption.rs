use aes_gcm::{aead::{Aead, Nonce}, Aes256Gcm, KeyInit};
use rand::RngCore;
use rand::rngs::OsRng;
use x25519_dalek::{EphemeralSecret, PublicKey, SharedSecret};

// Shared encryption utilities
pub struct Encryptor {
    cipher: Aes256Gcm,
}

impl Encryptor {
    pub fn new(shared_secret: SharedSecret) -> Self {
        // Generate cipher from shared secret
        let cipher = Aes256Gcm::new_from_slice(&shared_secret.to_bytes())
            .expect("Valid key length");
        Self { cipher }
    }

    pub fn encrypt(&self, data: &[u8]) -> Vec<u8> {
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::<Aes256Gcm>::from_slice(&nonce_bytes);

        let ciphertext = self.cipher
            .encrypt(nonce, data)
            .expect("Encryption failed");

        // Combine nonce and ciphertext
        let mut result = Vec::with_capacity(nonce_bytes.len() + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);
        result
    }

    pub fn decrypt(&self, data: &[u8]) -> Option<Vec<u8>> {
        if data.len() < 12 {
            return None;
        }

        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::<Aes256Gcm>::from_slice(nonce_bytes);

        self.cipher
            .decrypt(nonce, ciphertext)
            .ok()
    }
}

pub struct Keypair {
    pub private: EphemeralSecret,
    pub public: PublicKey,
}

// Generate a private key and corresponding public key
pub fn generate_keypair() -> Keypair {
    let private = EphemeralSecret::random_from_rng(OsRng);
    let public = PublicKey::from(&private);
    Keypair { private, public }
}