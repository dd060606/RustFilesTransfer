use std::collections::HashMap;
use std::sync::Arc;

use colored::Colorize;
use rustyline::ExternalPrinter;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

use crate::{ext_error, ext_success};

pub struct Server<P: ExternalPrinter + Send + Sync + 'static> {
    //Map of connections (ID, TcpStream)
    connections: HashMap<u16, TcpStream>,
    //ID of the current connection
    current_connection: u16,
    //Rusyline printer
    printer: Arc<Mutex<P>>,
    port: String,

}

impl<P: ExternalPrinter + Send + Sync + 'static> Server<P> {
    pub fn new(printer: Arc<Mutex<P>>, port: String) -> Self {
        Server {
            connections: HashMap::new(),
            current_connection: 1,
            printer,
            port,
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

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {

        //Search for an available port
        let listener = search_port(&mut self.port).await;

        //Get the printer lock
        let mut printer = self.printer.lock().await;
        let _ = ext_success!(printer, "Server running on {}:{}", "0.0.0.0", self.port);

        loop {
            let (socket, addr) = listener.accept().await?;
            ext_success!(printer, "New connection from: {}", addr);
            let printer_clone = Arc::clone(&self.printer);
            tokio::spawn(async move {
                let mut printer_clone = printer_clone.lock().await;

                if let Err(e) = Self::handle_connection(socket).await {
                    ext_error!(printer_clone,"Error handling connection: {}",e);
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

//Utils

//Search for an available port
async fn search_port(port: &mut String) -> TcpListener {
    loop {
        match TcpListener::bind(format!("0.0.0.0:{}", port)).await {
            Ok(listener) => break listener,
            Err(_) => {
                match port.parse::<u16>() {
                    Ok(port_num) => {
                        // Increment the port number and update the value
                        *port = (port_num + 1).to_string();
                    }
                    Err(_) => {
                        // The port is not a number, reset to a default
                        *port = "8505".to_string();
                    }
                }
            }
        }
    }
}