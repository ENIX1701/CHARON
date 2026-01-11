mod app;
mod client;
mod ui;

use app::{App, AppState};
use color_eyre::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, time::Duration};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    
    enable_raw_mode()?;
    let mut stderr = io::stderr();
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        if let Ok(message) = app.network_rx.try_recv() {
            app.handle_network_message(message);
        }

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if app.state == AppState::Input {
                    app.handle_input_mode(key);
                } else {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('h') => app.toggle_help(),
                        KeyCode::Char('r') => app.refresh_ghosts(),
                        _ => app.handle_key(key.code)
                    }
                }
            }
        } else {
            app.on_tick()
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    Ok(())
}
