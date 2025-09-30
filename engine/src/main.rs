use common::Message;
use std::env;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use rand::Rng;

// The run_engine function now takes a port string to identify itself.
async fn run_engine(port: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // CORRECTED LINE: Connect to the local machine (localhost).
    let mut stream = TcpStream::connect("127.0.0.1:8000").await?;
    println!("[Engine {}] Connected to driver.", port);

    loop {
        // 1. Request a task.
        println!("[Engine {}] Requesting a task.", port);
        let request = Message::RequestTask;
        let serialized_request = bincode::serialize(&request)?;
        let len_bytes = (serialized_request.len() as u32).to_be_bytes();
        stream.write_all(&len_bytes).await?;
        stream.write_all(&serialized_request).await?;

        // 2. Wait for a response.
        let mut len_bytes = [0u8; 4];
        if stream.read_exact(&mut len_bytes).await.is_err() {
            println!("[Engine {}] Driver disconnected.", port);
            break;
        }
        let len = u32::from_be_bytes(len_bytes) as usize;
        let mut buffer = vec![0u8; len];
        stream.read_exact(&mut buffer).await?;

        let msg: Message = bincode::deserialize(&buffer)?;

        match msg {
            Message::AssignTask(task) => {
                println!("[Engine {}] Received task: {}", port, task);
                // Simulate doing work.
                let sleep_duration = Duration::from_secs(rand::thread_rng().gen_range(1..=5));
                tokio::time::sleep(sleep_duration).await;
                println!("[Engine {}] Finished task: {}", port, task);
            }
            Message::NoMoreTasks => {
                println!("[Engine {}] No more tasks. Shutting down.", port);
                break; // Exit the loop.
            }
            _ => {
                eprintln!("[Engine {}] Received an unexpected message.", port);
            }
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    // Collect all command-line arguments, skipping the program name.
    let ports: Vec<String> = env::args().skip(1).collect();

    if ports.is_empty() {
        eprintln!("Usage: cargo run --bin engine <port1> <port2> ...");
        return;
    }

    let mut handles = vec![];

    println!("Starting engine instances for ports: {:?}", ports);

    // Spawn an engine task for each port argument.
    for port in ports {
        handles.push(tokio::spawn(run_engine(port)));
    }

    // Wait for all engine tasks to complete.
    for handle in handles {
        handle.await.unwrap().unwrap();
    }
}