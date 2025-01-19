use crate::messages::Message;

pub struct ElevateMessage;

impl Message for ElevateMessage {
    fn to_bytes(&self) -> Vec<u8> {
        Vec::new()
    }
    fn from_bytes(_data: &[u8]) -> Self {
        ElevateMessage
    }
}