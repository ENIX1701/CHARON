#[async_trait]
pub trait C2Client: Send + Sync {
    async fn fetch_ghosts(&self) -> Result<Vec<Ghost>, String>;
    async fn fetch_tasks(&self, ghost_id: &str) -> Result<Vec<Task>, String>;
    async fn send_task(&self, ghost_id: &str, req: TaskRequest) -> Result<String, String>;
    async fn update_config(&self, ghost_id: &str, config: GhostConfigUpdate) -> Result<String, String>;
    async fn kill_ghost(&self, ghost_id: &str) -> Result<String, String>;
}

pub struct RealClient {
    base_url: String,
    http: Client
}

impl RealClient {
    pub fn new() -> Self {
        let url = env::var("SHADOW_URL").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = env::var("SHADOW_PORT").unwrap_or_else(|_| "9999".to_string());
        let mut api_path = env::var("SHADOW_API_PATH").unwrap_or_else(|_| "/api/v1/charon".to_string());

        if !api_path.starts_with('/') {
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

        Self {
            base_url,
            http:: Client::new()
        }
    }
}

#[async_trait]
impl C2Client for RealClient {
    async fn fetch_ghosts(&self) -> Result<Vec<Ghost>, String> {
        let url = format!("{}/ghosts", self.base_url);
        self.http.get(&url).send().await
            .map_err(|_| "connection failed".to_string())?
            .json::<Vec<Ghost>>().await
            .map_err(|_| "failed to parse JSON".to_string())
    }

    async fn fetch_tasks(&self, ghost_id: &str) -> Result<Vec<Task>, String> {
        let url = format!("{}/ghosts/{}/tasks", self.base_url, ghost_id);
        self.http.get(&url).send().await
            .map_err(|_| "connection failed".to_string())?
            .json::<Vec<Task>>().await
            .map_err(|_| "failed to parse JSON".to_string())
    }

    async fn send_task(&self, ghost_id: &str, req: TaskRequest) -> Result<String, String> {
        let url = format!("{}/ghosts/{}/task", self.base_url, ghost_id);
        self.http.post(&url).json(&req).send().await
            .map_err(|e| format!("ERROR {}", e))?;

        if res.status().is_success() {
            Ok("task queued successfully".to_string())
        } else {
            Err(format!("ERROR returned status {}", res.status()))
        }
    }

    async fn update_config(&self, ghost_id: &str, config: GhostConfigUpdate) -> Result<String, String> {
        let url = format!("{}/ghosts/{}", self.base_url, ghost_id);
        self.http.post(&url).json(&req).send().await
            .map_err(|e| format!("ERROR {}", e))?;

        if res.status().is_success() {
            Ok("ghost config updated".to_string())
        } else {
            Err(format!("ERROR returned status {}", res.status()))
        }
    }

    async fn kill_ghost(&self, ghost_id: &str) -> Result<String, String> {
        let url = format!("{}/ghosts/{}/kill", self.base_url, ghost_id);
        self.http.post(&url).json(&req).send().await
            .map_err(|e| format!("ERROR {}", e))?;

        if res.status().is_success() {
            Ok("kill signal sent".to_string())
        } else {
            Err(format!("ERROR returned status {}", res.status()))
        }
    }
}
