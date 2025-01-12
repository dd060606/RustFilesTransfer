use std::sync::Arc;

use common::messages::list_files::ListFilesMessage;
use common::messages::Packet;
use rustyline::completion::{Completer, extract_word, FilenameCompleter, Pair};
use rustyline::Context;
use rustyline::error::ReadlineError;
use rustyline_derive::{Helper, Highlighter, Hinter, Validator};
use tokio::sync::Mutex;

use crate::connections::Connections;

const DEFAULT_BREAK_CHARS: [char; 3] = [' ', '\t', '\n'];

// Rustyline command completer
#[derive(Helper, Hinter, Validator, Highlighter)]
pub struct CommandHelper {
    pub file_completer: FilenameCompleter,
    pub commands: Vec<String>,
    pub connections: Arc<Mutex<Connections>>,
}

impl Completer for CommandHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        match auto_complete(&self.commands, &self.connections, line, pos) {
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

// Auto complete a command
fn auto_complete(
    commands: &Vec<String>,
    connections: &Arc<Mutex<Connections>>,
    line: &str,
    pos: usize,
) -> rustyline::Result<(usize, Vec<Pair>)> {
    let (start, word) = extract_word(line, pos, None, |c| DEFAULT_BREAK_CHARS.contains(&c));
    let pre_cmd = line[..start].trim();

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
        if pre_cmd.eq_ignore_ascii_case("select") || pre_cmd.eq_ignore_ascii_case("sel") {
            let connections = connections.lock().await;
            for id in connections.clients.keys() {
                matches.push(Pair {
                    display: id.to_string(),
                    replacement: id.to_string(),
                });
            }
        }
        // Auto complete the list_files command
        else if pre_cmd.eq_ignore_ascii_case("ls") || pre_cmd.eq_ignore_ascii_case("dir") {
            let path = extract_path(line);
            // Send a list file request to the client
            let mut connections = connections.lock().await;
            let message = ListFilesMessage {
                path,
                only_directories: true,
            };
            let packet = Packet::ListFiles(message);
            // Send the packet and wait for the response
            if let Ok(packet) = connections.send_message(&packet).await {
                //If the response is a list files response, add the files to the matches
                if let Packet::ListFilesResponse(response) = packet {
                    for file in response.files {
                        matches.push(Pair {
                            display: file.clone(),
                            replacement: file,
                        });
                    }
                }
            }
        }

        matches
    });

    Ok((start, res))
}

// Extract the path from a line
fn extract_path(line: &str) -> String {
    let command = line.trim();

    if let Some(path_part) = command.split_once(" ") {
        // Get the second part of the split to get the path
        let path_part = path_part.1.trim();

        // Check if the path is enclosed in quotes
        if (path_part.starts_with('"') && path_part.ends_with('"')) ||
            (path_part.starts_with('\'') && path_part.ends_with('\'')) {
            // Remove the enclosing quotes and return the inner part
            path_part[1..path_part.len() - 1].to_string()
        } else {
            // Return the path as it is (no quotes)
            path_part.to_string()
        }
    } else {
        // If there is no command or no space, return an empty string
        "".to_string()
    }
}