use crate::messages::Message;
use std::io::{Cursor, Read};
use std::path::PathBuf;

// Message to request copying a file.
pub struct CopyFileMessage {
    pub source: PathBuf,
    pub output: PathBuf,
}

impl Message for CopyFileMessage {
    fn to_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::new();

        // Serialize `source`
        let source_str = self.source.to_str().unwrap_or("");
        let source_bytes = source_str.as_bytes();
        let source_length = source_bytes.len() as u32;
        buffer.extend_from_slice(&source_length.to_le_bytes());
        buffer.extend_from_slice(source_bytes);

        // Serialize `output`
        let output_str = self.output.to_str().unwrap_or("");
        let output_bytes = output_str.as_bytes();
        let output_length = output_bytes.len() as u32;
        buffer.extend_from_slice(&output_length.to_le_bytes());
        buffer.extend_from_slice(output_bytes);

        buffer
    }
    fn from_bytes(data: &[u8]) -> Self
    where
        Self: Sized,
    {
        let mut cursor = Cursor::new(data);

        // Deserialize `source`
        let mut length_bytes = [0u8; 4];
        cursor.read_exact(&mut length_bytes).unwrap();
        let source_length = u32::from_le_bytes(length_bytes) as usize;
        let mut source_bytes = vec![0u8; source_length];
        cursor.read_exact(&mut source_bytes).unwrap();
        let source_str = String::from_utf8(source_bytes).unwrap();
        let source_path = PathBuf::from(source_str);

        // Deserialize `output`
        cursor.read_exact(&mut length_bytes).unwrap();
        let output_length = u32::from_le_bytes(length_bytes) as usize;
        let mut output_bytes = vec![0u8; output_length];
        cursor.read_exact(&mut output_bytes).unwrap();
        let output_str = String::from_utf8(output_bytes).unwrap();
        let output_path = PathBuf::from(output_str);

        CopyFileMessage {
            source: source_path,
            output: output_path,
        }
    }
}
