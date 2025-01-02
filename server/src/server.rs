use std::collections::HashMap;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

pub struct Server {
    //Map of connections (ID, TcpStream)
    pub connections: HashMap<u16, TcpStream>,
    //ID of the current connection
    pub current_connection: u16,
}

impl Server {
    pub fn new() -> Self {
        Server {
            connections: HashMap::new(),
            current_connection: 1,
        }
    }
    pub fn add_connection(&mut self, id: u16, stream: TcpStream) {
        self.connections.insert(id, stream);
    }

    pub fn get_connection(&mut self, id: u16) -> Option<&mut TcpStream> {
        self.connections.get_mut(&id)
    }

    pub fn remove_connection(&mut self, id: u16) {
        self.connections.remove(&id);
    }

    //Check if the connection exists
    pub fn exists(&self, id: u16) -> bool {
        self.connections.contains_key(&id)
    }
    //Change current connection ID
    pub fn set_current_connection(&mut self, id: u16) {
        self.current_connection = id;
    }

    pub fn get_stream(&mut self) -> Option<&mut TcpStream> {
        if let Some(stream) = self.get_connection(self.current_connection) {
            Some(stream)
        } else {
            None
        }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind("127.0.0.1:8080").await?;
        println!("Server running on {}", "127.0.0.1:8080");

        loop {
            let (socket, addr) = listener.accept().await?;
            println!("New connection from: {}", addr);

            tokio::spawn(async move {
                if let Err(e) = Self::handle_connection(socket).await {
                    eprintln!("Error handling connection: {}", e);
                }
            });
        }
    }

    async fn handle_connection(mut socket: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
        let mut buffer = [0u8; 1024];

        loop {
            let bytes_read = socket.read(&mut buffer).await?;

            if bytes_read == 0 {
                println!("Client disconnected");
                return Ok(());
            }

            let message = String::from_utf8_lossy(&buffer[..bytes_read]);
            println!("Received: {}", message);

            socket.write_all(message.as_bytes()).await?;
        }
    }
}