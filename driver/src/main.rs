use common::{Message, StudentRanking};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::env;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

// Min-Heap.
#[derive(Debug, Clone, Eq, PartialEq)]
struct HeapItem {
    record: StudentRanking,
    chunk_index: usize,
    element_index: usize,
}

// behave like a Min-Heap for our custom struct
impl Ord for HeapItem {
    fn cmp(&self, other: &Self) -> Ordering {
        other.record.cmp(&self.record)
    }
}

impl PartialOrd for HeapItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// K-way merge function to merge sorted chunks from different engines
fn k_way_merge(chunks: Vec<Vec<StudentRanking>>) -> Vec<StudentRanking> {
    let k = chunks.len();
    let mut heap = BinaryHeap::new();
    let mut final_sorted_list = Vec::new();

    for i in 0..k {
        if !chunks[i].is_empty() {
            heap.push(HeapItem {
                record: chunks[i][0].clone(),
                chunk_index: i,
                element_index: 0,
            });
        }
    }

    while let Some(smallest_item) = heap.pop() {
        final_sorted_list.push(smallest_item.record);

        let next_element_index = smallest_item.element_index + 1;
        // If the chunk has more elements, push the next one onto the heap.
        if next_element_index < chunks[smallest_item.chunk_index].len() {
            heap.push(HeapItem {
                record: chunks[smallest_item.chunk_index][next_element_index].clone(),
                chunk_index: smallest_item.chunk_index,
                element_index: next_element_index,
            });
        }
    }

    final_sorted_list
}

fn write_output_file(sorted_records: Vec<StudentRanking>) -> std::io::Result<()> {
    let file = File::create("output.txt")?;
    let mut writer = BufWriter::new(file);

    for record in sorted_records.iter() {
        writeln!(writer, "{}", record.student_id)?;
    }
    Ok(())
}

// Handles communication with a single engine worker over TCP
async fn handle_engine(
    mut stream: TcpStream,
    task_queue: Arc<Mutex<Vec<String>>>,
    results: Arc<Mutex<Vec<Vec<StudentRanking>>>>,
) {
    let addr = stream.peer_addr().expect("Failed to get peer address");
    println!("[Driver] Engine connected: {}", addr);

    loop {
        let mut len_bytes = [0u8; 4];
        if stream.read_exact(&mut len_bytes).await.is_err() {
            break; // Engine disconnected
        }
        let len = u32::from_be_bytes(len_bytes) as usize;
        if len == 0 {
            break;
        }

        let mut buffer = vec![0u8; len];
        stream
            .read_exact(&mut buffer)
            .await
            .expect("Failed to read message");
        let msg: Message = bincode::deserialize(&buffer).expect("Failed to deserialize message");

        match msg {
            Message::RequestTask => {
                let mut queue = task_queue.lock().await;
                let response = if let Some(task) = queue.pop() {
                    Message::AssignTask(task)
                } else {
                    Message::NoMoreTasks
                };

                let serialized_response = bincode::serialize(&response).unwrap();
                let len_bytes = (serialized_response.len() as u32).to_be_bytes();
                stream.write_all(&len_bytes).await.unwrap();
                stream.write_all(&serialized_response).await.unwrap();
            }
            Message::TaskResult(sorted_data) => {
                // println!("[Driver] Received a sorted chunk of size {} from {}", sorted_data.len(), addr);
                let mut results_vec = results.lock().await;
                results_vec.push(sorted_data);
            }
            _ => {
                eprintln!("[Driver] Received an unexpected message from {}", addr);
            }
        }
    }
    println!("[Driver] Engine {} disconnected.", addr);
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let engine_ports = &args[1..];
    println!(
        "[Driver] Acknowledging engine ports to be used: {:?}",
        engine_ports
    );

    let mut tasks = Vec::new();
    let dir_path = "sample_dataset/student_rankings";
    let paths = fs::read_dir(dir_path).expect("Failed to read sample_dataset directory");

    for path_entry in paths {
        let path = path_entry.expect("Failed to get path entry").path();
        if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
            if filename.ends_with(".csv") {
                tasks.push(filename.to_string());
            }
        }
    }
    let total_tasks = tasks.len();
    if total_tasks == 0 {
        println!("[Driver] No tasks found. Exiting.");
        return;
    }
    println!("[Driver] Found {} tasks to distribute.", total_tasks);

    let task_queue = Arc::new(Mutex::new(tasks));
    let results = Arc::new(Mutex::new(Vec::new()));

    let listener = TcpListener::bind("127.0.0.1:8000").await.unwrap();
    println!("[Driver] Server listening on port 8000...");

    // Spawn the connection listener as a background task.
    let task_queue_clone = Arc::clone(&task_queue);
    let results_clone = Arc::clone(&results);
    tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            let tqc = Arc::clone(&task_queue_clone);
            let rc = Arc::clone(&results_clone);
            tokio::spawn(handle_engine(stream, tqc, rc));
        }
    });

    // Wait in the main thread
    loop {
        tokio::time::sleep(Duration::from_millis(200)).await;
        let num_results = results.lock().await.len();
        if num_results == total_tasks {
            println!(
                "\n[Driver] All {} sorted chunks received. Starting final merge.",
                num_results
            );
            break;
        }
    }

    // Perform the final k-way merge
    let sorted_chunks = results.lock().await.clone();
    let final_result = k_way_merge(sorted_chunks);
    println!(
        "[Driver] Final merge complete. Total sorted records: {}",
        final_result.len()
    );

    write_output_file(final_result).expect("Failed to write output file");
    println!("[Driver] Successfully wrote to output.txt. Shutting down.");
}
