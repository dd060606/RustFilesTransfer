use crate::messages::{Message};

pub struct ErrorResponse {
    pub error: String
}
impl Message for ErrorResponse {
    fn to_bytes(&self) -> Vec<u8> {
        // Convert the message to a byte array
        self.error.as_bytes().to_vec()
    }
    fn from_bytes(data: &[u8]) -> Self where Self: Sized {
        // Convert the byte array to a message
        let error = String::from_utf8(data.to_vec()).unwrap();
        Self {
            error
        }
    }

}


