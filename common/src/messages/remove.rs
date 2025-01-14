use crate::messages::Message;
use crate::utils::files::{deserialize_path, serialize_path};
use std::io::Cursor;
use std::path::PathBuf;

// Message to remove a file or directory.
pub struct RemoveMessage {
    pub path: PathBuf,
}

impl Message for RemoveMessage {
    fn to_bytes(&self) -> Vec<u8> {
        serialize_path(&self.path)
    }

    fn from_bytes(data: &[u8]) -> Self
    where
        Self: Sized,
    {
        let mut cursor = Cursor::new(data);
        let path = deserialize_path(&mut cursor);

        Self { path }
    }
}
