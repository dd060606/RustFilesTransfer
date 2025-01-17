use super::{Command, CommandRegistry};
use crate::utils::files::extract_paths;
use crate::{error, success};
use async_trait::async_trait;
use colored::Colorize;
use common::messages::copy::CopyFileMessage;
use common::messages::Packet;
use std::path::PathBuf;

pub struct CopyCommand;

#[async_trait]
impl Command for CopyCommand {
    fn name(&self) -> &str {
        "copy"
    }

    fn description(&self) -> String {
        format!(
            "{} {} {} {}",
            self.name(),
            "<source>",
            "<destination>",
            "- Copy a file to a different location"
        )
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["cp"]
    }

    async fn execute(&self, registry: &CommandRegistry, args: Vec<String>) {
        if args.len() < 2 {
            registry.print_usage(self);
            return;
        }

        // Extract the source and destination paths
        let joined_args = args.join(" ");
        let paths = extract_paths(&joined_args);
        let source = paths[0];
        let destination = if paths.len() > 1 { paths[1] } else { source };

        // Create a new message
        let message = CopyFileMessage {
            source: PathBuf::from(source),
            output: PathBuf::from(destination),
        };
        let packet = Packet::CopyFile(message);
        let mut connections = registry.connections.lock().await;
        //Send the packet to the client
        match connections.send_message(&packet).await {
            Ok(res) => match res {
                Packet::ConfirmResponse(_) => {
                    success!("File copied successfully");
                }
                Packet::ErrorResponse(response) => {
                    error!("Error: {}", response.error);
                }
                _ => {}
            },
            Err(e) => {
                error!("Failed to send message: {}", e);
            }
        }
    }
}
