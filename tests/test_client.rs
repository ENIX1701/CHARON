use charon::client::{C2Client, RealClient};
use charon::models::{
    GhostBuildRequest, GhostConfigUpdate, ReplayStartRequest, TaskRequest, TaskStatus,
};
use serial_test::serial;
use std::env;

#[tokio::test]
#[serial]
async fn test_fetch_ghosts() {
    let mut server = mockito::Server::new_async().await;
    setup_env(&server.url());

    let mock = server
        .mock("GET", "/ghosts")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"[{"id":"test_ghost_1","hostname":"test","os":"linux","last_seen":153}]"#)
        .create_async()
        .await;

    let client = RealClient::new();
    let result = client.fetch_ghosts().await;

    mock.assert_async().await;
    assert!(result.is_ok());

    let ghosts = result.unwrap();
    assert_eq!(ghosts.len(), 1);
    assert_eq!(ghosts[0].id, "test_ghost_1");
}

#[tokio::test]
#[serial]
async fn test_fetch_tasks() {
    let mut server = mockito::Server::new_async().await;
    setup_env(&server.url());

    let mock = server
        .mock("GET", "/ghosts/test_ghost_1/tasks")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"[{"id":"task_1","command":"whoami","args":"","status":"done","result":"root"}]"#,
        )
        .create_async()
        .await;

    let client = RealClient::new();
    let result = client.fetch_tasks("test_ghost_1").await;

    mock.assert_async().await;
    assert!(result.is_ok());

    let tasks = result.unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].command, "whoami");
    assert_eq!(tasks[0].status, TaskStatus::Done);
}

#[tokio::test]
#[serial]
async fn test_send_task() {
    let mut server = mockito::Server::new_async().await;
    setup_env(&server.url());

    let mock = server
        .mock("POST", "/ghosts/test_ghost_1/task")
        .match_body(mockito::Matcher::Json(serde_json::json!({
            "command": "exec",
            "args": "ls -la"
        })))
        .with_status(200)
        .create_async()
        .await;

    let client = RealClient::new();
    let request = TaskRequest {
        command: "exec".to_string(),
        args: "ls -la".to_string(),
    };
    let result = client.send_task("test_ghost_1", request).await;

    mock.assert_async().await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Task queued successfully");
}

#[tokio::test]
#[serial]
async fn test_update_config() {
    let mut server = mockito::Server::new_async().await;
    setup_env(&server.url());

    let mock = server
        .mock("POST", "/ghosts/test_ghost_1")
        .match_body(mockito::Matcher::Json(serde_json::json!({
            "sleep_interval": 60,
            "jitter_percent": 10
        })))
        .with_status(200)
        .create_async()
        .await;

    let client = RealClient::new();
    let config = GhostConfigUpdate {
        sleep_interval: 60,
        jitter_percent: 10,
    };
    let result = client.update_config("test_ghost_1", config).await;

    mock.assert_async().await;
    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_kill_ghost() {
    let mut server = mockito::Server::new_async().await;
    setup_env(&server.url());

    let mock = server
        .mock("POST", "/ghosts/test_ghost_1/kill")
        .with_status(200)
        .create_async()
        .await;

    let client = RealClient::new();
    let result = client.kill_ghost("test_ghost_1").await;

    mock.assert_async().await;
    assert!(result.is_ok());
}

fn setup_env(url: &str) {
    let parsed_url = reqwest::Url::parse(&url).unwrap();

    unsafe {
        env::set_var(
            "SHADOW_URL",
            format!(
                "{}://{}",
                parsed_url.scheme(),
                parsed_url.host_str().unwrap()
            ),
        );
        env::set_var("SHADOW_PORT", parsed_url.port().unwrap().to_string());
        env::set_var("SHADOW_API_PATH", "");
    }
}

#[tokio::test]
#[serial]
async fn test_request_build() {
    let mut server = mockito::Server::new_async().await;
    setup_env(&server.url());

    let mock = server
        .mock("POST", "/build")
        .with_status(200)
        .with_body(r#""/downloads/Ghost""#)
        .create_async()
        .await;

    let client = RealClient::new();
    let req = GhostBuildRequest {
        target_url: "127.0.0.1".into(),
        target_port: "9999".into(),
        enable_debug: true,
        scenario_mode: "NONE".into(),
        impact_level: "TEST".into(),
        enable_persistence: false,
        persist_runcontrol: false,
        persist_service: false,
        persist_cron: false,
        enable_impact: false,
        impact_encrypt: false,
        encryption_algo: "XOR".into(),
        impact_wipe: false,
        enable_exfil: false,
        exfil_http: false,
        exfil_dns: false,
    };

    let result = client.request_build(req).await;
    mock.assert_async().await;
    assert!(result.is_ok());
    assert!(result.unwrap().contains("/downloads/Ghost"));
}

#[tokio::test]
#[serial]
async fn test_fetch_loot_list() {
    let mut server = mockito::Server::new_async().await;
    setup_env(&server.url());

    let mock = server
        .mock("GET", "/loot")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"["passwords.txt", "id_rsa"]"#)
        .create_async()
        .await;

    let client = RealClient::new();
    let result = client.fetch_loot_list().await;

    mock.assert_async().await;
    assert!(result.is_ok());
    let files = result.unwrap();
    assert_eq!(files.len(), 2);
    assert_eq!(files[0], "passwords.txt");
}

#[tokio::test]
#[serial]
async fn test_download_loot() {
    let mut server = mockito::Server::new_async().await;
    setup_env(&server.url());

    let mock = server
        .mock("GET", "/loot/download/test.txt")
        .with_status(200)
        .with_body("loot data")
        .create_async()
        .await;

    let client = RealClient::new();
    let dest = std::env::temp_dir().join("test.txt");
    let result = client
        .download_loot("test.txt", dest.to_str().unwrap())
        .await;

    mock.assert_async().await;
    assert!(result.is_ok());

    let _ = std::fs::remove_file(dest);
}

#[tokio::test]
#[serial]
async fn test_fetch_replay_status() {
    let mut server = mockito::Server::new_async().await;
    setup_env(&server.url());

    let mock = server
        .mock("GET", "/replay")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "running": true,
            "current_scenario": "task_flow",
            "available_scenarios": ["idle_fleet", "task_flow", "loot_burst"],
            "replay_ghost_count": 2
        }"#,
        )
        .create_async()
        .await;

    let client = RealClient::new();
    let result = client.fetch_replay_status().await;

    mock.assert_async().await;
    assert!(result.is_ok());

    let status = result.unwrap();
    assert!(status.running);
    assert_eq!(status.current_scenario.as_deref(), Some("task_flow"));
    assert_eq!(status.replay_ghost_count, 2);
}

