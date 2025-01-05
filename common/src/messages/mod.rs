use crate::messages::list_files::{ListFilesMessage, ListFilesResponse};
use crate::messages::ping::{PingMessage};

pub mod ping;
pub mod list_files;

// Available messages for the server and client to communicate with each other.
pub enum Packet {
    Ping(PingMessage),
    ListFiles(ListFilesMessage),
    ListFilesResponse(ListFilesResponse),
}

pub trait Message {
    // Converts the message into a byte array for sending over TCP.
    fn to_bytes(&self) -> Vec<u8>;
    // Parses a byte array into a message instance.
    fn from_bytes(data: &[u8]) -> Self
    where
        Self: Sized;
}

impl Message for Packet {
    // Converts the message into a byte array for sending over TCP.
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        match self {
            Packet::Ping(packet) => {
                bytes.push(1); // Type identifier
                bytes.extend_from_slice(&packet.to_bytes());
            }
            Packet::ListFiles(packet) => {
                bytes.push(2); // Type identifier
                bytes.extend_from_slice(&packet.to_bytes());
            }
            Packet::ListFilesResponse(packet) => {
                bytes.push(3); // Type identifier
                bytes.extend_from_slice(&packet.to_bytes());
            }
        }

        bytes
    }

    // Parses a byte array into a message instance.
    fn from_bytes(bytes: &[u8]) -> Self {
        let msg_type = bytes[0]; // Read the type identifier
        match msg_type {
            1 => Packet::Ping(PingMessage::from_bytes(&bytes[1..])),
            2 => Packet::ListFiles(ListFilesMessage::from_bytes(&bytes[1..])),
            3 => Packet::ListFilesResponse(ListFilesResponse::from_bytes(&bytes[1..])),
            _ => panic!("Unknown message type"),
        }
    }
}