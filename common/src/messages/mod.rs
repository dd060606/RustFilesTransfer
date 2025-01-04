pub enum MessageType {
    PING,
    PONG
}
pub enum MessageError {
    InvalidFormat,
    UnknownMessageType,
    IncompleteData,
}

pub trait Message {
    // Converts the message into a byte array for sending over TCP.
    fn to_bytes(&self) -> Vec<u8>;
    // Parses a byte array into a message instance.
    // Returns an error if the data is invalid or incomplete.
    fn from_bytes(data: &[u8]) -> Result<Self, MessageError>
    where
        Self: Sized;
}