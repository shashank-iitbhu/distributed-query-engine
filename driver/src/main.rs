use common::Message;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

// This function handles the conversation with a single engine.
async fn handle_engine(mut stream: TcpStream, task_queue: Arc<Mutex<Vec<String>>>) {
    let addr = stream.peer_addr().expect("Failed to get peer address");
    println!("[Driver] Engine connected: {}", addr);

    loop {
        // Wait for the engine to request a task.
        let mut len_bytes = [0u8; 4];
        if stream.read_exact(&mut len_bytes).await.is_err() {
            println!("[Driver] Engine {} disconnected.", addr);
            break;
        }
        let len = u32::from_be_bytes(len_bytes) as usize;
        let mut buffer = vec![0u8; len];
        stream.read_exact(&mut buffer).await.expect("Failed to read message");

        let msg: Message = bincode::deserialize(&buffer).expect("Failed to deserialize message");

        if let Message::RequestTask = msg {
            // Lock the queue to safely access it.
            let mut queue = task_queue.lock().await;
            let response = if let Some(task) = queue.pop() {
                println!("[Driver] Assigning task '{}' to {}", task, addr);
                Message::AssignTask(task)
            } else {
                println!("[Driver] No more tasks. Telling {} to shut down.", addr);
                Message::NoMoreTasks
            };
            
            let serialized_response = bincode::serialize(&response).unwrap();
            let len_bytes = (serialized_response.len() as u32).to_be_bytes();
            stream.write_all(&len_bytes).await.unwrap();
            stream.write_all(&serialized_response).await.unwrap();
        }
        // In a real implementation, you would handle TaskResult messages here.
    }
}

#[tokio::main]
async fn main() {
    // A simple list of tasks (filenames).
    let tasks = vec![
        "file1.csv".to_string(), "file2.csv".to_string(), "file3.csv".to_string(),
        "file4.csv".to_string(), "file5.csv".to_string(), "file6.csv".to_string(),
        "file7.csv".to_string(), "file8.csv".to_string(),
    ];

    // Create a thread-safe, shareable task queue.
    let task_queue = Arc::new(Mutex::new(tasks));

    let listener = TcpListener::bind("127.0.0.1:8000").await.unwrap();
    println!("[Driver] Server listening on port 8000...");

    // Accept incoming connections in a loop.
    while let Ok((stream, _)) = listener.accept().await {
        // For each connection, spawn a new asynchronous task.
        // Clone the Arc to give the new task its own reference to the queue.
        let task_queue_clone = Arc::clone(&task_queue);
        tokio::spawn(handle_engine(stream, task_queue_clone));
    }
}