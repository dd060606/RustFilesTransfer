#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env;

use crate::client::run_tcp_client;

mod client;
mod files;

#[tokio::main]
async fn main() {
    let args = env::args().collect::<Vec<String>>();
    if args.len() < 3 {
        eprintln!("Usage: {} <server ip> <server port>", args[0]);
        return;
    }

    let server_address = format!("{}:{}", args[1], args[2]);
    run_tcp_client(server_address).await;
}
