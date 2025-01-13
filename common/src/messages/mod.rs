use crate::messages::info::{InfoMessage, InfoResponse};
use crate::messages::list_files::{ListFilesMessage, ListFilesResponse};
use crate::messages::ping::PingMessage;
use crate::messages::response::ErrorResponse;

pub mod info;
pub mod list_files;
pub mod ping;
pub mod response;

// Available messages for the server and client to communicate with each other.
pub enum Packet {
    Ping(PingMessage),
    ListFiles(ListFilesMessage),
    ListFilesResponse(ListFilesResponse),
    ErrorResponse(ErrorResponse),
    Info(InfoMessage),
    InfoResponse(InfoResponse),
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
            Packet::ErrorResponse(packet) => {
                bytes.push(4); // Type identifier
                bytes.extend_from_slice(&packet.to_bytes());
            }
            Packet::Info(packet) => {
                bytes.push(5); // Type identifier
                bytes.extend_from_slice(&packet.to_bytes());
            }
            Packet::InfoResponse(packet) => {
                bytes.push(6); // Type identifier
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
            4 => Packet::ErrorResponse(ErrorResponse::from_bytes(&bytes[1..])),
            5 => Packet::Info(InfoMessage::from_bytes(&bytes[1..])),
            6 => Packet::InfoResponse(InfoResponse::from_bytes(&bytes[1..])),
            _ => {
                //If the packet is invalid, return a ErrorResponse packet
                let error = ErrorResponse {
                    error: "Error while decoding packet".to_string(),
                };
                Packet::ErrorResponse(error)
            }
        }
    }
}
