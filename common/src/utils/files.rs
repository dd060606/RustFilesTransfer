use std::io::{Cursor, Read};
use std::path::PathBuf;

// Helper function to serialize path
pub fn serialize_path(path: &PathBuf) -> Vec<u8> {
    let path_str = path.to_str().unwrap_or("");
    let path_bytes = path_str.as_bytes();
    let path_length = path_bytes.len() as u32;
    let mut result = Vec::new();
    result.extend_from_slice(&path_length.to_le_bytes());
    result.extend_from_slice(path_bytes);
    result
}

// Helper function to deserialize path
pub fn deserialize_path(cursor: &mut Cursor<&[u8]>) -> PathBuf {
    let mut length_bytes = [0u8; 4];
    cursor.read_exact(&mut length_bytes).unwrap();
    let path_length = u32::from_le_bytes(length_bytes) as usize;
    let mut path_bytes = vec![0u8; path_length];
    cursor.read_exact(&mut path_bytes).unwrap();
    let path_str = String::from_utf8(path_bytes).unwrap();
    PathBuf::from(path_str)
}
