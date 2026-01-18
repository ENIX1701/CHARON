use crate::models::TaskStatus;
use crate::state::{AppState, BuilderField, ConfigField, CurrentScreen};
use ratatui::{
    prelude::*,
    widgets::{
        Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table, Tabs
    }
};

pub fn draw(f: &mut Frame, app: &AppState) {
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

    render_header(f, app, chunks[0]);

    match app.current_screen {
        CurrentScreen::Dashboard => render_dashboard(f, app, chunks[1]),
        CurrentScreen::Terminal => render_terminal(f, app, chunks[1]),
        CurrentScreen::Config => render_config(f, app, chunks[1]),
        CurrentScreen::Builder => render_builder(f, app, chunks[1])
    }

    render_footer(f, app, chunks[2]);

    if app.show_help {
        render_help_popup(f);
    }

    if app.show_action_menu {
        render_action_menu(f, app);
    }
}

fn render_header(f: &mut Frame, app: &AppState, area: Rect) {
    let titles = vec![" DASHBOARD ", " TERMINAL ", " CONFIG ", " BUILDER "];
    let current_index = match app.current_screen {
        CurrentScreen::Dashboard => 0,
        CurrentScreen::Terminal => 1,
        CurrentScreen::Config => 2,
        CurrentScreen::Builder => 3
    };

    let tabs = Tabs::new(titles)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(" CHARON "))
        .select(current_index)
        .style(Style::default().fg(Color::Cyan))
        .highlight_style(Style::default()
            .add_modifier(Modifier::BOLD)
            .bg(Color::DarkGray));

    f.render_widget(tabs, area);
}

fn render_dashboard(f: &mut Frame, app: &AppState, area: Rect) {
    let header_cells = ["ID", "HOSTNAME", "OS", "LAST SEEN", "STATUS"]
        .iter()
        .map(|h| {
            Cell::from(*h).style(
                Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
            )
        });
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let rows = app.dashboard.ghosts.iter().map(|ghost| {
        let now = chrono::Utc::now().timestamp();
        let diff = now - ghost.last_seen;
        let (status_str, color) = if diff < 60 {    // make this configurable with GHOST beaconing rate * 3
            ("ACTIVE", Color::Green)
        } else {
            ("SILENT", Color::Red)
        };

        let cells = vec![
            Cell::from(ghost.id.chars().take(8).collect::<String>() + "..."),
            Cell::from(ghost.hostname.clone()),
            Cell::from(ghost.os.clone()),
            Cell::from(format!("{}s ago", diff)),
            Cell::from(status_str).style(Style::default().fg(color)),
        ];
        Row::new(cells).height(1)
    });

    let t = Table::new(
        rows,
        [
            Constraint::Percentage(15),
            Constraint::Percentage(25),
            Constraint::Percentage(15),
            Constraint::Percentage(20),
            Constraint::Percentage(25),
        ],
    )
    .header(header)
    .block(
        Block::default()
        .borders(Borders::ALL)
        .title(" ROAMING GHOSTs ")
    )
    .row_highlight_style(
        Style::default()
        .add_modifier(Modifier::REVERSED)
        .fg(Color::Yellow)
    )
    .highlight_symbol(">> ");

    let mut state = app.dashboard.table_state.clone();
    f.render_stateful_widget(t, area, &mut state);
}

fn render_terminal(f: &mut Frame, app: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(3)
        ])
        .split(area);
    
    let mut messages: Vec<ListItem> = Vec::new();
    let ghost_name = app.terminal.active_ghost_id.as_deref().unwrap_or("none");

    if app.terminal.tasks.is_empty() {
        messages.push(ListItem::new(Line::from(vec![
            Span::raw("No history available. Select a GHOST in the dashboard.")
        ])));
    }

    for task in &app.terminal.tasks {
        let header = Line::from(vec![
            Span::styled(format!("ghost@{}> ", ghost_name), Style::default().fg(Color::Green)),
            Span::styled(format!("{} {}", task.command, task.args), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        ]);
        messages.push(ListItem::new(header));
        
        match task.status {
            TaskStatus::Pending | TaskStatus::Sent => {
                messages.push(ListItem::new(Line::from(vec![
                    Span::styled("[PENDING...]", Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC))
                ])));
            },
            _ => {
                if let Some(result) = &task.result {
                    for line in result.lines() {
                        messages.push(ListItem::new(Line::from(vec![
                            Span::raw(line)
                        ])));
                    }
                }
            }
        }

        messages.push(ListItem::new(Line::from("")));
    }

    let history_block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" TERMINAL: {} ", ghost_name));

    let history_list = List::new(messages)
        .block(history_block)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    let mut list_state = app.terminal.list_state.clone();
    f.render_stateful_widget(history_list, chunks[0], &mut list_state);

    let (border_color, title) = (Color::Yellow, " COMMAND INPUT (TYPING) ");

    let cursor = "█";

    let input_text = vec![
        Line::from(vec![
            Span::styled("> ", Style::default().fg(Color::Cyan)),
            Span::styled(format!("{}{}", app.terminal.input_buffer, cursor), Style::default().fg(Color::White))
        ])
    ];

    let input_paragraph = Paragraph::new(input_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(title));
        
    f.render_widget(input_paragraph, chunks[1]);
}

