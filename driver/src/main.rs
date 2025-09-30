use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // We'll get the port numbers from the command line arguments.
    // For this simple test, we'll just listen on the first one.
    let args: Vec<String> = env::args().collect();
    let listen_port = &args[1];
    let listen_addr = format!("127.0.0.1:{}", listen_port);

    println!("[Driver] Listening for connections on {}", listen_addr);
    let listener = TcpListener::bind(listen_addr).await?;

    // Wait for an engine to connect.
    let (mut socket, addr) = listener.accept().await?;
    println!("[Driver] Accepted connection from: {}", addr);

    // Create a buffer to read the engine's message.
    let mut buf = vec![0; 1024];
    let n = socket.read(&mut buf).await?;
    let received_msg = std::str::from_utf8(&buf[..n])?;
    println!("[Driver] Received message from engine: '{}'", received_msg);

    // Send a response back to the engine.
    let response = "Hello from Driver!";
    socket.write_all(response.as_bytes()).await?;
    println!("[Driver] Sent response to engine.");

    Ok(())
}