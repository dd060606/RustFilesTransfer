use std::collections::HashMap;

use common::messages::{Message, Packet};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub struct Connections {
    //Map of connections (ID, TcpStream)
    pub clients: HashMap<u16, TcpStream>,
    //ID of the current connection
    pub current_client: u16,
}

impl Connections {
    pub fn new() -> Connections {
        let clients = HashMap::new();
        Connections {
            clients,
            current_client: 1,
        }
    }
    pub fn add_connection(&mut self, id: u16, stream: TcpStream) {
        self.clients.insert(id, stream);
    }

    pub fn get_connection(&mut self, id: u16) -> Option<&mut TcpStream> {
        self.clients.get_mut(&id)
    }

    pub fn remove_connection(&mut self, id: u16) {
        self.clients.remove(&id);
    }

    //Check if the connection exists
    pub fn exists(&self, id: u16) -> bool {
        self.clients.contains_key(&id)
    }
    //Change current client ID
    pub fn set_current_client(&mut self, id: u16) {
        self.current_client = id;
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
            // Wait for a response
            let mut buffer = [0; 1024];
            match stream.read(&mut buffer).await {
                Ok(size) => {
                    if size == 0 {
                        self.remove_connection(self.current_client);
                        Err(String::from("Error while receiving the message!"))
                    } else {
                        // Return the response
                        Ok(Packet::from_bytes(&buffer[..size]))
                    }
                }
                Err(e) => Err(format!("Failed to read from server: {}", e)),
            }
        } else {
            Err(format!(
                "Client {} not found, please change current client using 'select <id>'",
                self.current_client
            ))
        }
    }
}