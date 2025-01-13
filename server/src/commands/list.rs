use async_trait::async_trait;
use colored::Colorize;

use super::{Command, CommandRegistry};

pub struct ListCommand;

#[async_trait]
impl Command for ListCommand {
    fn name(&self) -> &str {
        "list"
    }

    fn description(&self) -> String {
        format!("{} {}", self.name(), "- Lists connected clients")
    }

    fn aliases(&self) -> Vec<&str> {
        vec![]
    }

    async fn execute(&self, registry: &CommandRegistry, _args: Vec<String>) {
        println!(
            "{}",
            "ID       IP                USERNAME        COMPUTER".cyan()
        );
        println!(
            "{}",
            "--       --                --------        --------".cyan()
        );
        // Print each connection
        let connections = registry.connections.lock().await;

        for connection in &connections.clients {
            let id = *connection.0;
            let address = connection.1.peer_addr().unwrap();
            let client_info = connections.get_client_info(id);

            if connections.current_client == id {
                // Print the selected connection in green
                println!(
                    "{}",
                    format!(
                        "{:<8} {:<18} {:<15} {:<15} (SELECTED)",
                        id, address, client_info.username, client_info.computer_name
                    )
                    .bright_green()
                );
            } else {
                // Print the other connections
                println!(
                    "{:<8} {:<18} {:<15} {:<15}",
                    id, address, client_info.username, client_info.computer_name
                );
            }
        }
    }
}
