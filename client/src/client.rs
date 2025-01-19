use std::env;
use std::error::Error;
use std::process::{Command, exit};
use std::time::Duration;

use common::messages::{Message, Packet};
use common::messages::info::InfoResponse;
use common::messages::list_files::ListFilesResponse;
use common::messages::ping::PingMessage;
use common::messages::response::{ConfirmResponse, ErrorResponse};
use tokio::fs::{copy, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::sleep;

use crate::files::{list_files, remove};

pub async fn run_tcp_client(addr: String) {
    loop {
        match TcpStream::connect(&addr).await {
            Ok(mut stream) => {
                println!("Connected to file server!");

                let mut buffer = [0; 4096];
                let mut total_data = Vec::new();

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
                                // Once the message is complete, handle the message
                                if let Err(e) =
                                    handle_message(&mut stream, &Packet::from_bytes(&total_data))
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

// Handle the message received from the server
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
            match list_files(&msg.path, msg.only_directories).await {
                Ok(files) => {
                    // Send the list of files back to the server
                    let response = ListFilesResponse { files };
                    let response_packet = Packet::ListFilesResponse(response);
                    stream.write_all(&*response_packet.to_bytes()).await?;
                }
                Err(e) => send_error(stream, e.to_string()).await?,
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
            stream.write_all(&*info_packet.to_bytes()).await?;
        }
        Packet::CopyFile(msg) => {
            // Copy file
            match copy(&msg.source, &msg.output).await {
                Ok(_) => send_confirm(stream).await?,
                Err(e) => send_error(stream, e.to_string()).await?,
            }
        }
        Packet::RemoveFile(msg) => {
            // Remove file
            match remove(&msg.path).await {
                Ok(_) => send_confirm(stream).await?,
                Err(e) => send_error(stream, e.to_string()).await?,
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
                    send_confirm(stream).await?;
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
                Err(e) => send_error(stream, e.to_string()).await?,
            }
        }
        Packet::Elevate(_) => {
            // Elevate the client
            if let Err(e) = run_as_admin() {
                send_error(stream, e.to_string()).await?;
            } else {
                send_confirm(stream).await?;
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

// Send an error message to the server
pub async fn send_error(stream: &mut TcpStream, error: String) -> Result<(), Box<dyn Error>> {
    let error_message = ErrorResponse { error: error };
    let error_packet = Packet::ErrorResponse(error_message);
    let _ = stream.write_all(&*error_packet.to_bytes()).await?;
    Ok(())
}

// Send a confirm response to the server
pub async fn send_confirm(stream: &mut TcpStream) -> Result<(), Box<dyn Error>> {
    let confirm_message = ConfirmResponse {};
    let confirm_response = Packet::ConfirmResponse(confirm_message);
    stream.write_all(&*confirm_response.to_bytes()).await?;
    Ok(())
}

// Elevate the client if possible
#[cfg(target_os = "windows")]
fn run_as_admin() -> Result<(), Box<dyn Error>> {
    let exe_path = env::current_exe()?;

    // Collect arguments, skipping the program path
    let args: Vec<String> = env::args().skip(1).collect();

    // Format the command to execute using PowerShell's Start-Process cmdlet
    let command = format!(
        "Start-Process \"{}\" -ArgumentList \"{}\" -Verb runAs",
        exe_path.display(),
        args.join(" ")
    );
    // Use `powershell` to request elevation
    match Command::new("powershell")
        .arg("-Command")
        .arg(command)
        .spawn()
    {
        Ok(mut child) => {
            // Wait for the child process to finish
            match child.wait() {
                Ok(status) => {
                    if !status.success() {
                        return Err("Failed to launch the process as administrator.".into());
                    }
                }
                Err(e) => {
                    return Err(format!("Failed to wait on child process: {}", e).into());
                }
            }
        }
        Err(e) => {
            return Err(format!("Failed to execute process with runas: {}", e).into());
        }
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn run_as_admin() -> Result<(), Box<dyn Error>> {
    let exe_path = env::current_exe()?;

    // Collect arguments, skipping the program path
    let args: Vec<String> = env::args().skip(1).collect();

    // Use `sudo` to request root privileges on Linux
    match Command::new("sudo")
        .arg(exe_path)
        .args(&args)
        .spawn()
    {
        Ok(mut child) => {
            // Wait for the child process to finish
            match child.wait() {
                Ok(status) => {
                    if !status.success() {
                        return Err("Failed to launch the process as administrator.".into());
                    }
                }
                Err(e) => {
                    return Err(format!("Failed to wait on child process: {}", e).into());
                }
            }
        }
        Err(e) => {
            return Err(format!("Failed to execute process with runas: {}", e).into());
        }
    }

    Ok(())
}
