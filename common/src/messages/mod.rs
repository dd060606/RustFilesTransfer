use crate::messages::ping::{PingMessage};

pub mod ping;
// Available messages for the server and client to communicate with each other.
pub enum BasePacket {
    Ping(PingMessage),
}

pub trait Message {
    // Converts the message into a byte array for sending over TCP.
    fn to_bytes(&self) -> Vec<u8>;
    // Parses a byte array into a message instance.
    fn from_bytes(data: &[u8]) -> Self
    where
        Self: Sized;
}

impl Message for BasePacket {
    // Converts the message into a byte array for sending over TCP.
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        match self {
            BasePacket::Ping(msg) => {
                bytes.push(1); // Type identifier
                bytes.extend_from_slice(&msg.to_bytes());
            }
        }

        bytes
    }

    // Parses a byte array into a message instance.
    fn from_bytes(bytes: &[u8]) -> Self {
        let msg_type = bytes[0]; // Read the type identifier
        match msg_type {
            1 => BasePacket::Ping(PingMessage::from_bytes(&bytes[1..])),
            _ => panic!("Unknown message type"),
        }
    }
}