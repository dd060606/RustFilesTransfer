use crate::messages::Message;
use crate::utils::files::{deserialize_path, serialize_path};
use std::io::Read;
use std::path::PathBuf;

// Message to prepare a file for uploading/downloading
pub struct PrepareFileMessage {
    pub output: PathBuf,
    pub size: u64,
}

impl Message for PrepareFileMessage {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Serialize `output` and `size`
        bytes.extend(serialize_path(&self.output));
        bytes.extend(&self.size.to_le_bytes());

        bytes
    }

    fn from_bytes(data: &[u8]) -> Self
    where
        Self: Sized,
    {
        let mut cursor = std::io::Cursor::new(data);

        // Deserialize `output` and `size`
        let output = deserialize_path(&mut cursor);
        let mut size_bytes = [0; 8];
        cursor.read_exact(&mut size_bytes).unwrap();
        let size = u64::from_le_bytes(size_bytes);

        Self { output, size }
    }
}
