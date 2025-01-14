use crate::messages::Message;

// Message to confirm that a task was successful.
pub struct ConfirmResponse;
impl Message for ConfirmResponse {
    fn to_bytes(&self) -> Vec<u8> {
        // ConfirmResponse is an empty message, so it doesn't need to serialize any data
        Vec::new()
    }
    fn from_bytes(_data: &[u8]) -> Self
    where
        Self: Sized,
    {
        // ConfirmResponse is an empty message, so it doesn't need to deserialize any data
        Self
    }
}
// Message to respond with an error message.
pub struct ErrorResponse {
    pub error: String,
}
impl Message for ErrorResponse {
    fn to_bytes(&self) -> Vec<u8> {
        // Convert the message to a byte array
        self.error.as_bytes().to_vec()
    }
    fn from_bytes(data: &[u8]) -> Self
    where
        Self: Sized,
    {
        // Convert the byte array to a message
        let error = String::from_utf8(data.to_vec()).unwrap();
        Self { error }
    }
}
