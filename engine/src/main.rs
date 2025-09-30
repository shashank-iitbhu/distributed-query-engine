use common::Message;
use std::env;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use std::time::Duration;

async fn run_engine(port: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let driver_addr = "127.0.0.1:8000";
    let mut stream: Option<TcpStream> = None;
    // retry logic to connect to driver
    for _ in 0..5 {
        match TcpStream::connect(driver_addr).await {
            Ok(s) => {
                stream = Some(s);
                break;
            }
            Err(_) => {
                // Wait for 1000 milliseconds before trying again.
                tokio::time::sleep(Duration::from_millis(1000)).await;
            }
        }
    }

    let mut stream = match stream {
        Some(s) => s,
        None => {
            return Err(
                format!("[Engine {}] Failed to connect to driver after multiple retries.", port).into(),
            )
        }
    };

    loop {
        println!("[Engine {}] Requesting a task.", port);
        let request = Message::RequestTask;
        let serialized_request = bincode::serialize(&request)?;
        let len_bytes = (serialized_request.len() as u32).to_be_bytes();
        stream.write_all(&len_bytes).await?;
        stream.write_all(&serialized_request).await?;

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
            Message::NoMoreTasks => {
                println!("[Engine {}] No more tasks. Shutting down.", port);
                break;
            }
            Message::AssignTask(task) => {
                let file_path = format!("sample_dataset/student_rankings/{}", task);
                println!("[Engine {}] Received task: {}. Reading file...", port, task);

                let mut rdr = csv::ReaderBuilder::new().has_headers(false).from_path(&file_path)?;
                let mut records: Vec<common::StudentRanking> = rdr.deserialize().collect::<Result<_, _>>()?;
                println!("[Engine {}] Read {} records from {}. Sorting...", port, records.len(), task);

                merge_sort(&mut records);
                println!("[Engine {}] Finished sorting {}.", port, task);

                let result_msg = Message::TaskResult(records);
                let serialized_result = bincode::serialize(&result_msg)?;

                let len_bytes = (serialized_result.len() as u32).to_be_bytes();
                stream.write_all(&len_bytes).await?;
                stream.write_all(&serialized_result).await?;

                println!("[Engine {}] Sent sorted result for {} back to driver.", port, task);
            }
            _ => {
                eprintln!("[Engine {}] Received an unexpected message.", port);
            }
        }
    }
    Ok(())
}

/// In-place merge sort implementation for StudentRanking
fn merge_sort(slice: &mut [common::StudentRanking]) {
    let len = slice.len();
    if len > 1 {
        let mid = len / 2;
        merge_sort(&mut slice[0..mid]);
        merge_sort(&mut slice[mid..len]);
        merge(slice, mid);
    }
}

// Merges two sorted halves of the slice
fn merge(slice: &mut [common::StudentRanking], mid: usize) {
    let left_half = slice[..mid].to_vec();
    let right_half = slice[mid..].to_vec();

    let mut i = 0; // Pointer for the left half
    let mut j = 0; // Pointer for the right half
    let mut k = 0; // Pointer for the main slice

    while i < left_half.len() && j < right_half.len() {
        if left_half[i] <= right_half[j] {
            slice[k] = left_half[i].clone();
            i += 1;
        } else {
            slice[k] = right_half[j].clone();
            j += 1;
        }
        k += 1;
    }

    while i < left_half.len() {
        slice[k] = left_half[i].clone();
        i += 1;
        k += 1;
    }

    while j < right_half.len() {
        slice[k] = right_half[j].clone();
        j += 1;
        k += 1;
    }
}

#[tokio::main]
async fn main() {
    let port = match env::args().nth(1) {
        Some(p) => p,
        None => {
            eprintln!("Usage: engine <port>");
            return;
        }
    };

    println!("[Engine {}] Starting...", port);

    // Run a single engine task for the given port.
    if let Err(e) = run_engine(port).await {
        eprintln!("Engine failed with error: {}", e);
    }
}