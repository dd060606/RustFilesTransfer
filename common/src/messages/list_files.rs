use crate::messages::{Message};

pub struct ListFilesMessage {
    pub path: String
}
impl Message for ListFilesMessage {
    fn to_bytes(&self) -> Vec<u8> {
        // Convert the message to a byte array
        self.path.as_bytes().to_vec()
    }
    fn from_bytes(data: &[u8]) -> Self where Self: Sized {
        // Convert the byte array to a message
        let path = String::from_utf8(data.to_vec()).unwrap();
        Self {
            path
        }
    }
}

pub struct ListFilesResponse {
    pub files: Vec<String>
}
impl Message for ListFilesResponse {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        // Convert file names to byte arrays and separate them with a null byte
        for file in &self.files {
            bytes.extend_from_slice(file.as_bytes());
            bytes.push(0);
        }
        bytes
    }
    fn from_bytes(data: &[u8]) -> Self where Self: Sized {
        // Convert the byte array to a vector of file names
        let mut files = Vec::new();
        let mut file = Vec::new();
        for byte in data {
            // If the byte is a null byte, then the file name is complete
            if *byte == 0 {
                files.push(String::from_utf8(file).unwrap());
                file = Vec::new();
            } else {
                file.push(*byte);
            }
        }
        Self {
            files
        }
    }
}
