# Distributed Query Engine

## Requirements
- **Rust** (version 1.70 or newer recommended)
- **Python 3**

## Setup & Usage
1. **Build the Rust binaries:**
   ```sh
   cargo build --release
   ```
2. **Run the test automation:**
   ```sh
   python3 test_runner.py
   ```

## Project Overview
This project demonstrates a distributed sorting system using a dynamic worker queue architecture. The core idea is to split a large dataset (student rankings in CSV files) across multiple worker nodes (engines), each of which sorts its chunk independently. A central driver coordinates the process, distributes tasks dynamically to available workers, and merges the sorted results efficiently.

- **Driver:** Discovers CSV files, manages a dynamic queue of tasks, assigns work to engines as they connect, and merges sorted results using a k-way merge.
- **Engine:** Connects to the driver, requests tasks, sorts its assigned chunk, and sends the sorted data back.

This approach allows for flexible scaling: you can run as many engine workers as you like, and tasks are distributed dynamically based on worker availability.

## Result
After running the test runner, you should see a summary in the terminal showing the success of each step:

![Test Runner Output](Screenshot 2025-09-30 at 8.45.42 PM.png)


