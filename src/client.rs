use serde::{Deserialize, Serialize};

pub const BASE_URL: &str = "http://127.0.0.1:9999/api/v1/charon";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ghost {
    pub id: String,
    pub hostname: String,
    pub os: String,
    pub last_seen: i64
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub command: String,
    pub args: String,
    pub status: String,
    pub result: Option<String>
}

#[derive(Serialize)]
pub struct TaskRequest {
    pub command: String,
    pub args: String
}

#[derive(Serialize)]
pub struct GhostConfigUpdate {
    pub sleep_interval: i64,
    pub jitter_percent: u8
}

pub async fn fetch_ghosts() -> Result<Vec<Ghost>, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/ghosts", BASE_URL);

    match client.get(&url).send().await {
        Ok(res) => match res.json::<Vec<Ghost>>().await {
            Ok(ghosts) => Ok(ghosts),
            Err(_) => Err("failed to parse JSON".to_string())
        },
        Err(_) => Err("connection failed".to_string())
    }
}

pub async fn fetch_tasks(ghost_id: String) -> Result<Vec<Task>, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/ghosts/{}/tasks", BASE_URL, ghost_id);

    match client.get(&url).send().await {
        Ok(res) => match res.json::<Vec<Task>>().await {
            Ok(tasks) => Ok(tasks),
            Err(_) => Err("failed to parse JSON".to_string())
        },
        Err(_) => Err("connection failed".to_string())
    }
}

pub async fn send_task(ghost_id: String, command: String, args: String) -> Result<String, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/ghosts/{}/task", BASE_URL, ghost_id);
    let body = TaskRequest { command, args };

    match client.post(&url).json(&body).send().await {
        Ok(res) => {
            if res.status().is_success() {
                Ok("task queued successfully".to_string())
            } else {
                Err(format!("ERROR returned status {}", res.status()))
            }
        },
        Err(e) => Err(format!("ERROR {}", e))
    }
}

pub async fn update_ghost_config(ghost_id: String, sleep: i64, jitter: u8) -> Result<String, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/ghosts/{}/config", BASE_URL, ghost_id);
    let body = GhostConfigUpdate { sleep_interval: sleep, jitter_percent: jitter };

    match client.post(&url).json(&body).send().await {
        Ok(res) => {
            if res.status().is_success() {
                Ok("ghost config updated".to_string())
            } else {
                Err(format!("ERROR returned status {}", res.status()))
            }
        },
        Err(e) => Err(format!("ERROR {}", e))
    }
}
