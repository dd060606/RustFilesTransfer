use std::sync::Arc;

use async_trait::async_trait;
use colored::Colorize;
use rustyline::completion::{Completer, extract_word, FilenameCompleter, Pair};
use rustyline::Context;
use rustyline::error::ReadlineError;
use rustyline_derive::{Helper, Highlighter, Hinter, Validator};
use tokio::sync::Mutex;

use crate::connections::Connections;

mod help;
mod list;
mod ping;
mod select;

const DEFAULT_BREAK_CHARS: [char; 3] = [' ', '\t', '\n'];

#[async_trait]
pub trait Command: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> String;
    fn aliases(&self) -> Vec<&str>;
    async fn execute(&self, registry: &CommandRegistry, args: Vec<String>);
}

pub struct CommandRegistry {
    pub(crate) commands: Vec<Box<dyn Command>>,
    connections: Arc<Mutex<Connections>>,
}

impl CommandRegistry {
    pub fn new(connections: Arc<Mutex<Connections>>) -> Self {
        let mut registry = Self {
            commands: Vec::new(),
            connections,
        };
        // Register commands here
        registry.register(Box::new(help::HelpCommand));
        registry.register(Box::new(list::ListCommand));
        registry.register(Box::new(ping::PingCommand));
        registry.register(Box::new(select::SelectCommand));

        registry
    }

    // Register a command
    pub fn register(&mut self, command: Box<dyn Command>) {
        self.commands.push(command);
    }


    // Execute a command
    pub async fn execute(&self, command: String) -> Result<(), String> {
        match self.get_cmd(&command) {
            // Command not found
            None => Err(format!(
                "Command '{}' not found. Use 'help' to see all available commands.",
                command.split_whitespace().next().unwrap_or("")
            )),
            // Execute the command
            Some(cmd) => Ok(cmd.execute(self, self.parse_args(&command)).await),
        }
    }

    //Get command by name
    pub fn get_cmd(&self, cmd_name: &String) -> Option<&Box<dyn Command>> {
        // Get the first word of the input
        let cmd_name = cmd_name.split_whitespace().next().unwrap_or("");

        for command in self.commands.iter() {
            // Check if the command name starts with the input
            if cmd_name.eq_ignore_ascii_case(&command.name()) {
                return Some(command);
            }
            // Check aliases
            for alias in command.aliases() {
                if cmd_name.eq_ignore_ascii_case(&alias) {
                    return Some(command);
                }
            }
        }
        None
    }

    // Parse command arguments
    pub fn parse_args(&self, cmd: &String) -> Vec<String> {
        // Split the command into arguments thanks to the whitespace
        let mut args: Vec<String> = cmd.split_whitespace().map(|s| s.to_string()).collect();
        // Remove the command name
        args.remove(0);
        args
    }

    // Display the usage of a command
    pub fn print_usage(&self, cmd: &dyn Command) {
        println!("{}", "Usage:\n------".bright_yellow());
        println!("  {}", cmd.description());
    }

    // Create a list of all available commands
    pub fn get_commands(&self) -> Vec<String> {
        let mut all_commands: Vec<String> = Vec::new();
        for cmd in self.commands.iter() {
            all_commands.push(cmd.name().to_string());
        }
        all_commands
    }
}

// Rustyline command completer
#[derive(Helper, Hinter, Validator, Highlighter)]
pub struct CommandHelper {
    pub file_completer: FilenameCompleter,
    pub commands: Vec<String>,
}

impl Completer for CommandHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        match auto_complete(&self.commands, line, pos) {
            Ok((start, matches)) => {
                // If no matches are found, use the file completer
                if matches.is_empty() {
                    self.file_completer.complete(line, pos, ctx)
                } else {
                    Ok((start, matches))
                }
            }
            Err(e) => Err(e),
        }
    }
}

// Auto complete a command
fn auto_complete(commands: &Vec<String>, line: &str, pos: usize) -> rustyline::Result<(usize, Vec<Pair>)> {
    let (start, word) = extract_word(line, pos, None, |c| DEFAULT_BREAK_CHARS.contains(&c));

    let matches = commands.iter()
        .filter_map(|hint| {
            if hint.starts_with(word) {
                let mut replacement = hint.to_string();
                replacement += " ";
                Some(Pair {
                    display: hint.to_string(),
                    replacement: replacement.to_string(),
                })
            } else {
                None
            }
        })
        .collect();
    Ok((start, matches))
}