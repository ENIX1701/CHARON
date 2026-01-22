use charon::models::{Ghost, Task, TaskStatus};
use charon::state::{
    AppState, BuilderCategory, BuilderField, BuilderState, ConfigField, ConfigState, CurrentScreen, DashboardState, TerminalState
};

#[test]
fn test_dashboard_selection() {
    let mut state = DashboardState::default();
    state.select_next();
    assert_eq!(state.selected_ghost_index(), Some(0));

    state.ghosts.push(Ghost {
        id: "1".into(),
        hostname: "hostname".into(),
        os: "os".into(),
        last_seen: 0
    });
    state.ghosts.push(Ghost {
        id: "2".into(),
        hostname: "hostname".into(),
        os: "os".into(),
        last_seen: 0
    });

    assert_eq!(state.selected_ghost_index(), Some(0));

    state.select_next();
    assert_eq!(state.selected_ghost_index(), Some(1));

    state.select_next();
    assert_eq!(state.selected_ghost_index(), Some(0));

    state.select_prev();
    assert_eq!(state.selected_ghost_index(), Some(1));
}

#[test]
fn test_terminal_scrolling() {
    let mut state = TerminalState::default();
    state.scroll_down();
    assert_eq!(state.list_state.selected(), None);

    state.tasks.push(Task {
        id: "1".into(),
        command: "command".into(),
        args: "".into(),
        status: TaskStatus::Pending,
        result: None
    });
    state.tasks.push(Task {
        id: "2".into(),
        command: "command".into(),
        args: "".into(),
        status: TaskStatus::Pending,
        result: None
    });

    state.list_state.select(Some(0));

    state.scroll_down();
    assert_eq!(state.list_state.selected(), Some(1));

    state.scroll_down();
    assert_eq!(state.list_state.selected(), Some(1));
    
    state.scroll_up();
    assert_eq!(state.list_state.selected(), Some(0));
}

#[test]
fn test_config_field_navigation() {
    let mut state = ConfigState::default();

    assert_eq!(state.selected_field, ConfigField::Sleep);
    state.next_field();
    assert_eq!(state.selected_field, ConfigField::Jitter);
    state.next_field();
    assert_eq!(state.selected_field, ConfigField::Submit);
    state.next_field();
    assert_eq!(state.selected_field, ConfigField::Sleep);

    state.prev_field();
    assert_eq!(state.selected_field, ConfigField::Submit);
}

#[test]
fn test_app_tab_navigation() {
    let mut app = AppState::default();
    assert_eq!(app.current_screen, CurrentScreen::Dashboard);

    app.next_tab();
    assert_eq!(app.current_screen, CurrentScreen::Terminal);

    app.next_tab();
    assert_eq!(app.current_screen, CurrentScreen::Config);

    app.next_tab();
    assert_eq!(app.current_screen, CurrentScreen::Builder);

    app.next_tab();
    assert_eq!(app.current_screen, CurrentScreen::Dashboard);

    app.prev_tab();
    assert_eq!(app.current_screen, CurrentScreen::Builder);
}

#[test]
fn test_builder_category_navigation() {
    let mut state = BuilderState::default();

    state.active_category = BuilderCategory::General;
    state.selected_field = BuilderField::CategorySelect;

    state.next_field();
    assert_eq!(state.selected_field, BuilderField::Url);

    let next_cat = state.active_category.next();
    assert_eq!(next_cat, BuilderCategory::Persistence)
}

#[test]
fn test_builder_navigation_persistence_skips() {
    let mut state = BuilderState::default();
    state.active_category = BuilderCategory::Persistence;

    state.selected_field = BuilderField::CategorySelect;
    state.next_field();
    assert_eq!(state.selected_field, BuilderField::PersistToggle);
    state.next_field();
    assert_eq!(state.selected_field, BuilderField::PersistRunControl);

    state.enable_persistence = false;
    state.selected_field = BuilderField::PersistToggle;
    state.next_field();
    assert_eq!(state.selected_field, BuilderField::Submit);
}

#[test]
fn test_builder_navigation_impact_skips() {
    let mut state = BuilderState::default();
    state.active_category = BuilderCategory::Impact;

    state.selected_field = BuilderField::CategorySelect;
    state.next_field();
    assert_eq!(state.selected_field, BuilderField::ImpactToggle);
    state.next_field();
    assert_eq!(state.selected_field, BuilderField::ImpactEncrypt);

    state.enable_impact = false;
    state.selected_field = BuilderField::ImpactToggle;
    state.next_field();
    assert_eq!(state.selected_field, BuilderField::Submit);
}

#[test]
fn test_builder_navigation_exfil_skips() {
    let mut state = BuilderState::default();
    state.active_category = BuilderCategory::Exfiltration;

    state.selected_field = BuilderField::CategorySelect;
    state.next_field();
    assert_eq!(state.selected_field, BuilderField::ExfilToggle);
    state.next_field();
    assert_eq!(state.selected_field, BuilderField::ExfilHttp);

    state.enable_exfil = false;
    state.selected_field = BuilderField::ExfilToggle;
    state.next_field();
    assert_eq!(state.selected_field, BuilderField::Submit);
}

#[test]
fn test_terminal_scroll_to_bottom() {
    let mut state = TerminalState::default();

    for i in 0..5 {
        state.tasks.push(Task {
            id: i.to_string(),
            command: "".into(),
            args: "".into(),
            status: TaskStatus::Pending,
            result: None
        });
    }

    state.scroll_to_bottom();
    assert_eq!(state.list_state.selected(), Some(4));
}
