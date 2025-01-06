use async_trait::async_trait;
use colored::Colorize;
use common::messages::list_files::ListFilesMessage;
use common::messages::Packet;

use crate::{error, success};

use super::{Command, CommandRegistry};

pub struct ListFilesCommand;

#[async_trait]
impl Command for ListFilesCommand {
    fn name(&self) -> &str {
        "ls"
    }

    fn description(&self) -> String {
        format!(
            "{} {} {}",
            self.name(),
            "(<path>)",
            "- Lists files in the current directory"
        )
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["dir"]
    }

    async fn execute(&self, registry: &CommandRegistry, args: Vec<String>) {
        // Create a new ping message
        let message = ListFilesMessage {
            path: args.join(" "),
        };
        // Create a new packet with the ping message
        let packet = Packet::ListFiles(message);

        let mut connections = registry.connections.lock().await;

        // Send the packet to the client
        match connections.send_message(&packet).await {
            Ok(res) => match res {
                Packet::ListFilesResponse(response) => {
                    success!("Files:");
                    println!("{}", response.files.join("\n"));
                }
                Packet::ErrorResponse(response) => {
                    error!("Error: {}", response.error);
                }
                _ => {}
            },
            Err(e) => {
                error!("Failed to send message: {}", e);
            }
        };
    }
}
