pub mod messages;
mod utils;

#[cfg(test)]
mod tests {
    use crate::messages::ping::PingMessage;
    use crate::messages::{Message, Packet};

    #[test]
    fn it_works() {
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
}
