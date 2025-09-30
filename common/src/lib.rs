use serde::{Deserialize, Serialize};

// This struct will eventually hold our student data.
// For now, it's just a placeholder.
#[derive(Serialize, Deserialize, Debug)]
pub struct StudentRanking {
    pub student_id: String,
}

// This enum defines every possible message in our protocol.
#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    RequestTask,
    AssignTask(String), // The String will be the filename.
    TaskResult(String), // A simple confirmation string for now.
    NoMoreTasks,
}