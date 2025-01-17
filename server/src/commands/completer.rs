use common::messages::list_files::ListFilesMessage;
use common::messages::Packet;
use rustyline::completion::{extract_word, Completer, FilenameCompleter, Pair};
use rustyline::error::ReadlineError;
use rustyline::Context;
use rustyline_derive::{Helper, Highlighter, Hinter, Validator};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::connections::Connections;

const DEFAULT_BREAK_CHARS: [char; 3] = [' ', '\t', '\n'];

// Rustyline command completer
#[derive(Helper, Hinter, Validator, Highlighter)]
pub struct CommandHelper {
    file_completer: FilenameCompleter,
    commands: Vec<String>,
    connections: Arc<Mutex<Connections>>,
}

impl Completer for CommandHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        match self.auto_complete(&self.commands, &self.connections, line, pos) {
            Ok((start, matches)) => {
                // If no matches are found, use the file completer
                if matches.is_empty() {
                    //   self.file_completer.complete(line, pos, ctx)
                    Ok((start, vec![]))
                } else {
                    Ok((start, matches))
                }
            }
            Err(e) => Err(e),
        }
    }
}

impl CommandHelper {
    pub fn new(commands: Vec<String>, connections: Arc<Mutex<Connections>>) -> Self {
        Self {
            file_completer: FilenameCompleter::new(),
            commands,
            connections,
        }
    }
    // Auto complete a command
    fn auto_complete(
        &self,
        commands: &Vec<String>,
        connections: &Arc<Mutex<Connections>>,
        line: &str,
        pos: usize,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let (start, word) = extract_word(line, pos, None, |c| DEFAULT_BREAK_CHARS.contains(&c));
        // Get the command before the word
        let pre_cmd = line.split_once(" ").unwrap_or(("", "")).0;

        // If the command is empty, search for commands to auto complete
        if pre_cmd.len() == 0 {
            // Search for matches
            let matches = commands
                .iter()
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
            return Ok((start, matches));
        }

        // Async block to get the connections
        let res = futures::executor::block_on(async {
            let mut matches = Vec::new();
            // Auto complete the select command
            if is_pre_cmd(pre_cmd, &["select", "sel"]) {
                let connections = connections.lock().await;
                for id in connections.clients.keys() {
                    matches.push(Pair {
                        display: id.to_string(),
                        replacement: id.to_string(),
                    });
                }
            }
            // Auto complete the list files command
            else if is_pre_cmd(pre_cmd, &["ls", "dir"]) {
                let path = line.split_once(" ").unwrap().1;
                matches = remote_completion(path, true, connections).await;
            }
            // Auto complete the remove file command
            else if is_pre_cmd(pre_cmd, &["rm", "del"]) {
                let path = line.split_once(" ").unwrap().1;
                matches = remote_completion(path, false, connections).await;
            } else if is_pre_cmd(pre_cmd, &["cp", "copy"]) {
                let path = line.split_once(" ").unwrap().1;
                matches = remote_completion(path, false, connections).await;
            }

            matches
        });

        Ok((start, res))
    }
}
fn is_pre_cmd(pre_cmd: &str, cmd: &[&str]) -> bool {
    cmd.iter().any(|&c| pre_cmd.eq_ignore_ascii_case(c))
}

// Remote file completion
async fn remote_completion(
    current_path: &str,
    only_dir: bool,
    connections: &Arc<Mutex<Connections>>,
) -> Vec<Pair> {
    let mut matches = Vec::new();
    // Remove quotes from the path
    let path = current_path.trim().replace("\"", "").replace("\'", "");
    let mut connections = connections.lock().await;
    // Create a new message
    let message = ListFilesMessage {
        path: path.clone(),
        only_directories: only_dir,
    };
    // Create a new packet with the message
    let packet = Packet::ListFiles(message);
    // Send the packet to the client
    match connections.send_message(&packet).await {
        Ok(res) => match res {
            Packet::ListFilesResponse(response) => {
                for file in response.files {
                    // Check if the path starts with the file
                    if file.starts_with(&path) {
                        matches.push(Pair {
                            display: file.to_string(),
                            replacement: file.to_string(),
                        });
                    }
                }
            }
            _ => {}
        },
        Err(_) => {}
    };
    matches
}
