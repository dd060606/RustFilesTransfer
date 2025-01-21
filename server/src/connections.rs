use std::collections::HashMap;

use common::messages::{Message, Packet};
use common::utils::encryption::{decrypt_packet, encrypt_packet, Encryptor};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[derive(Clone)]
pub struct ClientInfo {
    pub username: String,
    pub computer_name: String,
}

pub struct Connections {
    //Map of connections (ID, TcpStream)
    pub clients: HashMap<u16, TcpStream>,
    //Map of encryption keys (ID, Encryptor)
    encryption: HashMap<u16, Encryptor>,
    pub clients_info: HashMap<u16, ClientInfo>,
    //ID of the current connection
    pub current_client: u16,
}

impl Connections {
    pub fn new() -> Connections {
        let clients = HashMap::new();
        let encryption = HashMap::new();
        let clients_info = HashMap::new();
        Connections {
            clients,
            encryption,
            clients_info,
            current_client: 1,
        }
    }
    pub fn add_connection(&mut self, id: u16, stream: TcpStream, encryptor: Encryptor) {
        self.clients.insert(id, stream);
        self.encryption.insert(id, encryptor);
    }
    pub fn add_info(&mut self, id: u16, info: ClientInfo) {
        self.clients_info.insert(id, info);
    }
    pub fn get_connection(&mut self, id: u16) -> Option<&mut TcpStream> {
        self.clients.get_mut(&id)
    }

    pub fn remove_connection(&mut self, id: u16) {
        self.clients.remove(&id);
        self.clients_info.remove(&id);
        self.encryption.remove(&id);
    }

    //Check if the connection exists
    pub fn exists(&self, id: u16) -> bool {
        self.clients.contains_key(&id)
    }
    //Change current client ID
    pub fn set_current_client(&mut self, id: u16) {
        self.current_client = id;
    }

    //Get client info
    pub fn get_client_info(&self, id: u16) -> ClientInfo {
        //Return the client info if it exists
        self.clients_info.get(&id).cloned().unwrap_or(ClientInfo {
            username: String::from("Unknown"),
            computer_name: String::from("Unknown"),
        })
    }

    //Send a message to the selected client
    pub async fn send_message(&mut self, message: &Packet) -> Result<Packet, String> {
        let client_id = self.current_client;

        // Get both connection and encryptor with a single borrow
        let (stream, encryptor) = {
            if !self.clients.contains_key(&client_id) {
                return Err(format!(
                    "Client {} not found, please change current client using 'select '",
                    client_id
                ));
            }
            if !self.encryption.contains_key(&client_id) {
                return Err("Encryptor not found!".to_string());
            }

            // Now we know both exist, we can get references
            let stream = self.clients.get_mut(&client_id).unwrap();
            let encryptor = self.encryption.get(&client_id).unwrap();
            (stream, encryptor)
        };

        // Encrypt the packet
        let encrypted_packet = encrypt_packet(&message.to_bytes(), encryptor);
        // Send the message
        if let Err(e) = stream.write_all(&encrypted_packet).await {
            self.remove_connection(client_id);
            return Err(e.to_string());
        }

        // Read the response
        let mut buffer = [0; 1024];
        let mut total_data = Vec::new();

        loop {
            match stream.read(&mut buffer).await {
                Ok(size) => {
                    if size == 0 {
                        if total_data.is_empty() {
                            self.remove_connection(client_id);
                            return Err("Connection closed unexpectedly!".to_string());
                        } else {
                            break;
                        }
                    } else {
                        total_data.extend_from_slice(&buffer[..size]);
                        if size < 1024 {
                            break;
                        }
                    }
                }
                Err(e) => {
                    self.remove_connection(client_id);
                    return Err(format!("Failed to read from client: {}", e));
                }
            }
        }

        // Decrypt the response
        let decrypted_response = decrypt_packet(&total_data, encryptor);
        // Convert the decrypted byte array into a Packet
        Ok(Packet::from_bytes(&decrypted_response))
    }

    // Send a file chunk to the selected client without acknowledgment
    pub async fn send_file_chunk(&mut self, data: &Vec<u8>) -> Result<(), String> {
        if let Some(stream) = self.get_connection(self.current_client) {
            if let Err(e) = stream.write_all(data).await {
                self.remove_connection(self.current_client);
                return Err(e.to_string());
            }
            Ok(())
        } else {
            Err(format!(
                "Client {} not found, please change current client using 'select <id>'",
                self.current_client
            ))
        }
    }

    // Send a message to a specific client
    pub async fn send_message_to(&mut self, message: &Packet, id: u16) -> Result<Packet, String> {
        let id_backup = self.current_client;
        self.set_current_client(id);
        let res = self.send_message(message).await;
        self.set_current_client(id_backup);
        res
    }
}
