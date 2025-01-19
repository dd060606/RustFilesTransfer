pub mod messages;

pub mod utils;

#[cfg(test)]
mod tests {
    use x25519_dalek::PublicKey;

    use crate::messages::{Message, Packet};
    use crate::messages::ping::PingMessage;
    use crate::utils::encryption::{Encryptor, generate_keypair};

    #[test]
    fn test_packet() {
        let ping_msg = PingMessage {
            message: "Hello, world!".to_string(),
        };

        let ping_packet = Packet::Ping(ping_msg);
        let encoded = ping_packet.to_bytes();
        let decoded_msg = Packet::from_bytes(&encoded);
        if let Packet::Ping(msg) = decoded_msg {
            println!("Message: {}", msg.message);
            assert_eq!(msg.message, "Hello, world!");
        }
    }

    #[test]
    fn test_encryption() {
        //Server side
        let server_keypair = generate_keypair();
        let server_pub_bytes = server_keypair.public.to_bytes();

        //Client side
        let client_keypair = generate_keypair();
        let client_pub_bytes = client_keypair.public.to_bytes();

        // Server side
        let server_shared_secret = server_keypair.private.diffie_hellman(&PublicKey::from(client_pub_bytes));
        let server_encryptor = Encryptor::new(server_shared_secret);
        let encrypted_data = server_encryptor.encrypt(b"Hello, world!");

        // Client side
        let client_shared_secret = client_keypair.private.diffie_hellman(&PublicKey::from(server_pub_bytes));
        let client_encryptor = Encryptor::new(client_shared_secret);

        let mut encrypted_response = vec![];
        if let Some(decrypted_data) = client_encryptor.decrypt(&encrypted_data) {
            println!("Received from server: {:?}", String::from_utf8_lossy(&decrypted_data));
            assert_eq!(decrypted_data, b"Hello, world!");
            // Echo back the received message
            encrypted_response = client_encryptor.encrypt(&decrypted_data);
        }
        //Server side
        if let Some(decrypted_response) = server_encryptor.decrypt(&encrypted_response) {
            println!("Received from client: {:?}", String::from_utf8_lossy(&decrypted_response));
            assert_eq!(decrypted_response, b"Hello, world!");
        }
    }
}
