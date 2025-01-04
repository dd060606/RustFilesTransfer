use crate::messages::{Message};

pub struct PingMessage {
    pub message: String
}
impl Message for PingMessage {
    fn to_bytes(&self) -> Vec<u8> {
        // Convert the message to a byte array
        self.message.as_bytes().to_vec()
    }
    fn from_bytes(data: &[u8]) -> Self where Self: Sized {
        // Convert the byte array to a message
        let message = String::from_utf8(data.to_vec()).unwrap();
        Self {
            message
        }
    }

}


