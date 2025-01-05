use std::error::Error;
use std::time::Duration;

use common::messages::{Message, Packet};
use common::messages::list_files::ListFilesResponse;
use common::messages::ping::PingMessage;
use tokio::fs::read_dir;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::sleep;

pub async fn run_tcp_client(addr: String) {
    loop {
        match TcpStream::connect(&addr).await {
            Ok(mut stream) => {
                println!("Connected to file server!");

                let mut buffer = [0u8; 1024];
                loop {
                    match stream.read(&mut buffer).await {
                        Ok(0) => {
                            // 0 bytes read indicates the connection is closed
                            println!("Connection closed by server.");
                            break; // Exit the loop to auto reconnect
                        }
                        Ok(n) => {
                            if let Err(e) = handle_message(&mut stream, &Packet::from_bytes(&buffer[..n])).await {
                                eprintln!("Failed to handle message: {}", e);
                                break; // Exit the loop to auto reconnect
                            }
                        }
                        Err(e) => {
                            eprintln!("Error reading from stream: {}", e);
                            break; // Exit the loop to auto reconnect
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

pub async fn handle_message(stream: &mut TcpStream, message: &Packet) -> Result<(), Box<dyn Error>> {
    match message {
        Packet::Ping(msg) => {
            let pong_message = PingMessage {
                message: msg.message.clone(),
            };
            let pong_packet = Packet::Ping(pong_message);
            stream.write_all(&*pong_packet.to_bytes()).await?;
        }
        Packet::ListFiles(msg) => {
            let mut entries = read_dir(&msg.path).await?;
            let mut files: Vec<String> = Vec::new();
            while let Some(entry) = entries.next_entry().await? {
                let entry_path = entry.path();
                files.push(entry_path.to_string_lossy().to_string());
            }
            let response = ListFilesResponse {
                files,
            };
            let response_packet = Packet::ListFilesResponse(response);
            stream.write_all(&*response_packet.to_bytes()).await?;
        }
        _ => {}
    }
    Ok(())
}
