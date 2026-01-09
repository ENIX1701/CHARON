use crate::client::{self, Ghost};
use crossterm::event::KeyCode;
use ratatui::widgets::TableState;
use tokio::sync::mpsc;

#[derive(PartialEq)]
pub enum AppState {
    Normal,
    Input,
    Help
}

pub enum NetworkEvent {
    GhostsFetched(Result<Vec<Ghost>, String>),
    TaskSent(Result<String, String>)
}

pub struct App {
    pub current_tab: usize,
    pub state: AppState,
    pub ghosts: Vec<Ghost>,
    pub ghost_table_state: TableState,

    pub input_buffer: String,
    pub selected_ghost_index: Option<usize>,
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
            input_buffer: String::new(),
            selected_ghost_index: None,
            status_message: "READY press \'h\' for help".to_string(),
            network_tx: tx,
            network_rx: rx,
            tick_count: 0
        };
        
        app.refresh_ghosts();
        app
    }

    pub fn next_tab(&mut self) {
        self.current_tab = (self.current_tab + 1) % 2;
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
    }
    
    pub fn refresh_ghosts(&mut self) {
        let tx = self.network_tx.clone();
        tokio::spawn(async move {
            let res = client::fetch_ghosts().await;
            let _ = tx.send(NetworkEvent::GhostsFetched(res)).await;
        });
    }

    pub fn send_current_command(&mut self) {
        if let Some(idx) = self.selected_ghost_index {
            if let Some(ghost) = self.ghosts.get(idx) {
                let id = ghost.id.clone();
                let full_command = self.input_buffer.clone();
                let tx = self.network_tx.clone();

                let parts: Vec<&str> = full_command.splitn(2, ' ').collect();
                let command = parts[0].to_string();
                let args = if parts.len() > 1 { parts[1].to_string() } else { "".to_string() };

                tokio::spawn(async move {
                    let res = client::send_task(id, command, args).await;
                    let _ = tx.send(NetworkEvent::TaskSent(res)).await;
                });

                self.status_message = "sending task".to_string();
                self.input_buffer.clear();
            }
        }
    }

    pub fn handle_network_message(&mut self, message: NetworkEvent) {
        match message {
            NetworkEvent::GhostsFetched(result) => match result {
                Ok(data) => {
                    self.ghosts = data;

                    if self.ghosts.len() > 0 && self.ghost_table_state.selected().is_none() {
                        self.ghost_table_state.select(Some(0));
                    }

                    self.status_message = format!("UPDATED {} ghosts online", self.ghosts.len());
                },
                Err(e) => self.status_message = format!("ERROR {}", e)
            },
            NetworkEvent::TaskSent(result) => match result {
                Ok(message) => self.status_message = format!("SUCCESS {}", message),
                Err(e) => self.status_message = format!("ERROR {}", e)
            }
        }
    }

    pub fn handle_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Down => self.next_ghost(),
            KeyCode::Up => self.prev_ghost(),
            KeyCode::Char('i') if self.current_tab == 1 => {
                self.state = AppState::Input;
            },
            _ => {}
        }
    }

    pub fn handle_input_mode(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Esc => self.state = AppState::Normal,
            KeyCode::Enter => {
                self.send_current_command();
                self.state = AppState::Normal;
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
    }

    pub fn prev_ghost(&mut self) {
        let i = match self.ghost_table_state.selected() {
            Some(i) => if i == 0 { self.ghosts.len().saturating_sub(1) } else { i - 1 },
            None => 0
        };

        self.ghost_table_state.select(Some(i));
        self.selected_ghost_index = Some(i);
    }
}
