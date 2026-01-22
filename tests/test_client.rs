use serial_test::serial;
use charon::client::{C2Client, RealClient};
use charon::models::{TaskRequest, GhostConfigUpdate};
use std::env;

// TODO: fix this test, there's error with parsing the response JSON body?
// the error was a missing quote. im gonna kms
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
async fn text_fetch_tasks() {
    let mut server = mockito::Server::new_async().await;
    setup_env(&server.url());

    let mock = server
        .mock("GET", "/ghosts/test_ghost_1/tasks")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"[{"id":"task_1","command":"whoami","args":"","status":"success","result":"root"}]"#)
        .create_async()
        .await;

    let client = RealClient::new();
    let result = client.fetch_tasks("test_ghost_1").await;

    mock.assert_async().await;
    assert!(result.is_ok());

    let tasks = result.unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].command, "whoami");
}

#[tokio::test]
#[serial]
async fn text_send_task() {
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
    let request = TaskRequest { command: "exec".to_string(), args: "ls -la".to_string() };
    let result = client.send_task("test_ghost_1", request).await;

    mock.assert_async().await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Task queued successfully");
}

#[tokio::test]
#[serial]
async fn text_update_config() {
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
    let config = GhostConfigUpdate { sleep_interval: 60, jitter_percent: 10 };
    let result = client.update_config("test_ghost_1", config).await;

    mock.assert_async().await;
    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn text_kill_ghost() {
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
            format!("{}://{}", parsed_url.scheme(), parsed_url.host_str().unwrap())
        );
        env::set_var("SHADOW_PORT", parsed_url.port().unwrap().to_string());
        env::set_var("SHADOW_API_PATH", "");
    }
}
