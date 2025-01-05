use std::error::Error;
use std::time::Duration;

use common::messages::{Message, Packet};
use common::messages::ping::PingMessage;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::sleep;

pub async fn run_tcp_client(addr: String) {
    loop {
        match TcpStream::connect(&addr).await {
            Ok(mut stream) => {
                println!("Connected to file server!");

                // Example of reading and writing to the stream
                let mut buffer = [0u8; 1024];
                loop {
                    if let Ok(n) = stream.read(&mut buffer).await {
                        if n == 0 {
                            // End of stream
                            println!("Connection closed by server.");
                            break;
                        } else {
                            //Handle message from the server
                            match handle_message(&mut stream, &Packet::from_bytes(&buffer[..n])).await {
                                Ok(_) => {}
                                Err(e) => {
                                    eprintln!("Failed to handle message: {}", e);
                                    break;
                                }
                            }
                        }
                    }
                    sleep(Duration::from_millis(100)).await; // Prevent busy-waiting
                }
            }
            Err(e) => {
                eprintln!("Failed to connect to file server: {}. Retrying in 10 seconds...", e);
                sleep(Duration::from_secs(10)).await;
            }
        }
    }
}

// Handle a message from the server
pub async fn handle_message(stream: &mut TcpStream, message: &Packet) -> Result<(), Box<dyn Error>> {
    match message {
        Packet::Ping(msg) => {
            // Create a pong message
            let pong_message = PingMessage {
                message: msg.message.clone(),
            };
            let pong_packet = Packet::Ping(pong_message);
            // Send the pong message back to the server
            stream.write_all(&*pong_packet.to_bytes()).await?
        }
    }
    Ok(())
}