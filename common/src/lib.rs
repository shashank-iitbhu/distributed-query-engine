use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

// student's ranking information
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct StudentRanking {
    pub student_id: String,
    pub batch_year: i32,
    pub university_ranking: i32,
    pub batch_ranking: i32,
}

// This enum defines every possible message in our protocol.
#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    RequestTask,
    AssignTask(String),
    TaskResult(Vec<StudentRanking>),
    NoMoreTasks,
}

impl Ord for StudentRanking {
    fn cmp(&self, other: &Self) -> Ordering {
        self.batch_year
            .cmp(&other.batch_year)
            .then_with(|| self.university_ranking.cmp(&other.university_ranking))
            .then_with(|| self.batch_ranking.cmp(&other.batch_ranking))
    }
}

impl PartialOrd for StudentRanking {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
