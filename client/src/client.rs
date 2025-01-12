use std::env;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::time::Duration;

use common::messages::{Message, Packet};
use common::messages::list_files::ListFilesResponse;
use common::messages::ping::PingMessage;
use common::messages::response::ErrorResponse;
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
                            if let Err(e) =
                                handle_message(&mut stream, &Packet::from_bytes(&buffer[..n])).await
                            {
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
                eprintln!(
                    "Failed to connect to file server: {}. Retrying in 10 seconds...",
                    e
                );
                sleep(Duration::from_secs(10)).await;
            }
        }
    }
}

pub async fn handle_message(
    stream: &mut TcpStream,
    message: &Packet,
) -> Result<(), Box<dyn Error>> {
    match message {
        Packet::Ping(msg) => {
            //Ping response
            let pong_message = PingMessage {
                message: msg.message.clone(),
            };
            let pong_packet = Packet::Ping(pong_message);
            stream.write_all(&*pong_packet.to_bytes()).await?;
        }
        Packet::ListFiles(msg) => {
            let path = if msg.path.is_empty() {
                // If no path is provided, use the current directory
                env::current_dir().unwrap_or_default()
            } else {
                let pathbuf = PathBuf::from(&msg.path);
                // If the path does not exist return parent directory
                if !pathbuf.exists() {
                    pathbuf.parent().unwrap_or(Path::new("/")).to_path_buf()
                } else {
                    pathbuf
                }
            };
            // Handle the result of reading the directory
            match read_dir(path).await {
                Ok(mut entries) => {
                    let mut files: Vec<String> = Vec::new();
                    // Iterate over the entries in the directory
                    while let Some(entry) = entries.next_entry().await? {
                        let entry_path = entry.path();
                        // Skip files if only directories are requested
                        if msg.only_directories && entry_path.is_file() {
                            continue;
                        }
                        files.push(entry_path.to_string_lossy().to_string());
                    }
                    println!("Sending list of files to server... {:?}", files);
                    // Send the list of files back to the server
                    let response = ListFilesResponse { files };
                    let response_packet = Packet::ListFilesResponse(response);
                    stream.write_all(&*response_packet.to_bytes()).await?;
                }
                Err(err) => {
                    send_error(stream, err.to_string()).await?;
                }
            }
        }
        _ => {}
    }
    Ok(())
}

// Send an error message to the server
pub async fn send_error(stream: &mut TcpStream, error: String) -> Result<(), Box<dyn Error>> {
    let error_message = ErrorResponse { error: error };
    let error_packet = Packet::ErrorResponse(error_message);
    let _ = stream.write_all(&*error_packet.to_bytes()).await?;
    Ok(())
}
