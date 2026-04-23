use charon::action::Action;
use charon::models::{Ghost, ReplayStatus};
use charon::state::{AppState, BuilderField, ConfigField, CurrentScreen};
use charon::update::{Command, update};

#[test]
fn test_update_quit() {
    let mut app = AppState::default();
    let command = update(&mut app, Action::Quit);
    assert_eq!(command, Some(Command::Quit));
}

#[test]
fn test_update_navigation_tabs() {
    let mut app = AppState::default();
    assert_eq!(app.current_screen, CurrentScreen::Dashboard);

    update(&mut app, Action::NextTab);
    assert_eq!(app.current_screen, CurrentScreen::Terminal);

    update(&mut app, Action::PrevTab);
    assert_eq!(app.current_screen, CurrentScreen::Dashboard);
}

#[test]
fn test_dashboard_enter_selects_ghost() {
    let mut app = AppState::default();
    app.dashboard.ghosts.push(Ghost {
        id: "test_ghost_1".to_string(),
        hostname: "hostname".to_string(),
        os: "linux".to_string(),
        last_seen: 0,
        is_replay: false,
    });
    app.dashboard.table_state.select(Some(0));

    let command = update(&mut app, Action::Enter);

    assert_eq!(app.current_screen, CurrentScreen::Terminal);
    assert_eq!(
        app.terminal.active_ghost_id,
        Some("test_ghost_1".to_string())
    );
    assert!(matches!(command, Some(Command::FetchTasks(id)) if id == "test_ghost_1"));
}

#[test]
fn test_terminal_input_execution() {
    let mut app = AppState::default();
    app.current_screen = CurrentScreen::Terminal;
    app.terminal.active_ghost_id = Some("test_ghost_1".to_string());

    app.terminal.input_mode = true;
    app.terminal.input_buffer = "ls -la".to_string();

    let command = update(&mut app, Action::Enter);

    match command {
        Some(Command::SendTask { ghost_id, req }) => {
            assert_eq!(ghost_id, "test_ghost_1");
            assert_eq!(req.command, "EXEC");
            assert_eq!(req.args, "ls -la");
        }
        _ => panic!("Expected SendTask command"),
    }

    assert!(app.terminal.input_buffer.is_empty());
}

#[test]
fn test_config_submit() {
    let mut app = AppState::default();
    app.dashboard.ghosts.push(Ghost {
        id: "test_ghost_1".to_string(),
        hostname: "hostname".to_string(),
        os: "linux".to_string(),
        last_seen: 0,
        is_replay: false,
    });
    app.dashboard.table_state.select(Some(0));

    app.current_screen = CurrentScreen::Config;

    app.config.sleep_input = "100".to_string();
    app.config.jitter_input = "20".to_string();
    app.config.selected_field = ConfigField::Submit;

    let command = update(&mut app, Action::Enter);

    match command {
        Some(Command::UpdateGhostConfig { ghost_id, config }) => {
            assert_eq!(ghost_id, "test_ghost_1");
            assert_eq!(config.sleep_interval, 100);
            assert_eq!(config.jitter_percent, 20);
        }
        _ => panic!("Expected UpdateGhostConfig command"),
    }
}

#[test]
fn test_builder_toggle_action() {
    let mut app = AppState::default();
    app.current_screen = CurrentScreen::Builder;
    app.builder.selected_field = BuilderField::EnableDebug;
    app.builder.enable_debug = true;

    update(&mut app, Action::ToggleBuilderSwitch);
    assert_eq!(app.builder.enable_debug, false);
}

#[test]
fn test_builder_start_build() {
    let mut app = AppState::default();
    app.current_screen = CurrentScreen::Builder;
    app.builder.selected_field = BuilderField::Submit;

    let command = update(&mut app, Action::StartBuild);

    assert!(matches!(command, Some(Command::BuildPayload { .. })));
    assert_eq!(app.builder.build_status_msg, "BUILDING...");
}

