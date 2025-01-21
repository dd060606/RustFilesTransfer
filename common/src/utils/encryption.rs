use aes_gcm::{
    aead::{Aead, Nonce},
    Aes256Gcm, KeyInit,
};
use rand::rngs::OsRng;
use rand::RngCore;
use x25519_dalek::{EphemeralSecret, PublicKey};

// Shared encryption utilities
pub struct Encryptor {
    cipher: Aes256Gcm,
}

impl Encryptor {
    pub fn new(keypair: Keypair, received_public_key: [u8; 32]) -> Self {
        // Generate shared secret
        let shared_secret = keypair
            .private
            .diffie_hellman(&PublicKey::from(received_public_key));
        // Generate cipher from shared secret
        let cipher =
            Aes256Gcm::new_from_slice(&shared_secret.to_bytes()).expect("Valid key length");
        Self { cipher }
    }

    pub fn encrypt(&self, data: &[u8]) -> Vec<u8> {
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::<Aes256Gcm>::from_slice(&nonce_bytes);

        let ciphertext = self.cipher.encrypt(nonce, data).expect("Encryption failed");

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

        self.cipher.decrypt(nonce, ciphertext).ok()
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

// Encrypts a packet while preserving its structure
pub fn encrypt_packet(packet: &[u8], encryptor: &Encryptor) -> Vec<u8> {
    // Verify packet length
    if packet.len() < 5 {
        return vec![];
    }
    // Extract packet components
    let packet_type = packet[0];
    let content_size = u32::from_be_bytes([packet[1], packet[2], packet[3], packet[4]]);

    // Verify packet length matches declared size
    if packet.len() != (5 + content_size as usize) {
        return vec![];
    }

    // Get content to encrypt
    let content = &packet[5..];

    // Encrypt the content
    let encrypted_content = encryptor.encrypt(content);
    let encrypted_size = encrypted_content.len() as u32;

    // Build new packet
    let mut result = Vec::with_capacity(5 + encrypted_content.len());
    result.push(packet_type);
    result.extend_from_slice(&encrypted_size.to_be_bytes());
    result.extend_from_slice(&encrypted_content);

    result
}

// Decrypts a packet while preserving its structure
pub fn decrypt_packet(packet: &[u8], encryptor: &Encryptor) -> Vec<u8> {
    // Verify packet length
    if packet.len() < 5 {
        return vec![];
    }
    // Extract packet components
    let packet_type = packet[0];
    let encrypted_size = u32::from_be_bytes([packet[1], packet[2], packet[3], packet[4]]);

    // Verify packet length matches declared size
    if packet.len() != (5 + encrypted_size as usize) {
        return vec![];
    }

    // Get encrypted content
    let encrypted_content = &packet[5..];

    // Decrypt the content
    let decrypted_content = encryptor.decrypt(encrypted_content).unwrap_or(vec![]);
    let decrypted_size = decrypted_content.len() as u32;

    // Build decrypted packet
    let mut result = Vec::with_capacity(5 + decrypted_content.len());
    result.push(packet_type);
    result.extend_from_slice(&decrypted_size.to_be_bytes());
    result.extend_from_slice(&decrypted_content);

    result
}
