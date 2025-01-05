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
        println!("{}", "ID       IP".cyan());
        println!("{}", "--       --".cyan());
        //Print each connections
        let connections = registry.connections.lock().await;

        for connection in &connections.clients {
            let id = *connection.0;
            let address = connection.1.peer_addr().unwrap();
            if connections.current_client == id {
                //Print the selected connection in green
                println!(
                    "{}",
                    format!("{}        {}  (SELECTED)", id, address).bright_green()
                )
            } else {
                println!("{}        {}", id, address);
            }
        }
    }
}