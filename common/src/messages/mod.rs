use crate::messages::copy::CopyFileMessage;
use crate::messages::info::{InfoMessage, InfoResponse};
use crate::messages::list_files::{ListFilesMessage, ListFilesResponse};
use crate::messages::ping::PingMessage;
use crate::messages::response::{ConfirmResponse, ErrorResponse};

// The messages module contains all the messages that the server and client can send to each other.
pub mod copy;
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
    ConfirmResponse(ConfirmResponse),
    Info(InfoMessage),
    InfoResponse(InfoResponse),
    CopyFile(CopyFileMessage),
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
    fn to_bytes(&self) -> Vec<u8> {
        // Serialize a packet into a byte array.
        fn serialize_packet(type_id: u8, packet_bytes: &[u8]) -> Vec<u8> {
            let mut bytes = Vec::new();
            let size = packet_bytes.len() as u32; // Packet size
            bytes.push(type_id); // Type identifier
            bytes.extend_from_slice(&size.to_be_bytes()); // Packet size (4 bytes)
            bytes.extend_from_slice(packet_bytes); // Packet data
            bytes
        }

        match self {
            Packet::Ping(packet) => serialize_packet(1, &packet.to_bytes()),
            Packet::ListFiles(packet) => serialize_packet(2, &packet.to_bytes()),
            Packet::ListFilesResponse(packet) => serialize_packet(3, &packet.to_bytes()),
            Packet::ErrorResponse(packet) => serialize_packet(4, &packet.to_bytes()),
            Packet::ConfirmResponse(packet) => serialize_packet(5, &packet.to_bytes()),
            Packet::Info(packet) => serialize_packet(6, &packet.to_bytes()),
            Packet::InfoResponse(packet) => serialize_packet(7, &packet.to_bytes()),
            Packet::CopyFile(packet) => serialize_packet(8, &packet.to_bytes()),
        }
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let msg_type = bytes[0]; // Read type identifier
        let size_bytes = &bytes[1..5]; // Read size (4 bytes)
        let size = u32::from_be_bytes(size_bytes.try_into().unwrap()) as usize;

        match msg_type {
            1 => Packet::Ping(PingMessage::from_bytes(&bytes[5..5 + size])),
            2 => Packet::ListFiles(ListFilesMessage::from_bytes(&bytes[5..5 + size])),
            3 => Packet::ListFilesResponse(ListFilesResponse::from_bytes(&bytes[5..5 + size])),
            4 => Packet::ErrorResponse(ErrorResponse::from_bytes(&bytes[5..5 + size])),
            5 => Packet::ConfirmResponse(ConfirmResponse::from_bytes(&bytes[5..5 + size])),
            6 => Packet::Info(InfoMessage::from_bytes(&bytes[5..5 + size])),
            7 => Packet::InfoResponse(InfoResponse::from_bytes(&bytes[5..5 + size])),
            8 => Packet::CopyFile(CopyFileMessage::from_bytes(&bytes[5..5 + size])),
            _ => {
                // The packet type is unknown
                let error = ErrorResponse {
                    error: "Error while decoding packet".to_string(),
                };
                Packet::ErrorResponse(error)
            }
        }
    }
}
