use regex::Regex;
use std::io::{stdout, Write};

//Utils
pub fn get_path_from_args(args: &Vec<String>) -> (String, String) {
    let args_str = args.join(" ");
    // Regex pattern to match arguments enclosed in double or single quotes, or standalone arguments
    let re = Regex::new(r#"'([^']*)'|"([^"]*)"|(\S+)"#).unwrap();

    let mut parsed_args: Vec<String> = Vec::new();

    for cap in re.captures_iter(&args_str) {
        if let Some(single_quoted) = cap.get(1) {
            // Argument was in single quotes
            parsed_args.push(single_quoted.as_str().to_string());
        } else if let Some(double_quoted) = cap.get(2) {
            // Argument was in double quotes
            parsed_args.push(double_quoted.as_str().to_string());
        } else if let Some(unquoted) = cap.get(3) {
            // Argument was not in quotes
            parsed_args.push(unquoted.as_str().to_string());
        }
    }

    // Check if there are at least two arguments
    if parsed_args.len() > 1 {
        (parsed_args[0].clone(), parsed_args[1].clone())
    } else {
        // If only one argument is provided, return it as both source and destination
        (parsed_args[0].clone(), parsed_args[0].clone())
    }
}

pub fn print_progress(current: u64, total: u64) {
    let percent = (current as f64 / total as f64) * 100.0;
    let progress = (percent / 2.0).round() as u64; // max 50 chars width

    // Clear the current line and print the new progress bar
    print!("\r[");
    for _ in 0..progress {
        print!("#");
    }
    for _ in progress..50 {
        print!(" ");
    }
    print!("] {:.2}%", percent);

    // Flush the output to ensure it prints immediately
    let _ = stdout().flush();
}
