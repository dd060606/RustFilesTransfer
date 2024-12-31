mod commands;

use std::collections::HashSet;
use std::time::Duration;
use rustyline::completion::{extract_word, Completer, FilenameCompleter, Pair};
use rustyline::error::ReadlineError;
use rustyline::{CompletionType, Config, Context, EditMode, Editor, ExternalPrinter};
use rustyline_derive::{Helper, Highlighter, Hinter, Validator};
use tokio::task;

const DEFAULT_BREAK_CHARS: [char; 3] = [' ', '\t', '\n'];

#[derive(Hash, Debug, PartialEq, Eq)]
struct Command {
    cmd: String,
    pre_cmd: String,
}

impl Command {
    fn new(cmd: &str, pre_cmd: &str) -> Self {
        Self {
            cmd: cmd.into(),
            pre_cmd: pre_cmd.into(),
        }
    }
}
struct CommandCompleter {
    cmds: HashSet<Command>,
}

impl CommandCompleter {
    pub fn find_matches(&self, line: &str, pos: usize) -> rustyline::Result<(usize, Vec<Pair>)> {
        let (start, word) = extract_word(line, pos, None, | c|      DEFAULT_BREAK_CHARS.contains(   &c));
        let pre_cmd = line[..start].trim();

        let matches = self
            .cmds
            .iter()
            .filter_map(|hint| {
                if hint.cmd.starts_with(word) && pre_cmd == &hint.pre_cmd {
                    let mut replacement = hint.cmd.clone();
                    replacement += " ";
                    Some(Pair {
                        display: hint.cmd.to_string(),
                        replacement: replacement.to_string(),
                    })
                } else {
                    None
                }
            })
            .collect();
        Ok((start, matches))
    }
}

#[derive(Helper, Hinter, Validator, Highlighter)]
struct MyHelper {
    file_completer: FilenameCompleter,
    cmd_completer: CommandCompleter,
}

impl Completer for MyHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        match self.cmd_completer.find_matches(line, pos) {
            Ok((start, matches)) => {
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

fn cmd_sets() -> HashSet<Command> {
    let mut set = HashSet::new();
    set.insert(Command::new("helper", "about"));
    set.insert(Command::new("hinter", "about"));
    set.insert(Command::new("highlighter", "about"));
    set.insert(Command::new("validator", "about"));
    set.insert(Command::new("completer", "about"));

    set.insert(Command::new("release", "dev"));
    set.insert(Command::new("deploy", "dev"));
    set.insert(Command::new("compile", "dev"));
    set.insert(Command::new("test", "dev"));

    set.insert(Command::new("history", ""));
    set.insert(Command::new("about", ""));
    set.insert(Command::new("help", ""));
    set.insert(Command::new("dev", ""));
    set
}
#[tokio::main]
async fn main() {

    let config = Config::builder()
        .history_ignore_space(true)
        .completion_type(CompletionType::List)
        .edit_mode(EditMode::Emacs)
        .build();
    let h = MyHelper {
        file_completer: FilenameCompleter::new(),
        cmd_completer: CommandCompleter { cmds: cmd_sets() },
    };
    // Create a new instance of the default editor
    let mut rl = Editor::with_config(config).expect("Failed to create editor");
    rl.set_helper(Some(h));
    let mut printer = rl.create_external_printer().expect("Failed to create external printer");
    //Simulate a background task that prints a message in the console
    task::spawn(async move {
        let mut i = 0usize;
        loop {
            printer
                .print(format!("External message #{i}"))
                .expect("External print failure");
            let wait_ms = 5000;
tokio::time::sleep(Duration::from_millis(wait_ms as u64)).await;
            i += 1;
        }
    });

    // Command loop
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());
                println!("Line: {}", line);
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
    }
}