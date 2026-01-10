use crate::app::{App, AppState};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table, Tabs}
};

pub fn draw(f: &mut Frame, app: &mut App) {
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

    match app.current_tab {
        0 => render_dashboard(f, app, chunks[1]),
        1 => render_terminal(f, app, chunks[1]),
        _ => {}
    }

    render_footer(f, app, chunks[2]);

    if app.state == AppState::Help {
        render_help_popup(f);
    }
}

fn render_header(f: &mut Frame, app: &mut App, area: Rect) {
    let titles = vec![" DASHBOARD ", " TERMINAL "];
    let tabs = Tabs::new(titles)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(" CHARON "))
        .select(app.current_tab)
        .style(Style::default().fg(Color::Cyan))
        .highlight_style(Style::default()
            .add_modifier(Modifier::BOLD)
            .bg(Color::DarkGray));

    f.render_widget(tabs, area);
}

fn render_dashboard(f: &mut Frame, app: &mut App, area: Rect) {
    let header_cells = ["ID", "HOSTNAME", "OS", "LAST SEEN", "STATUS"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let rows = app.ghosts.iter().map(|item| {
        let now = chrono::Utc::now().timestamp();
        let diff = now - item.last_seen;
        let (status, color) = if diff < 30 {
            ("ACTIVE", Color::Green)
        } else {
            ("DEAD", Color::Red)
        };

        let cells = vec![
            Cell::from(item.id.chars().take(8).collect::<String>() + "..."),
            Cell::from(item.hostname.clone()),
            Cell::from(item.os.clone()),
            Cell::from(format!("{}s ago", diff)),
            Cell::from(status).style(Style::default().fg(color)),
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
    .block(Block::default().borders(Borders::ALL).title(" ROAMING GHOSTs "))
    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED).fg(Color::Yellow))
    .highlight_symbol(">> ");

    f.render_stateful_widget(t, area, &mut app.ghost_table_state);
}

fn render_terminal(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(3)
        ])
        .split(area);

    
    let selected_ghost_name = if let Some(idx) = app.selected_ghost_index {
        app.ghosts.get(idx).map(|g| g.hostname.as_str()).unwrap_or("none")
    } else {
        "none"
    };
    
    let mut messages: Vec<ListItem> = Vec::new();

    if app.tasks.is_empty() {
        messages.push(ListItem::new(Line::from(vec![
            Span::raw("No history available. Type a command to start.")
        ])));
    }

    for task in &app.tasks {
        let cmd_line = Line::from(vec![
            Span::styled(format!("{}@{}> ", "ghost", selected_ghost_name), Style::default().fg(Color::Green)),
            Span::styled(format!("{} {}", task.command, task.args), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        ]);

        messages.push(ListItem::new(cmd_line));
        
        match task.status.as_str() {
            "pending" | "sent" => {
                messages.push(ListItem::new(Line::from(vec![
                    Span::styled("[PENDING...]", Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC))
                ])));
            },
            _ => {
                if let Some(output) = &task.result {
                    for line in output.lines() {
                        messages.push(ListItem::new(Line::from(vec![
                            Span::raw(line)
                        ])));
                    }
                }
            }
        }

        messages.push(ListItem::new(Line::from("")));
    }

    if app.should_scroll {
        if !messages.is_empty() {
            app.task_list_state.select(Some(messages.len() - 1));
        }
        app.should_scroll = false;
    }

    let history_block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" TERMINAL: {} ", selected_ghost_name));

    let history_list = List::new(messages)
        .block(history_block)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    f.render_stateful_widget(history_list, chunks[0], &mut app.task_list_state);

    let (border_color, title) = if app.state == AppState::Input {
        (Color::Yellow, " COMMAND INPUT (TYPING) ")
    } else {
        (Color::White, " COMMAND INPUT (Press 'i') ")
    };

    let cursor = if app.state == AppState::Input { "█" } else { "" };

    let input_text = vec![
        Line::from(vec![
            Span::styled("> ", Style::default().fg(Color::Cyan)),
            Span::styled(format!("{}{}", app.input_buffer, cursor), Style::default().fg(Color::White))
        ])
    ];

    let input_paragraph = Paragraph::new(input_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(title));
        
    f.render_widget(input_paragraph, chunks[1]);
}

fn render_footer(f: &mut Frame, app: &mut App, area: Rect) {
    let status_style = if app.status_message.contains("ERROR") {
        Style::default().fg(Color::Red)
    } else {
        Style::default().fg(Color::Green)
    };

    let footer = Paragraph::new(format!("STATUS {} | [q] quit | [tab] view", app.status_message))
        .style(status_style)
        .block(Block::default().borders(Borders::ALL));
    
    f.render_widget(footer, area);
}

fn render_help_popup(f: &mut Frame) {
    let block = Block::default().borders(Borders::ALL).title(" HELP ");
    let area = centered_rect(60, 50, f.area());
    let text = vec![
        Line::from("=== NAVIGATION ==="),
        Line::from(""),
        Line::from("tab: switch tabs (dashboard <-> builder)"),
        Line::from("up/down: select GHOST from list"),
        Line::from("r: force refresh"),
        Line::from("h: toggle this window"),
        Line::from("q: quit"),
        Line::from(""),
        Line::from("=== TERMINAL MODE ==="),
        Line::from("i: enter input mode"),
        Line::from("enter: send command"),
        Line::from("esc: exit input mode"),
    ];
    let p = Paragraph::new(text).block(block).style(Style::default().bg(Color::DarkGray));

    f.render_widget(Clear, area);
    f.render_widget(p, area);
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