use super::{Command, CommandRegistry};
use crate::utils::files::{get_path_from_args, print_progress};
use crate::{error, success};
use async_trait::async_trait;
use colored::Colorize;
use common::messages::files::PrepareFileMessage;
use common::messages::Packet;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

pub struct UploadCommand;

#[async_trait]
impl Command for UploadCommand {
    fn name(&self) -> &str {
        "upload"
    }

    fn description(&self) -> String {
        format!(
            "{} {} {} {}",
            self.name(),
            "<source>",
            "<destination>",
            "- Upload a local file to the remote client"
        )
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["up"]
    }

    async fn execute(&self, registry: &CommandRegistry, args: Vec<String>) {
        if args.len() < 1 {
            registry.print_usage(self);
            return;
        }
        // If the user only provides the source file
        let (source, destination) = get_path_from_args(&args);

        // Open the local file
        match File::open(source).await {
            Ok(mut file) => {
                // Get the metadata of the file
                match file.metadata().await {
                    Ok(metadata) => {
                        let prepare_message = PrepareFileMessage {
                            output: PathBuf::from(destination),
                            size: metadata.len(),
                        };
                        let prepare_packet = Packet::PrepareFile(prepare_message);
                        let mut connections = registry.connections.lock().await;
                        // Send the prepare packet to the client
                        match connections.send_message(&prepare_packet).await {
                            Ok(res) => match res {
                                Packet::ConfirmResponse(_) => {
                                    // There is no problem with the file size or the file output
                                    // So we send the file to the client
                                    let mut buffer = [0; 65536]; // Chunk size (64KB)
                                    let mut sent_bytes: u64 = 0;

                                    loop {
                                        match file.read(&mut buffer).await {
                                            Ok(file_bytes) => {
                                                if file_bytes == 0 {
                                                    break;
                                                }
                                                // Send the file chunk to the client
                                                match connections
                                                    .send_file_chunk(&buffer[..file_bytes].to_vec())
                                                    .await
                                                {
                                                    Ok(_) => {
                                                        //Update the progress
                                                        sent_bytes += file_bytes as u64;
                                                        print_progress(sent_bytes, metadata.len());
                                                    }
                                                    Err(err) => {
                                                        error!(
                                                            "Failed to send file chunk: {}",
                                                            err
                                                        );
                                                        return;
                                                    }
                                                }
                                            }
                                            Err(err) => {
                                                error!("Failed to read local file: {}", err);
                                                return;
                                            }
                                        }
                                    }
                                    println!(); // Print a newline
                                    success!("File uploaded successfully");
                                }
                                Packet::ErrorResponse(response) => {
                                    error!("Client error: {}", response.error);
                                }
                                _ => {}
                            },
                            Err(e) => {
                                error!("Failed to send message: {}", e);
                            }
                        }
                    }
                    Err(err) => {
                        error!("Failed to open local file: {}", err);
                    }
                }
            }
            Err(err) => {
                error!("Failed to open local file: {}", err);
            }
        }
    }
}
