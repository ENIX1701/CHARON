use color_eyre::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState, Tabs, Wrap},
};
use serde::{Deserialize, Serialize};
use std::{io, time::Duration};

// === STRUCTS ===
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Implant {
    id: String,
    hostname: String,
    os: String,
    last_seen: i64,
}

// === GLOBAL STATE ===
struct App {
    current_tab: usize,
    should_quit: bool,
    status_message: String,
    
    ghosts: Vec<Implant>,
    ghost_table_state: TableState,
}

impl App {
    fn new() -> Self {
        Self {
            current_tab: 0,
            should_quit: false,
            status_message: "Press 'r' to refresh data from SHADOW".to_string(),    // TODO: auto refresh; has to be implemented with non-blocking interface loop
            ghosts: vec![],
            ghost_table_state: TableState::default(),
        }
    }

    // connects to SHADOW to get the list of active implants
    fn fetch_ghosts(&mut self) {
        // blocking reqwest here for simplicity in TUI loop
        // TODO: this should be done in a separate thread (tokio::spawn)
        let client = reqwest::blocking::Client::new();
        match client.get("http://127.0.0.1:9999/charon/ghosts").send() {    // TODO: parametrize URL
            Ok(resp) => {
                if let Ok(ghosts) = resp.json::<Vec<Implant>>() {
                    self.ghosts = ghosts;
                    self.status_message = format!("Updated: {} ghosts found.", self.ghosts.len());
                } else {
                    self.status_message = "ERROR parsing JSON from SHADOW".to_string();
                }
            }
            Err(_) => {
                self.status_message = "ERROR connecting to SHADOW".to_string();
            }
        }
    }

    fn next_tab(&mut self) {
        self.current_tab = (self.current_tab + 1) % 2;
    }

    fn help(&mut self) {
        // TODO
    }

    fn on_key(&mut self, key: KeyCode) {    // TODO: add help panel with some guide for new users
        match key {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Tab => self.next_tab(),
            KeyCode::Char('r') => self.fetch_ghosts(),
            KeyCode::Char('h') => self.help(),
            KeyCode::Down => {
                let i = match self.ghost_table_state.selected() {
                    Some(i) => {
                        if i >= self.ghosts.len().saturating_sub(1) { 0 } else { i + 1 }
                    }
                    None => 0,
                };
                self.ghost_table_state.select(Some(i));
            }
            KeyCode::Up => {
                let i = match self.ghost_table_state.selected() {
                    Some(i) => {
                        if i == 0 { self.ghosts.len().saturating_sub(1) } else { i - 1 }
                    }
                    None => 0,
                };
                self.ghost_table_state.select(Some(i));
            }
            _ => {}
        }
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    
    enable_raw_mode()?;
    let mut stderr = io::stderr();
    execute!(stderr, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    app.fetch_ghosts();

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    app.on_key(key.code);
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

// === UI RENDERING ===
fn ui(f: &mut Frame, app: &mut App) {
    // layout
    // header with tabs, main section and footer showing status
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // tabs
            Constraint::Min(1),    // content
            Constraint::Length(3), // status
        ])
        .split(f.area());

    // header
    let titles = vec!["GHOSTS (dashboard)", "TASKING (builder)"];
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(" CHARON C2 "))
        .select(app.current_tab)
        .style(Style::default().fg(Color::Cyan))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD).bg(Color::DarkGray));
    f.render_widget(tabs, chunks[0]);

    // content
    match app.current_tab {
        0 => render_dashboard(f, app, chunks[1]),
        1 => render_builder(f, chunks[1]),
        _ => {}
    };

    // footer
    let status_style = if app.status_message.contains("ERROR") {
        Style::default().fg(Color::Red)
    } else {
        Style::default().fg(Color::Green)
    };
    
    let footer = Paragraph::new(format!("STATUS: {} | [q] quit | [r] refresh | [tab] switch view", app.status_message))
        .style(status_style)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, chunks[2]);
}

fn render_dashboard(f: &mut Frame, app: &mut App, area: Rect) {
    let selected_style = Style::default().add_modifier(Modifier::REVERSED).fg(Color::Yellow);
    // let normal_style = Style::default().fg(Color::White);

    let header_cells = ["ID", "HOSTNAME", "OS", "LAST SEEN", "STATUS"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Cyan)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let rows = app.ghosts.iter().map(|item| {
        let now = chrono::Utc::now().timestamp();
        let diff = now - item.last_seen;
        let status = if diff < 60 { "ACTIVE" } else { "DEAD" }; // status currently based on last seen, TODO: think if this can be done better
        let status_color = if diff < 60 { Color::Green } else { Color::Red };

        let cells = vec![
            Cell::from(item.id.chars().take(8).collect::<String>() + "..."),
            Cell::from(item.hostname.clone()),
            Cell::from(item.os.clone()),
            Cell::from(diff.to_string() + "s ago"),
            Cell::from(status).style(Style::default().fg(status_color)),
        ];
        Row::new(cells).height(1)
    });

    let t = Table::new(
        rows,
        [
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(15),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(" ROAMING GHOSTs "))
    .row_highlight_style(selected_style)
    .highlight_symbol(">> ");

    f.render_stateful_widget(t, area, &mut app.ghost_table_state);
}

// TODO: placeholder for now, think through how to implement the builder after GHOST is implemented
fn render_builder(f: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" BUILDER ");
    
    let text = vec![
        Line::from("Placeholder for builder."),
        Line::from("Here you will select an implant and craft a command or payload."),
        Line::from(""),
        Line::from(Span::styled("TODO: implement", Style::default().fg(Color::Yellow))),
    ];

    let p = Paragraph::new(text)
        .block(block)
        .wrap(Wrap { trim: true });
    
    f.render_widget(p, area);
}
