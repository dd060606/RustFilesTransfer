use std::env;
use std::io::Error;
use std::path::{Path, PathBuf};
use tokio::fs::read_dir;

pub async fn list_files(path_str: &String, only_directories: bool) -> Result<Vec<String>, Error> {
    let path = if path_str.is_empty() {
        // If no path is provided, use the current directory
        env::current_dir().unwrap_or_default()
    } else {
        let pathbuf = PathBuf::from(&path_str);
        // If the path does not exist return parent directory
        if !pathbuf.exists() {
            pathbuf.parent().unwrap_or(Path::new("/")).to_path_buf()
        } else {
            pathbuf
        }
    };
    // Handle the result of reading the directory
    match read_dir(path).await {
        Ok(mut entries) => {
            let mut files: Vec<String> = Vec::new();
            // Iterate over the entries in the directory
            while let Ok(Some(entry)) = entries.next_entry().await {
                let entry_path = entry.path();
                // Skip files if only directories are requested
                if only_directories && entry_path.is_file() {
                    continue;
                }
                files.push(entry_path.to_string_lossy().to_string());
            }
            Ok(files)
        }
        Err(err) => Err(err),
    }
}
