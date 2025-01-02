use colored::Colorize;
use rustyline::{CompletionType, Config, EditMode, Editor, ExternalPrinter};
use rustyline::completion::FilenameCompleter;
use rustyline::error::ReadlineError;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::task;

use crate::commands::{CommandHelper, CommandRegistry};
use crate::server::Server;

mod commands;
mod macros;
mod server;


#[tokio::main]
async fn main() {
    // Initialize the command registry
    let cmd_registry = CommandRegistry::new();
    // Create a new instance of the default editor
    let config = Config::builder()
        .history_ignore_space(true)
        .completion_type(CompletionType::List)
        .edit_mode(EditMode::Emacs)
        .build();
    let cmd_helper = CommandHelper {
        file_completer: FilenameCompleter::new(),
        commands: cmd_registry.get_commands(),
    };
    let mut rl = Editor::with_config(config).expect("Failed to create editor");
    rl.set_helper(Some(cmd_helper));
    let mut printer = rl.create_external_printer().expect("Failed to create external printer");


    // Start the TCP server
    task::spawn(async move {
        Server::new().run().await.unwrap();
    });

    // Command loop
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());
                // Execute the command
                match cmd_registry.execute(line) {
                    // Command not found error
                    Err(e) => eprintln!("{}", e),
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
