use std::sync::Arc;

use colored::Colorize;
use common::messages::info::InfoMessage;
use common::messages::Packet;
use common::utils::encryption::{generate_keypair, Encryptor};
use rustyline::ExternalPrinter;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

use crate::connections::{ClientInfo, Connections};
use crate::{ext_error, ext_success};

pub struct Server<P: ExternalPrinter + Send + Sync + 'static> {
    port: String,
    connections: Arc<Mutex<Connections>>,
    //Rusyline printer
    printer: Arc<Mutex<P>>,
}

impl<P: ExternalPrinter + Send + Sync + 'static> Server<P> {
    pub fn new(port: String, connections: Arc<Mutex<Connections>>, printer: Arc<Mutex<P>>) -> Self {
        Server {
            port,
            connections,
            printer,
        }
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        //Search for an available port
        let listener = search_port(&mut self.port).await;

        //Get the printer lock
        let mut printer = self.printer.lock().await;
        ext_success!(printer, "Server running on {}:{}", "0.0.0.0", self.port);


        // Counter to assign unique IDs to connections
        let mut next_id: u16 = 1;
        loop {
            let (mut socket, addr) = listener.accept().await?;
            // Exchange keys with the client
            let packet_encryption = match exchange_keys(&mut socket).await {
                Some(encryptor) => encryptor,
                None => {
                    ext_error!(printer, "Failed to exchange keys with client {}", addr);
                    continue;
                }
            };
            ext_success!(printer, "New connection: {} ({})", addr, next_id);

            // Add the connection to the list
            let mut connections = self.connections.lock().await;
            connections.add_connection(next_id, socket, packet_encryption);
            // Add the connection info
            let info_msg = InfoMessage {};
            let info_packet = Packet::Info(info_msg);
            if let Ok(info) = connections.send_message_to(&info_packet, next_id).await {
                // If the response is an InfoResponse, add the info to the list
                if let Packet::InfoResponse(response) = info {
                    connections.add_info(
                        next_id,
                        ClientInfo {
                            username: response.username,
                            computer_name: response.computer_name,
                        },
                    );
                }
            }
            // Increment the next ID for the next connection
            next_id += 1;
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

// Exchange keys and create an encryptor instance
async fn exchange_keys(stream: &mut TcpStream) -> Option<Encryptor> {
    // Generate a keypair and send the public key to the client
    let keypair = generate_keypair();
    if let Err(_) = stream.write_all(&keypair.public.to_bytes()).await {
        return None;
    }
    // Read the client's public key
    let mut client_public_bytes = [0u8; 32];
    if let Err(_) = stream.read_exact(&mut client_public_bytes).await {
        return None;
    }
    // Create the encryptor instance using the keypair and client's public key
    Some(Encryptor::new(keypair, client_public_bytes))
}