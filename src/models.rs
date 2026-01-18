use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Pending,
    Sent,
    Running,
    Success,
    Failed,
    #[serde(other)]
    Unknown
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskStatus::Pending => write!(f, "PENDING"),
            TaskStatus::Sent => write!(f, "SENT"),
            TaskStatus::Running => write!(f, "RUNNING"),
            TaskStatus::Success => write!(f, "SUCCESS"),
            TaskStatus::Failed => write!(f, "FAILED"),
            TaskStatus::Unknown => write!(f, "UNKNOWN")
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ghost {
    pub id: String,
    pub hostname: String,
    pub os: String,
    pub last_seen: i64
}

impl Ghost {
    pub fn is_active(&self, current_time: i64, timeout_seconds: i64) -> bool {
        (current_time - self.last_seen) < timeout_seconds
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub command: String,
    pub args: String,
    pub status: TaskStatus,
    pub result: Option<String>
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskRequest {
    pub command: String,
    pub args: String
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GhostConfigUpdate {
    pub sleep_interval: i64,
    pub jitter_percent: i16
}