#[test]
fn test_receive_ghosts_updates_state_and_fetches_replay_status_on_dashboard() {
    let mut app = AppState::default();
    app.current_screen = CurrentScreen::Dashboard;

    let ghosts = vec![Ghost {
        id: "test_ghost_1".to_string(),
        hostname: "hostname".to_string(),
        os: "linux".to_string(),
        last_seen: 0,
        is_replay: false,
    }];

    let command = update(&mut app, Action::ReceiveGhosts(Ok(ghosts)));

    assert_eq!(app.dashboard.ghosts.len(), 1);
    assert_eq!(app.dashboard.ghosts[0].id, "test_ghost_1");
    assert_eq!(command, Some(Command::FetchReplayStatus));
}

#[test]
fn test_receive_ghosts_updates_state_without_replay_fetch_off_dashboard() {
    let mut app = AppState::default();
    app.current_screen = CurrentScreen::Terminal;

    let ghosts = vec![Ghost {
        id: "test_ghost_1".to_string(),
        hostname: "hostname".to_string(),
        os: "linux".to_string(),
        last_seen: 0,
        is_replay: false,
    }];

    let command = update(&mut app, Action::ReceiveGhosts(Ok(ghosts)));

    assert_eq!(app.dashboard.ghosts.len(), 1);
    assert_eq!(command, None);
}

#[test]
fn test_auto_refresh() {
    let mut app = AppState::default();

    app.current_screen = CurrentScreen::Dashboard;
    let cmd_1 = update(&mut app, Action::AutoRefresh);
    assert_eq!(cmd_1, Some(Command::FetchGhosts));

    app.current_screen = CurrentScreen::Terminal;
    app.terminal.active_ghost_id = Some("active_id".to_string());
    let cmd_2 = update(&mut app, Action::AutoRefresh);
    assert_eq!(cmd_2, Some(Command::FetchTasks("active_id".to_string())));
}

#[test]
fn test_dashboard_replay_next_scenario() {
    let mut app = AppState::default();
    app.current_screen = CurrentScreen::Dashboard;

    let command = update(&mut app, Action::ReplayNextScenario);

    assert_eq!(command, None);
    assert_eq!(app.dashboard.replay.selected_scenario(), "task_flow");
}

#[test]
fn test_dashboard_replay_start_stop_start_path() {
    let mut app = AppState::default();
    app.current_screen = CurrentScreen::Dashboard;
    app.dashboard.replay.running = false;
    app.dashboard.replay.selected_scenario_idx = 1;

    let command = update(&mut app, Action::ReplayStartStop);

    match command {
        Some(Command::StartReplay(req)) => {
            assert_eq!(req.scenario, "task_flow");
        }
        _ => panic!("Expected StartReplay command"),
    }
}

#[test]
fn test_dashboard_replay_start_stop_stop_path() {
    let mut app = AppState::default();
    app.current_screen = CurrentScreen::Dashboard;
    app.dashboard.replay.running = true;

    let command = update(&mut app, Action::ReplayStartStop);
    assert_eq!(command, Some(Command::StopReplay));
}

#[test]
fn test_dashboard_replay_reset() {
    let mut app = AppState::default();
    app.current_screen = CurrentScreen::Dashboard;

    let command = update(&mut app, Action::ReplayReset);
    assert_eq!(command, Some(Command::ResetReplay));
}

#[test]
fn test_receive_replay_status_updates_dashboard_state() {
    let mut app = AppState::default();

    let status = ReplayStatus {
        running: true,
        current_scenario: Some("loot_burst".to_string()),
        available_scenarios: vec![
            "idle_fleet".to_string(),
            "task_flow".to_string(),
            "loot_burst".to_string(),
        ],
        replay_ghost_count: 3,
    };

    let command = update(&mut app, Action::ReceiveReplayStatus(Ok(status)));
    assert_eq!(command, None);

    assert!(app.dashboard.replay.running);
    assert_eq!(
        app.dashboard.replay.current_scenario.as_deref(),
        Some("loot_burst")
    );
    assert_eq!(app.dashboard.replay.replay_ghost_count, 3);
    assert_eq!(app.dashboard.replay.selected_scenario(), "loot_burst");
}
