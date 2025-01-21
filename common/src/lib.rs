pub mod messages;

pub mod utils;

#[cfg(test)]
mod tests {
    use crate::messages::ping::PingMessage;
    use crate::messages::{Message, Packet};
    use crate::utils::encryption::{decrypt_packet, encrypt_packet, generate_keypair, Encryptor};

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
        let server_encryptor = Encryptor::new(server_keypair, client_pub_bytes);

        let packet_bytes = Packet::Ping(PingMessage {
            message: "Hello, world!".to_string(),
        })
        .to_bytes();

        let encrypted_data = encrypt_packet(&packet_bytes, &server_encryptor);
        // Client side
        let client_encryptor = Encryptor::new(client_keypair, server_pub_bytes);

        let mut encrypted_response = vec![];
        let decrypted_packet_bytes = decrypt_packet(&encrypted_data, &client_encryptor);
        if let Packet::Ping(msg) = Packet::from_bytes(&decrypted_packet_bytes) {
            println!("Received from server: {}", msg.message);
            assert_eq!(msg.message, "Hello, world!");
            // Echo back the received message
            encrypted_response = encrypt_packet(&Packet::Ping(msg).to_bytes(), &client_encryptor);
        }

        //Server side
        let decrypted_response_bytes = decrypt_packet(&encrypted_response, &server_encryptor);
        if let Packet::Ping(msg) = Packet::from_bytes(&decrypted_response_bytes) {
            println!("Received from client: {}", msg.message);
            assert_eq!(msg.message, "Hello, world!");
        }
    }
}
