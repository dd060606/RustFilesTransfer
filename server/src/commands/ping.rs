use async_trait::async_trait;
use colored::Colorize;
use common::messages::Packet;
use common::messages::ping::PingMessage;

use crate::error;

use super::{Command, CommandRegistry};

pub struct PingCommand;

#[async_trait]
impl Command for PingCommand {
    fn name(&self) -> &str {
        "ping"
    }

    fn description(&self) -> String {
        format!("{} {}", self.name(), "- Sends a ping message to the server")
    }

    fn aliases(&self) -> Vec<&str> {
        vec![]
    }

    async fn execute(&self, registry: &CommandRegistry, args: Vec<String>) {
        // Create a new ping message
        let message = PingMessage {
            message: args.join(" "),
        };
        // Create a new packet with the ping message
        let packet = Packet::Ping(message);

        let mut connections = registry.connections.lock().await;

        // Send the packet to the client
        match connections.send_message(&packet).await {
            Ok(res) => {
                let Packet::Ping(msg) = res;
                println!("Response: {}", msg.message);
            }
            Err(e) => {
                error!("Failed to send message: {}", e);
            }
        };
    }
}