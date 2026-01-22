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
        last_seen: 1000
    };

    assert!(ghost.is_active(1050, 60));
    assert!(!ghost.is_active(1100, 60));
}
