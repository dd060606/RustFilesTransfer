use crate::connections::Connections;
use rustyline::completion::{extract_word, Completer, FilenameCompleter, Pair};
use rustyline::error::ReadlineError;
use rustyline::Context;
use rustyline_derive::{Helper, Highlighter, Hinter, Validator};
use std::sync::Arc;
use tokio::sync::Mutex;

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
        matches
    });

    Ok((start, res))
}
