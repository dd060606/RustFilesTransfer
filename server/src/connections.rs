use common::messages::{Message, Packet};
use std::collections::HashMap;
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
    pub clients_info: HashMap<u16, ClientInfo>,
    //ID of the current connection
    pub current_client: u16,
}

impl Connections {
    pub fn new() -> Connections {
        let clients = HashMap::new();
        let clients_info = HashMap::new();
        Connections {
            clients,
            clients_info,
            current_client: 1,
        }
    }
    pub fn add_connection(&mut self, id: u16, stream: TcpStream) {
        self.clients.insert(id, stream);
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
        if let Some(stream) = self.get_connection(self.current_client) {
            // Send the message
            match stream.write_all(&*message.to_bytes()).await {
                Ok(_) => {}
                Err(e) => {
                    self.remove_connection(self.current_client);
                    return Err(e.to_string());
                }
            }

            // Read the response
            let mut buffer = [0; 1024];
            let mut total_data = Vec::new();

            loop {
                match stream.read(&mut buffer).await {
                    Ok(size) => {
                        if size == 0 {
                            // If total_data is empty, the connection was closed unexpectedly
                            if total_data.is_empty() {
                                self.remove_connection(self.current_client);
                                return Err(String::from("Connection closed unexpectedly!"));
                            } else {
                                // If there is already data, consider it a normal closure
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
                        return Err(format!("Failed to read from server: {}", e));
                    }
                }
            }
            // Convert the total_data into a Packet
            Ok(Packet::from_bytes(&total_data))
        } else {
            Err(format!(
                "Client {} not found, please change current client using 'select <id>'",
                self.current_client
            ))
        }
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
