use std::env;
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::sleep;

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

async fn run_tcp_client(addr: String) {
    loop {
        match TcpStream::connect(&addr).await {
            Ok(mut stream) => {
                println!("Connected to file server!");

                // Example of reading and writing to the stream
                let mut buffer = [0u8; 1024];
                if let Ok(n) = stream.read(&mut buffer).await {
                    if n == 0 {
                        // End of stream
                        println!("Connection closed by server.");
                    } else {
                        println!("Received: {:?}", &buffer[..n]);

                        // Example of writing to the stream
                        if let Err(e) = stream.write_all(b"Hello, server!").await {
                            eprintln!("Write failed: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to connect to file server: {}. Retrying in 10 seconds...", e);
                sleep(Duration::from_secs(10)).await;
            }
        }
    }
}