use std::time::Duration;

use async_trait::async_trait;
use colored::Colorize;
use common::messages::elevate::ElevateMessage;
use common::messages::Packet;
use tokio::time::sleep;

use crate::{error, success};

use super::{Command, CommandRegistry};

pub struct ElevateCommand;

#[async_trait]
impl Command for ElevateCommand {
    fn name(&self) -> &str {
        "elevate"
    }

    fn description(&self) -> String {
        format!("{} {}", self.name(), "- Elevates the client to admin")
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["admin"]
    }

    async fn execute(&self, registry: &CommandRegistry, _args: Vec<String>) {
        // Create a new elevation packet
        let message = ElevateMessage {};
        let packet = Packet::Elevate(message);
        let mut connections = registry.connections.lock().await;

        // Send the packet to the client
        match connections.send_message(&packet).await {
            Ok(res) => {
                if let Packet::ConfirmResponse(_) = res {
                    // Wait for a new connection to be established
                    drop(connections);
                    sleep(Duration::from_secs(1)).await;

                    // Update the current client to the last connected client
                    let mut connections = registry.connections.lock().await;
                    let current_client = connections.current_client;
                    connections.remove_connection(current_client);
                    match connections.clients.iter().last() {
                        Some(client) => {
                            connections.current_client = client.0.clone();
                        }
                        None => {
                            connections.current_client = 1;
                        }
                    }
                    println!();
                    success!("Client elevated to admin (ID: {})", connections.current_client);
                } else if let Packet::ErrorResponse(response) = res {
                    error!("Failed to elevate client to admin: {}", response.error);
                }
            }
            Err(e) => {
                error!("Failed to send message: {}", e);
            }
        };
    }
}