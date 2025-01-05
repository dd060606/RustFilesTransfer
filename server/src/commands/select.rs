use async_trait::async_trait;
use colored::Colorize;

use crate::{error, success};

use super::{Command, CommandRegistry};

pub struct SelectCommand;

#[async_trait]
impl Command for SelectCommand {
    fn name(&self) -> &str {
        "select"
    }

    fn description(&self) -> String {
        format!("{} {} {}", self.name(), "<id>", "- Selects a client")
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["sel"]
    }

    async fn execute(&self, registry: &CommandRegistry, args: Vec<String>) {
        if args.len() < 1 {
            registry.print_usage(self);
            return;
        }
        if let Ok(id) = args[0].parse::<u16>() {
            let mut connections = registry.connections.lock().await;
            if connections.exists(id) {
                // Change the current client
                connections.set_current_client(id);
                success!("Client {} selected", id);
            } else {
                // Client does not exist
                error!(
                    "Client {} does not exist, please check available clients using 'list'",
                    id
                );
            }
        } else {
            error!("Please provide a valid client id (number)");
        }
    }
}
