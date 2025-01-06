use std::sync::Arc;

use colored::Colorize;
use rustyline::completion::FilenameCompleter;
use rustyline::error::ReadlineError;
use rustyline::{CompletionType, Config, EditMode, Editor};
use tokio::sync::Mutex;
use tokio::task;

use crate::commands::completer::CommandHelper;
use crate::commands::CommandRegistry;
use crate::connections::Connections;
use crate::server::Server;

mod commands;
mod connections;
mod macros;
mod server;

#[tokio::main]
async fn main() {
    // Create a new instance of the connections struct
    let connections = Arc::new(Mutex::new(Connections::new()));
    let connections_clone = Arc::clone(&connections);

    // Initialize the command registry
    let cmd_registry = CommandRegistry::new(connections_clone);
    // Create a new instance of the default editor
    let config = Config::builder()
        .history_ignore_space(true)
        .completion_type(CompletionType::List)
        .edit_mode(EditMode::Emacs)
        .build();
    let cmd_helper = CommandHelper {
        file_completer: FilenameCompleter::new(),
        commands: cmd_registry.get_commands(),
        connections: connections.clone(),
    };
    let mut rl = Editor::with_config(config).expect("Failed to create editor");
    rl.set_helper(Some(cmd_helper));

    // Get port from cli arguments (default is 8505)
    let port = std::env::args().nth(1).unwrap_or("8505".to_string());
    // Create an external printer
    let printer = Arc::new(Mutex::new(
        rl.create_external_printer()
            .expect("Failed to create external printer"),
    ));

    // Start the TCP server
    task::spawn(async move {
        Server::new(port, connections, printer).run().await.unwrap();
    });

    // Command loop
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());
                // Execute the command
                match cmd_registry.execute(line).await {
                    // Command not found error
                    Err(e) => {
                        error!("{}", e);
                    }
                    _ => {}
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
}
