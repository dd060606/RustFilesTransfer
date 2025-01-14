use super::{Command, CommandRegistry};
use crate::{error, success};
use async_trait::async_trait;
use colored::Colorize;
use common::messages::copy::CopyFileMessage;
use common::messages::Packet;
use regex::Regex;

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

        let args_str = args.join(" ");
        // Regex pattern to match arguments enclosed in double or single quotes, or standalone arguments
        let re = Regex::new(r#"'([^']*)'|"([^"]*)"|(\S+)"#).unwrap();

        let mut parsed_args: Vec<String> = Vec::new();

        for cap in re.captures_iter(&args_str) {
            if let Some(single_quoted) = cap.get(1) {
                // Argument was in single quotes
                parsed_args.push(single_quoted.as_str().to_string());
            } else if let Some(double_quoted) = cap.get(2) {
                // Argument was in double quotes
                parsed_args.push(double_quoted.as_str().to_string());
            } else if let Some(unquoted) = cap.get(3) {
                // Argument was not in quotes
                parsed_args.push(unquoted.as_str().to_string());
            }
        }

        let source = &parsed_args[0];
        let destination = &parsed_args[1];

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
