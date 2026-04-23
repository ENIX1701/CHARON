use charon::models::{Ghost, TaskStatus};

#[test]
fn test_task_status_display() {
    assert_eq!(format!("{}", TaskStatus::Pending), "PENDING");
    assert_eq!(format!("{}", TaskStatus::Sent), "SENT");
    assert_eq!(format!("{}", TaskStatus::Running), "RUNNING");
    assert_eq!(format!("{}", TaskStatus::Success), "SUCCESS");
    assert_eq!(format!("{}", TaskStatus::Failed), "FAILED");
    assert_eq!(format!("{}", TaskStatus::Unknown), "UNKNOWN");
}

#[test]
fn test_ghost_is_active() {
    let ghost = Ghost {
        id: "test_ghost".to_string(),
        hostname: "host".to_string(),
        os: "linux".to_string(),
        last_seen: 1000,
        is_replay: false,
    };

    assert!(ghost.is_active(1050, 60));
    assert!(!ghost.is_active(1100, 60));
}

#[test]
fn test_ghost_deserialize_defaults_is_replay() {
    let ghost: Ghost = serde_json::from_str(
        r#"{
            "id": "test_ghost",
            "hostname": "host",
            "os": "linux",
            "last_seen": 1000
        }"#,
    )
    .unwrap();

    assert!(!ghost.is_replay);
}

#[test]
fn test_task_status_done_deserializes() {
    let status: TaskStatus = serde_json::from_str(r#""done""#).unwrap();
    assert_eq!(status, TaskStatus::Done);
}
