use std::sync::Arc;

use colored::Colorize;
use common::messages::info::InfoMessage;
use common::messages::Packet;
use rustyline::ExternalPrinter;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

use crate::connections::{ClientInfo, Connections};
use crate::ext_success;

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
            let (socket, addr) = listener.accept().await?;
            ext_success!(printer, "New connection: {} ({})", addr, next_id);

            // Add the connection to the list
            let mut connections = self.connections.lock().await;
            connections.add_connection(next_id, socket);
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
