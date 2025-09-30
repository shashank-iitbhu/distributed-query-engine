use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // The driver port should be the first argument, its own port the second.
    let args: Vec<String> = env::args().collect();
    let driver_port = &args[1];
    let engine_port = &args[2];
    let driver_addr = format!("127.0.0.1:{}", driver_port);

    println!("[Engine-{}] Connecting to driver at {}", engine_port, driver_addr);
    let mut stream = TcpStream::connect(driver_addr).await?;
    println!("[Engine-{}] Connected to driver.", engine_port);

    // Send a message to the driver.
    let message = format!("Hello from Engine listening on port {}", engine_port);
    stream.write_all(message.as_bytes()).await?;
    println!("[Engine-{}] Sent message to driver.", engine_port);

    // Wait for the driver's response.
    let mut buf = vec![0; 1024];
    let n = stream.read(&mut buf).await?;
    let received_msg = std::str::from_utf8(&buf[..n])?;
    println!("[Engine-{}] Received response from driver: '{}'", engine_port, received_msg);

    Ok(())
}