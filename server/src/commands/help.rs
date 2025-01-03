use async_trait::async_trait;
use colored::Colorize;

use crate::error;

use super::{Command, CommandRegistry};

pub struct HelpCommand;

#[async_trait]
impl Command for HelpCommand {
    fn name(&self) -> &str {
        "help"
    }

    fn description(&self) -> String {
        format!(
            "{} {} {}",
            self.name(),
            "(<command name>)",
            "- Displays a list of available commands"
        )
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["h"]
    }

    async fn execute(&self, registry: &CommandRegistry, args: Vec<String>) {
        if args.len() > 0 {
            // Print the usage of a specific command
            if let Some(cmd) = registry.get_cmd(&args[0]) {
                registry.print_usage(&**cmd)
            } else {
                error!("Command '{}' not found. Use 'help' to see all available commands.", args[0]);
            }
        } else {
            // Print all commands
            println!("{}", "Commands\n=========".bold().cyan());
            registry
                .commands
                .iter()
                .for_each(|cmd| println!("{}", cmd.description()));
        }
    }
}
