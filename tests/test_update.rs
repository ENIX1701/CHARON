use charon::action::Action;
use charon::models::Ghost;
use charon::state::{AppState, CurrentScreen, ConfigField, BuilderField};
use charon::update::{update, Command};

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
        last_seen: 0
    });
    app.dashboard.table_state.select(Some(0));

    let command = update(&mut app, Action::Enter);

    assert_eq!(app.current_screen, CurrentScreen::Terminal);
    assert_eq!(app.terminal.active_ghost_id, Some("test_ghost_1".to_string()));
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
        },
        _ => panic!("Expected SendTask command")
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
        last_seen: 0
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
        },
        _ => panic!("Expected UpdateGhostConfig command")
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
fn test_receive_ghosts_updates_state() {
    let mut app = AppState::default();
    let ghosts = vec![Ghost {
        id: "test_ghost_1".to_string(),
        hostname: "hostname".to_string(),
        os: "linux".to_string(),
        last_seen: 0
    }];

    update(&mut app, Action::ReceiveGhosts(Ok(ghosts)));

    assert_eq!(app.dashboard.ghosts.len(), 1);
    assert_eq!(app.dashboard.ghosts[0].id, "test_ghost_1");
}