#[tokio::test]
#[serial]
async fn test_start_replay() {
    let mut server = mockito::Server::new_async().await;
    setup_env(&server.url());

    let mock = server
        .mock("POST", "/replay/start")
        .match_body(mockito::Matcher::Json(serde_json::json!({
            "scenario": "idle_fleet"
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "running": true,
            "current_scenario": "idle_fleet",
            "available_scenarios": ["idle_fleet", "task_flow", "loot_burst"],
            "replay_ghost_count": 3
        }"#,
        )
        .create_async()
        .await;

    let client = RealClient::new();
    let result = client
        .start_replay(ReplayStartRequest {
            scenario: "idle_fleet".to_string(),
        })
        .await;

    mock.assert_async().await;
    assert!(result.is_ok());

    let status = result.unwrap();
    assert!(status.running);
    assert_eq!(status.current_scenario.as_deref(), Some("idle_fleet"));
    assert_eq!(status.replay_ghost_count, 3);
}

#[tokio::test]
#[serial]
async fn test_stop_replay() {
    let mut server = mockito::Server::new_async().await;
    setup_env(&server.url());

    let mock = server
        .mock("POST", "/replay/stop")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "running": false,
            "current_scenario": "task_flow",
            "available_scenarios": ["idle_fleet", "task_flow", "loot_burst"],
            "replay_ghost_count": 2
        }"#,
        )
        .create_async()
        .await;

    let client = RealClient::new();
    let result = client.stop_replay().await;

    mock.assert_async().await;
    assert!(result.is_ok());

    let status = result.unwrap();
    assert!(!status.running);
    assert_eq!(status.current_scenario.as_deref(), Some("task_flow"));
}

#[tokio::test]
#[serial]
async fn test_reset_replay() {
    let mut server = mockito::Server::new_async().await;
    setup_env(&server.url());

    let mock = server
        .mock("POST", "/replay/reset")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "running": false,
            "current_scenario": null,
            "available_scenarios": ["idle_fleet", "task_flow", "loot_burst"],
            "replay_ghost_count": 0
        }"#,
        )
        .create_async()
        .await;

    let client = RealClient::new();
    let result = client.reset_replay().await;

    mock.assert_async().await;
    assert!(result.is_ok());

    let status = result.unwrap();
    assert!(!status.running);
    assert!(status.current_scenario.is_none());
    assert_eq!(status.replay_ghost_count, 0);
}
