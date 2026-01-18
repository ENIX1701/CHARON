use crate::models::{Ghost, Task};
use ratatui::widgets::{ListState, TableState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurrentScreen {
    Dashboard,
    Terminal,
    Config,
    Builder
}

#[derive(Debug, Clone)]
pub struct DashboardState {
    pub ghosts: Vec<Ghost>,
    pub table_state: TableState
}

impl Default for DashboardState {
    fn default() -> Self {
        let mut state = TableState::default();
        state.select(Some(0));

        Self {
            ghosts: Vec::new(),
            table_state: state
        }
    }
}

impl DashboardState {
    pub fn select_next(&mut self) {
        if self.ghosts.is_empty() {
            return;
        }

        let i = match self.table_state.selected() {
            Some(i) => {
                if i >= self.ghosts.len() - 1 { 0 }
                else { i + 1 }
            },
            None => 0
        };

        self.table_state.select(Some(i));
    }

    pub fn select_prev(&mut self) {
        if self.ghosts.is_empty() {
            return;
        }

        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 { self.ghosts.len() - 1 }
                else { i - 1 }
            },
            None => 0
        };

        self.table_state.select(Some(i));
    }

    pub fn selected_ghost_id(&self) -> Option<String> {
        self.table_state
            .selected()
            .and_then(|i| self.ghosts.get(i))
            .map(|g| g.id.clone())
    }

    pub fn selected_ghost_index(&self) -> Option<usize> {
        self.table_state.selected()
    }
}

#[derive(Debug, Clone)]
pub struct TerminalState {
    pub tasks: Vec<Task>,
    pub list_state: ListState,
    pub input_buffer: String,
    pub active_ghost_id: Option<String>,
    pub input_mode: bool
}

impl Default for TerminalState {
    fn default() -> Self {
        Self {
            tasks: Vec::new(),
            list_state: ListState::default(),
            input_buffer: String::new(),
            active_ghost_id: None,
            input_mode: false
        }
    }
}

impl TerminalState {
    pub fn scroll_down(&mut self) {
        if self.tasks.is_empty() {
            return;
        }

        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.tasks.len() - 1 { i }
                else { i + 1 }
            },
            None => 0
        };

        self.list_state.select(Some(i));
    }

    pub fn scroll_up(&mut self) {
        if self.tasks.is_empty() {
            return;
        }

        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 { 0 }
                else { i - 1 }
            },
            None => 0
        };

        self.list_state.select(Some(i));
    }

    pub fn scroll_to_bottom(&mut self) {
        if !self.tasks.is_empty() {
            self.list_state.select(Some(self.tasks.len() - 1));
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigField {
    Sleep,
    Jitter,
    Submit
}

#[derive(Debug, Clone)]
pub struct ConfigState {
    pub selected_field: ConfigField,
    pub sleep_input: String,
    pub jitter_input: String
}

impl Default for ConfigState {
    fn default() -> Self {
        Self {
            selected_field: ConfigField::Sleep,
            sleep_input: "10".to_string(),
            jitter_input: "5".to_string()
        }
    }
}

impl ConfigState {
    pub fn next_field(&mut self) {
        self.selected_field = match self.selected_field {
            ConfigField::Sleep => ConfigField::Jitter,
            ConfigField::Jitter => ConfigField::Submit,
            ConfigField::Submit => ConfigField::Sleep
        };
    }

    pub fn prev_field(&mut self) {
        self.selected_field = match self.selected_field {
            ConfigField::Sleep => ConfigField::Submit,
            ConfigField::Jitter => ConfigField::Sleep,
            ConfigField::Submit => ConfigField::Jitter
        };
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuilderField {
    Url,
    Port,
    EnableDebug,
    EnablePersistence,
    EnableImpact,
    EnableExfil,
    Submit
}

#[derive(Debug, Clone)]
pub struct BuilderState {
    pub selected_field: BuilderField,
    pub target_url: String,
    pub target_port: String,
    pub enable_debug: bool,
    pub enable_persistence: bool,
    pub enable_impact: bool,
    pub enable_exfil: bool,
    pub build_status_msg: String
}

impl Default for BuilderState {
    fn default() -> Self {
        Self {
            selected_field: BuilderField::Url,
            target_url: "127.0.0.1".to_string(),
            target_port: "9999".to_string(),
            enable_debug: true,
            enable_persistence: true,
            enable_impact: true,
            enable_exfil: true,
            build_status_msg: "IDLE".to_string()
        }
    }
}

impl BuilderState {
    pub fn next_field(&mut self) {
        use BuilderField::*;
        self.selected_field = match self.selected_field {
            Url => Port,
            Port => EnableDebug,
            EnableDebug => EnablePersistence,
            EnablePersistence => EnableImpact,
            EnableImpact => EnableExfil,
            EnableExfil => Submit,
            Submit => Url
        };
    }

    pub fn prev_field(&mut self) {
        use BuilderField::*;
        self.selected_field = match self.selected_field {
            Url => Submit,
            Port => Url,
            EnableDebug => Port,
            EnablePersistence => EnableDebug,
            EnableImpact => EnablePersistence,
            EnableExfil => EnableImpact,
            Submit => EnableExfil
        };
    }
}

#[derive(Clone, Debug)]
pub struct AppState {
    // main state
    pub current_screen: CurrentScreen,

    // substates
    pub dashboard: DashboardState,
    pub terminal: TerminalState,
    pub config: ConfigState,
    pub builder: BuilderState,

    // global flags
    pub show_help: bool,
    pub show_action_menu: bool,
    pub status_message: String
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            current_screen: CurrentScreen::Dashboard,
            dashboard: DashboardState::default(),
            terminal: TerminalState::default(),
            config: ConfigState::default(),
            builder: BuilderState::default(),
            show_help: false,
            show_action_menu: false,
            status_message: "READY - press 'h' for help".to_string()
        }
    }
}

impl AppState {
    pub fn next_tab(&mut self) {
        self.current_screen = match self.current_screen {
            CurrentScreen::Dashboard => CurrentScreen::Terminal,
            CurrentScreen::Terminal => CurrentScreen::Config,
            CurrentScreen::Config => CurrentScreen::Builder,
            CurrentScreen::Builder => CurrentScreen::Dashboard
        };
    }

    pub fn prev_tab(&mut self) {
        self.current_screen = match self.current_screen {
            CurrentScreen::Dashboard => CurrentScreen::Builder,
            CurrentScreen::Terminal => CurrentScreen::Dashboard,
            CurrentScreen::Config => CurrentScreen::Terminal,
            CurrentScreen::Builder => CurrentScreen::Config
        };
    }
}
