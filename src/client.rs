use serde::{Deserialize, Serialize};
use std::env;

pub fn get_base_url() -> String {
    let url = env::var("SHADOW_URL").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("SHADOW_PORT").unwrap_or_else(|_| "9999".to_string());
    let mut api_path = env::var("SHADOW_API_PATH").unwrap_or_else(|_| "/api/v1/charon".to_string());

    if !api_path.starts_with('/') {
        api_path = format!("/{}", api_path);
    }

    if !port.is_empty() {
        format!("{}:{}{}", url, port, api_path)
    } else {
        format!("{}{}", url, api_path)
    }
}

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
    pub jitter_percent: i16
}

pub async fn fetch_ghosts() -> Result<Vec<Ghost>, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/ghosts", get_base_url());

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
    let url = format!("{}/ghosts/{}/tasks", get_base_url(), ghost_id);

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
    let url = format!("{}/ghosts/{}/task", get_base_url(), ghost_id);
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

pub async fn update_ghost_config(ghost_id: String, sleep: i64, jitter: i16) -> Result<String, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/ghosts/{}", get_base_url(), ghost_id);
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

pub async fn kill_ghost(ghost_id: String) -> Result<String, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/ghosts/{}/kill", get_base_url(), ghost_id);

    match client.post(&url).send().await {
        Ok(res) => {
            if res.status().is_success() {
                Ok("kill signal sent".to_string())
            } else {
                Err(format!("ERROR returned status {}", res.status()))
            }
        },
        Err(e) => Err(format!("ERROR {}", e))
    }
}
