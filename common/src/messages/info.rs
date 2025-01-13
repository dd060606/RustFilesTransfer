use crate::messages::Message;

//Message to request information about the client.
pub struct InfoMessage;

impl Message for InfoMessage {
    fn to_bytes(&self) -> Vec<u8> {
        Vec::new()
    }
    fn from_bytes(_data: &[u8]) -> Self {
        InfoMessage
    }
}
pub struct InfoResponse {
    pub computer_name: String,
    pub username: String,
}

impl Message for InfoResponse {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Serialize the computer_name and username as UTF-8 strings.
        let comp_name_bytes = self.computer_name.as_bytes();
        let user_name_bytes = self.username.as_bytes();

        // Add the lengths of the strings as u16 for decoding later.
        bytes.extend_from_slice(&(comp_name_bytes.len() as u16).to_be_bytes());
        bytes.extend_from_slice(comp_name_bytes);
        bytes.extend_from_slice(&(user_name_bytes.len() as u16).to_be_bytes());
        bytes.extend_from_slice(user_name_bytes);

        bytes
    }

    fn from_bytes(data: &[u8]) -> Self {
        // Read the length of the computer_name.
        let comp_name_len = u16::from_be_bytes([data[0], data[1]]) as usize;
        let comp_name_end = 2 + comp_name_len;
        let computer_name = String::from_utf8(data[2..comp_name_end].to_vec()).unwrap();

        // Read the length of the username.
        let user_name_len =
            u16::from_be_bytes([data[comp_name_end], data[comp_name_end + 1]]) as usize;
        let user_name_end = comp_name_end + 2 + user_name_len;
        let username = String::from_utf8(data[comp_name_end + 2..user_name_end].to_vec()).unwrap();

        InfoResponse {
            computer_name,
            username,
        }
    }
}
