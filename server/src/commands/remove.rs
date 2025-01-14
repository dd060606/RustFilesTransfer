use super::{Command, CommandRegistry};
use crate::{error, success};
use async_trait::async_trait;
use colored::Colorize;
use common::messages::remove::RemoveMessage;
use common::messages::Packet;
use std::path::PathBuf;
pub struct RemoveFileCommand;

#[async_trait]
impl Command for RemoveFileCommand {
    fn name(&self) -> &str {
        "rm"
    }

    fn description(&self) -> String {
        format!(
            "{} {} {}",
            self.name(),
            "(<path>)",
            "- Removes a file or directory"
        )
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["del"]
    }

    async fn execute(&self, registry: &CommandRegistry, args: Vec<String>) {
        if args.len() < 1 {
            registry.print_usage(self);
            return;
        }
        let path = args.join(" ").replace("\"", "").replace("\'", "");
        let remove_message = RemoveMessage {
            path: PathBuf::from(path),
        };
        let remove_packet = Packet::RemoveFile(remove_message);
        let mut connections = registry.connections.lock().await;
        match connections.send_message(&remove_packet).await {
            Ok(res) => match res {
                Packet::ConfirmResponse(_) => {
                    success!("File removed successfully");
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
