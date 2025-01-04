pub mod messages;


#[cfg(test)]
mod tests {
    use crate::messages::{BasePacket, Message};
    use crate::messages::ping::PingMessage;

    #[test]
    fn it_works() {
        let ping_msg = PingMessage {
            message: "Hello, world!".to_string(),
        };

        let ping_packet = BasePacket::Ping(ping_msg);
        let encoded = ping_packet.to_bytes();
        let decoded_msg = BasePacket::from_bytes(&encoded);
        //if let BasePacket::Ping(msg) = decoded_msg {
        //    println!("Message: {}", msg.message);
        //
        let BasePacket::Ping(msg) = decoded_msg;
        assert_eq!(msg.message, "Hello, world!");
    }
}
