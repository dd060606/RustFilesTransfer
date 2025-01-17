use regex::Regex;
use std::io::{stdout, Write};

//Extract paths from a string
pub fn extract_paths(input: &str) -> Vec<&str> {
    // Create a regex pattern that matches:
    // 1. Quoted paths (both single and double quotes)
    // 2. Unquoted paths (containing typical path characters)
    let pattern = r#"(?x)
        # Match quoted paths (both single and double quotes)
        (?:"([^"]+)"|'([^']+)')  # Capture paths in quotes
        |
        # Match unquoted paths
        ([A-Za-z]:[\\/][^\s"']+  # Windows paths starting with drive letter
        |
        /[^\s"']+)               # Unix-style paths starting with /
    "#;

    let re = Regex::new(pattern).unwrap();
    let mut paths = Vec::new();

    for cap in re.captures_iter(input) {
        // Check each capture group and add the non-empty one
        if let Some(quoted_double) = cap.get(1) {
            paths.push(quoted_double.as_str());
        } else if let Some(quoted_single) = cap.get(2) {
            paths.push(quoted_single.as_str());
        } else if let Some(unquoted) = cap.get(3) {
            paths.push(unquoted.as_str());
        }
    }

    paths
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