fn render_config(f: &mut Frame, app: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // info
            Constraint::Length(3),  // sleep
            Constraint::Length(3),  // jitter
            Constraint::Length(3),  // submit
            Constraint::Min(1)
        ])
        .margin(1)
        .split(area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" GHOST CONFIGURATION ");
    f.render_widget(block, area);

    let get_style = |field: ConfigField| {
        if app.config.selected_field == field {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        }
    };

    let sleep_p = Paragraph::new(format!("Sleep interval (s): {}", app.config.sleep_input))
        .block(Block::default().borders(Borders::ALL).border_style(get_style(ConfigField::Sleep)));
    f.render_widget(sleep_p, chunks[1]);

    let jitter_p = Paragraph::new(format!("Jitter (%): {}", app.config.jitter_input))
        .block(Block::default().borders(Borders::ALL).border_style(get_style(ConfigField::Jitter)));
    f.render_widget(jitter_p, chunks[2]);

    let submit_style = if app.config.selected_field == ConfigField::Submit {
        Style::default().bg(Color::Blue).fg(Color::White)
    } else {
        Style::default().bg(Color::DarkGray)
    };
    let submit_p = Paragraph::new("[ UPDATE CONFIGURATION ]")
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).style(submit_style));
    f.render_widget(submit_p, chunks[3]);
}

fn render_builder(f: &mut Frame, app: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // url/port
            Constraint::Length(8),  // toggles
            Constraint::Length(3),  // submit
            Constraint::Min(1)      // status
        ])
        .margin(1)
        .split(area);

    let main_block = Block::default().borders(Borders::ALL).title(" GHOST PAYLOAD BUILDER ");
    f.render_widget(main_block, area);

    let net_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(chunks[0]);
    
    let get_style = |field: BuilderField| {
        if app.builder.selected_field == field {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        }
    };

    f.render_widget(
        Paragraph::new(format!("SHADOW URL: {}", app.builder.target_url))
        .block(Block::default().borders(Borders::ALL).border_style(get_style(BuilderField::Url))),
        net_chunks[0]
    );

    f.render_widget(
        Paragraph::new(format!("PORT: {}", app.builder.target_port))
        .block(Block::default().borders(Borders::ALL).border_style(get_style(BuilderField::Port))),
        net_chunks[1]
    );

    let toggles = vec![
        ("Debug mode", app.builder.enable_debug, BuilderField::EnableDebug),
        ("Persistence", app.builder.enable_persistence, BuilderField::EnablePersistence),
        ("Impact", app.builder.enable_impact, BuilderField::EnableImpact),
        ("Exfiltration", app.builder.enable_exfil, BuilderField::EnableExfil)
    ];
    
    let items: Vec<ListItem> = toggles
        .iter()
        .map(|(label, active, field)| {
            let check = if *active { "[x]" } else { "[ ]" };
            let content = format!("{} {}", check, label);
            let style = if app.builder.selected_field == *field {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(content).style(style)
        })
        .collect();

    f.render_widget(
        List::new(items).block(Block::default().borders(Borders::ALL).title(" Modules ")),
        chunks[1]
    );

    let btn_style = if app.builder.selected_field == BuilderField::Submit {
        Style::default().bg(Color::Red).fg(Color::White).add_modifier(Modifier::BOLD)
    } else {
        Style::default().bg(Color::DarkGray).fg(Color::Gray)
    };

    f.render_widget(
        Paragraph::new("[ COMPILE PAYLOAD ]")
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).style(btn_style)),
        chunks[2]
    );

    let status_color = if app.builder.build_status_msg.contains("ERROR") { Color::Red }
                        else if app.builder.build_status_msg.contains("SUCCESS") { Color::Green }
                        else { Color::Cyan };

    f.render_widget(Paragraph::new(app.builder.build_status_msg.clone())
        .block(
            Block::default()
                .borders(Borders::TOP)
                .title(" BUILD OUTPUT ")
        )
        .style(Style::default().fg(status_color)),
        chunks[3]
    );
}

fn render_footer(f: &mut Frame, app: &AppState, area: Rect) {
    let status_style = if app.status_message.to_lowercase().contains("error") {
        Style::default().fg(Color::Red)
    } else {
        Style::default().fg(Color::Green)
    };

    f.render_widget(
        Paragraph::new(format!(
            "STATUS {} | [q] quit | [x] actions | [←/→] change tabs", app.status_message
        ))
        .style(status_style)
        .block(Block::default().borders(Borders::ALL)),
        area
    );
}

fn render_help_popup(f: &mut Frame) {
    let area = centered_rect(60, 50, f.area());
    f.render_widget(Clear, area);

    let text = vec![
        Line::from("=== NAVIGATION ==="),
        Line::from(""),
        Line::from("left/right: switch tabs"),
        Line::from("up/down: select item or field"),
        Line::from("x: open action menu"),
        Line::from("r: force refresh"),
        Line::from("h: toggle this window"),
        Line::from("q: quit"),
        Line::from(""),
        Line::from("=== TERMINAL MODE ==="),
        Line::from(""),
        Line::from("i: enter input mode"),
        Line::from("enter: send command"),
        Line::from("esc: exit input mode"),
        Line::from(""),
    ];

    f.render_widget(
        Paragraph::new(text).block(Block::default().borders(Borders::ALL).title(" HELP ")).style(Style::default().bg(Color::DarkGray)),
    area);
}

fn render_action_menu(f: &mut Frame, _app: &AppState) {
    let area = centered_rect(40, 20, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" GHOST ACTION ")
        .style(Style::default().bg(Color::DarkGray));

    let text = vec![
        Line::from(""),
        Line::from(Span::styled(" [!] KILL SWITCH ", Style::default().add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from("Press [ENTER] to confirm kill"),
        Line::from("Press [ESC] to cancer")
    ];

    f.render_widget(
        Paragraph::new(text).block(block).alignment(Alignment::Center),
        area
    );
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2)
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2)
        ])
        .split(popup_layout[1])[1]
}
