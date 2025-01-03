use std::collections::HashMap;

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
    //Change current connection ID
    pub fn set_current_connection(&mut self, id: u16) {
        self.current_client = id;
    }

    pub fn get_stream(&mut self) -> Option<&mut TcpStream> {
        if let Some(stream) = self.get_connection(self.current_client) {
            Some(stream)
        } else {
            None
        }
    }
}