use super::{Command, CommandRegistry};
use crate::utils::files::get_path_from_args;
use crate::{error, success};
use async_trait::async_trait;
use colored::Colorize;
use common::messages::copy::CopyFileMessage;
use common::messages::Packet;

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

        let (source, destination) = get_path_from_args(&args);

        // Create a new message
        let message = CopyFileMessage {
            source: std::path::PathBuf::from(source),
            output: std::path::PathBuf::from(destination),
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
