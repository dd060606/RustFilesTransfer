use crate::messages::Message;
use crate::utils::files::{deserialize_path, serialize_path};
use std::io::Cursor;
use std::path::PathBuf;

pub struct CopyFileMessage {
    pub source: PathBuf,
    pub output: PathBuf,
}

impl Message for CopyFileMessage {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Serialize both `source` and `output`
        bytes.extend(serialize_path(&self.source));
        bytes.extend(serialize_path(&self.output));

        bytes
    }

    fn from_bytes(data: &[u8]) -> Self {
        let mut cursor = Cursor::new(data);

        // Deserialize both `source` and `output`
        let source_path = deserialize_path(&mut cursor);
        let output_path = deserialize_path(&mut cursor);

        Self {
            source: source_path,
            output: output_path,
        }
    }
}
