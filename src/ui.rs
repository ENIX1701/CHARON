use crate::app::{App, AppState, ConfigField};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Clear, List, ListItem, ListState, Paragraph, Row, Table, Tabs}
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
        2 => render_config(f, app, chunks[1]),
        // 3 => render_builder(f, app, chunks[1]),
        _ => {}
    }

    render_footer(f, app, chunks[2]);

    match app.state {
        AppState::Help => render_help_popup(f),
        AppState::ActionMenu => render_action_menu(f, app),
        AppState::ConfirmModal => render_confirm_modal(f, app),
        _ => {}
    }
}

fn render_action_menu(f: &mut Frame, app: &mut App) {
    let area = centered_rect(40, 30, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" GHOST ACTION ")
        .style(Style::default().bg(Color::DarkGray));

    let items = vec![
        ListItem::new(" [!] KILL SWITCH ").style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
    ];

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(Color::Red).fg(Color::White));

    let mut state = ListState::default();
    state.select(Some(app.action_menu_index));

    f.render_stateful_widget(list, area, &mut state);
}

fn render_confirm_modal(f: &mut Frame, app: &mut App) {
    let area = centered_rect(50, 20, f.area());
    f.render_widget(Clear, area);

    let ghost_name = if let Some(idx) = app.selected_ghost_index {
        app.ghosts.get(idx).map(|g| g.hostname.as_str()).unwrap_or("unknown")
    } else {
        "unknown"
    };

    let (title, color, prompt_text) = match app.action_menu_index {
        0 => (
            " CONFIRM KILL ",
            Color::Red,
            vec![
                Span::raw("Are you sure you want to "),
                Span::styled("KILL", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!(" ghost '{}'?", ghost_name))
            ]
        ),
        _ => (" CONFIRM ", Color::Blue, vec![Span::raw("are you sure?")])
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Style::default().bg(color).fg(Color::White));
    
    let text = vec![
        Line::from(""),
        Line::from(prompt_text),
        Line::from(""),
        Line::from("press [ENTER] to confirm or [ESC] to cancel")
    ];

    let p = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center);

    f.render_widget(p, area);
}

fn render_header(f: &mut Frame, app: &mut App, area: Rect) {
    let titles = vec![" DASHBOARD ", " TERMINAL ", " CONFIG ", " BUILDER "];
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
        let (status, color) = if diff < 30 {    // make this configurable with GHOST beaconing rate * 3
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

fn render_config(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // info
            Constraint::Length(3),  // sleep
            Constraint::Length(3),  // jitter
            Constraint::Length(1),  // submit
            Constraint::Min(1)
        ])
        .margin(1)
        .split(area);

    let block = Block::default().borders(Borders::ALL).title(" GHOST CONFIGURATION ");
    f.render_widget(block, area);

    // ghost info

    // sleep panel
    let sleep_active = app.config.selection == ConfigField::Sleep;
    let sleep_style = if sleep_active { Style::default().fg(Color::Yellow) } else { Style::default() };

    let sleep_text = format!("Sleep interval (sec): {}", app.config.sleep_input);
    let sleep_p = Paragraph::new(sleep_text)
        .block(Block::default().borders(Borders::ALL).border_style(sleep_style))
        .style(if sleep_active { Style::default().add_modifier(Modifier::BOLD) } else { Style::default() });
    f.render_widget(sleep_p, chunks[1]);

    // jitter panel
    let jitter_active = app.config.selection == ConfigField::Jitter;
    let jitter_style = if jitter_active { Style::default().fg(Color::Yellow) } else { Style::default() };

    let jitter_text = format!("Jitter (%): {}", app.config.jitter_input);
    let jitter_p = Paragraph::new(jitter_text)
        .block(Block::default().borders(Borders::ALL).border_style(jitter_style))
        .style(if jitter_active { Style::default().add_modifier(Modifier::BOLD) } else { Style::default() });
    f.render_widget(jitter_p, chunks[2]);

    let btn_active = app.config.selection == ConfigField::Submit;
    let btn_style = if btn_active {
        Style::default().bg(Color::Blue).fg(Color::White).add_modifier(Modifier::BOLD)
    } else {
        Style::default().bg(Color::DarkGray).fg(Color::Gray)
    };

    let btn_p = Paragraph::new("[ UPDATE CONFIGURATION ]")
        .alignment(Alignment::Center)
        .style(btn_style)
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(btn_p, chunks[3]);
}

fn render_footer(f: &mut Frame, app: &mut App, area: Rect) {
    let status_style = if app.status_message.contains("ERROR") {
        Style::default().fg(Color::Red)
    } else {
        Style::default().fg(Color::Green)
    };

    let footer = Paragraph::new(format!("STATUS {} | [q] quit | [x] actions | [←/→] change tabs", app.status_message))
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
        Line::from("left/right: switch tabs"),
        Line::from("up/down: select item or field"),
        Line::from("x: open action menu"),
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