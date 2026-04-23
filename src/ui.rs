use crate::models::TaskStatus;
use crate::state::{AppState, BuilderCategory, BuilderField, ConfigField, CurrentScreen};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table, Tabs},
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
        CurrentScreen::Builder => render_builder(f, app, chunks[1]),
        CurrentScreen::Loot => render_loot(f, app, chunks[1]),
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
    let titles = vec![
        " DASHBOARD ",
        " TERMINAL ",
        " CONFIG ",
        " BUILDER ",
        " LOOT ",
    ];
    let current_index = match app.current_screen {
        CurrentScreen::Dashboard => 0,
        CurrentScreen::Terminal => 1,
        CurrentScreen::Config => 2,
        CurrentScreen::Builder => 3,
        CurrentScreen::Loot => 4,
    };

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(" CHARON "))
        .select(current_index)
        .style(Style::default().fg(Color::Cyan))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::DarkGray),
        );

    f.render_widget(tabs, area);
}

fn render_dashboard(f: &mut Frame, app: &AppState, area: Rect) {
    let header_cells = ["ID", "HOSTNAME", "OS", "LAST SEEN", "STATUS"]
        .iter()
        .map(|h| {
            Cell::from(*h).style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
        });
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let rows = app.dashboard.ghosts.iter().map(|ghost| {
        let now = chrono::Utc::now().timestamp();
        let diff = now - ghost.last_seen;
        let (status_str, color) = if diff < 60 {
            // make this configurable with GHOST beaconing rate * 3
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
            .title(" ROAMING GHOSTs "),
    )
    .row_highlight_style(
        Style::default()
            .add_modifier(Modifier::REVERSED)
            .fg(Color::Yellow),
    )
    .highlight_symbol(">> ");

    let mut state = app.dashboard.table_state.clone();
    f.render_stateful_widget(t, area, &mut state);
}

fn render_terminal(f: &mut Frame, app: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(area);

    let mut messages: Vec<ListItem> = Vec::new();
    let ghost_name = app.terminal.active_ghost_id.as_deref().unwrap_or("none");

    if app.terminal.tasks.is_empty() {
        messages.push(ListItem::new(Line::from(vec![Span::raw(
            "No history available. Select a GHOST in the dashboard.",
        )])));
    }

    for task in &app.terminal.tasks {
        let mut lines = Vec::new();

        lines.push(Line::from(vec![
            Span::styled(
                format!("ghost@{}> ", ghost_name),
                Style::default().fg(Color::Green),
            ),
            Span::styled(
                format!("{} {}", task.command, task.args),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));

        match task.status {
            TaskStatus::Pending | TaskStatus::Sent => {
                lines.push(Line::from(vec![Span::styled(
                    "[PENDING...]",
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::ITALIC),
                )]));
            }
            _ => {
                if let Some(result) = &task.result {
                    for line in result.lines() {
                        lines.push(Line::from(Span::raw(line)));
                    }
                }
            }
        }
        lines.push(Line::from(""));

        messages.push(ListItem::new(lines));
    }

    let history_block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" TERMINAL: {} ", ghost_name));

    let history_list = List::new(messages)
        .block(history_block)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    let mut list_state = app.terminal.list_state.clone();
    f.render_stateful_widget(history_list, chunks[0], &mut list_state);

    let (border_color, title) = if app.terminal.input_mode {
        (Color::Yellow, " COMMAND INPUT (TYPING) ")
    } else {
        (Color::DarkGray, " COMMAND INPUT (Press 'i' to type) ")
    };

    let cursor = if app.terminal.input_mode { "█" } else { "" };

    let input_text = vec![Line::from(vec![
        Span::styled("> ", Style::default().fg(Color::Cyan)),
        Span::styled(
            format!("{}{}", app.terminal.input_buffer, cursor),
            Style::default().fg(Color::White),
        ),
    ])];

    let input_paragraph = Paragraph::new(input_text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(title),
    );

    f.render_widget(input_paragraph, chunks[1]);
}

fn render_config(f: &mut Frame, app: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // info
            Constraint::Length(3), // sleep
            Constraint::Length(3), // jitter
            Constraint::Length(3), // submit
            Constraint::Min(1),
        ])
        .margin(1)
        .split(area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" GHOST CONFIGURATION ");
    f.render_widget(block, area);

    let get_style = |field: ConfigField| {
        if app.config.selected_field == field {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        }
    };

    let sleep_p = Paragraph::new(format!("Sleep interval (s): {}", app.config.sleep_input)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(get_style(ConfigField::Sleep)),
    );
    f.render_widget(sleep_p, chunks[1]);

    let jitter_p = Paragraph::new(format!("Jitter (%): {}", app.config.jitter_input)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(get_style(ConfigField::Jitter)),
    );
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
            Constraint::Length(3), // category tabs
            Constraint::Min(1),    // content
            Constraint::Length(3), // submit
            Constraint::Length(3), // status
        ])
        .margin(1)
        .split(area);

    let main_block = Block::default()
        .borders(Borders::ALL)
        .title(" GHOST PAYLOAD BUILDER ");
    f.render_widget(main_block, area);

    let categories = vec![" GENERAL ", " PERSISTENCE ", " IMPACT ", " EXFILTRATION "];
    let cat_index = match app.builder.active_category {
        BuilderCategory::General => 0,
        BuilderCategory::Persistence => 1,
        BuilderCategory::Impact => 2,
        BuilderCategory::Exfiltration => 3,
    };

    let cat_style = if app.builder.selected_field == BuilderField::CategorySelect {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Cyan)
    };

    let cat_tabs = Tabs::new(categories)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(cat_style),
        )
        .select(cat_index)
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
    f.render_widget(cat_tabs, chunks[0]);

    match app.builder.active_category {
        BuilderCategory::General => {
            let gen_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Min(6),
                    Constraint::Length(3),
                ])
                .split(chunks[1]);

            let get_style = |field: BuilderField| {
                if app.builder.selected_field == field {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                }
            };

            f.render_widget(
                Paragraph::new(format!("SHADOW URL: {}", app.builder.target_url)).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(get_style(BuilderField::Url)),
                ),
                gen_chunks[0],
            );

            f.render_widget(
                Paragraph::new(format!("PORT: {}", app.builder.target_port)).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(get_style(BuilderField::Port)),
                ),
                gen_chunks[1],
            );

            let dbg_check = if app.builder.enable_debug {
                "[x]"
            } else {
                "[ ]"
            };
            f.render_widget(
                Paragraph::new(format!("{} debug mode", dbg_check)).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(get_style(BuilderField::EnableDebug)),
                ),
                gen_chunks[2],
            );

            let scenario_desc = match app.builder.scenario_mode.as_str() {
                "NONE" => "No preset overrides. Configure all modules manually.",
                "RANSOMWARE" => "Modules: Persistence, Impact (Encrypt), Exfil",
                "ESPIONAGE" => "Modules: Persistence, Gather, Exfil",
                "WIPER" => "Modules: Impact (Wipe)",
                "INFOSTEALER" => "Modules: Gather (SSH, passwd/shadow), Exfil",
                "APT" => "Modules: Persistence | Sleep 60s, high jitter",
                "APT29" => "Modules: RunControl Persistence, Gather, Exfil | Sleep 4h",
                "APT44" => "Modules: Cron Persistence, Impact (Wipe)",
                "APT38" => "Modules: Gather (SysInfo, SSH), Exfil, Impact (Encrypt)",
                _ => "Unknown configuration",
            };

            let modes = [
                "NONE",
                "RANSOMWARE",
                "ESPIONAGE",
                "WIPER",
                "INFOSTEALER",
                "APT",
                "APT29",
                "APT44",
                "APT38",
            ];
            let mut mode_spans = vec![Span::raw("Available: ")];

            for (i, mode) in modes.iter().enumerate() {
                if *mode == app.builder.scenario_mode.as_str() {
                    let color = match *mode {
                        "NONE" => Color::DarkGray,
                        "RANSOMWARE" | "WIPER" => Color::Red,
                        "ESPIONAGE" | "INFOSTEALER" => Color::Yellow,
                        "APT" | "APT29" | "APT44" | "APT38" => Color::Magenta,
                        _ => Color::White,
                    };
                    mode_spans.push(Span::styled(
                        format!("[{}]", mode),
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    ));
                } else {
                    mode_spans.push(Span::styled(*mode, Style::default().fg(Color::DarkGray)));
                }

                if i < modes.len() - 1 {
                    mode_spans.push(Span::raw(" "));
                }
            }

            let scenario_text = vec![
                Line::from(mode_spans),
                Line::from(""),
                Line::from(Span::styled(
                    format!("-> {}", scenario_desc),
                    Style::default()
                        .fg(Color::Gray)
                        .add_modifier(Modifier::ITALIC),
                )),
            ];

            f.render_widget(
                Paragraph::new(scenario_text)
                    .wrap(ratatui::widgets::Wrap { trim: true })
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(" SCENARIO MODE ")
                            .border_style(get_style(BuilderField::Scenario)),
                    ),
                gen_chunks[3],
            );

            let sev_color = match app.builder.impact_level.as_str() {
                "TEST" => Color::Green,
                "USER" => Color::Yellow,
                "SYSTEM" => Color::Red,
                _ => Color::White,
            };

            f.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::raw("severity: [ "),
                    Span::styled(
                        &app.builder.impact_level,
                        Style::default().fg(sev_color).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" ]"),
                ]))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(get_style(BuilderField::ImpactLevel)),
                ),
                gen_chunks[4],
            );
        }
        BuilderCategory::Persistence => {
            let mut items = vec![(
                "enable persistence",
                app.builder.enable_persistence,
                BuilderField::PersistToggle,
            )];

            if app.builder.enable_persistence {
                items.extend_from_slice(&[
                    (
                        "method: runcontrol",
                        app.builder.persist_runcontrol,
                        BuilderField::PersistRunControl,
                    ),
                    (
                        "method: service",
                        app.builder.persist_service,
                        BuilderField::PersistService,
                    ),
                    (
                        "method: cron",
                        app.builder.persist_cron,
                        BuilderField::PersistCron,
                    ),
                ]);
            }

            render_checkbox_list(f, &app.builder.selected_field, items, chunks[1]);
        }
        BuilderCategory::Impact => {
            let mut list_items = Vec::new();

            let create_item = |label: &str, is_selected: bool, is_active: bool, is_radio: bool| {
                let check = if is_radio {
                    if is_active { "(o)" } else { "( )" }
                } else {
                    if is_active { "[x]" } else { "[ ]" }
                };

                let content = format!("{} {}", check, label);
                let style = if is_selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                ListItem::new(content).style(style)
            };

            list_items.push(create_item(
                "enable impact",
                app.builder.selected_field == BuilderField::ImpactToggle,
                app.builder.enable_impact,
                false,
            ));

            if app.builder.enable_impact {
                list_items.push(create_item(
                    "encryption",
                    app.builder.selected_field == BuilderField::ImpactEncrypt,
                    app.builder.impact_encrypt,
                    false,
                ));

                if app.builder.impact_encrypt {
                    list_items.push(create_item(
                        "   XOR",
                        app.builder.selected_field == BuilderField::ImpactEncryptAlgoXor,
                        app.builder.encryption_algo == "XOR",
                        true,
                    ));
                    list_items.push(create_item(
                        "   AES",
                        app.builder.selected_field == BuilderField::ImpactEncryptAlgoAes,
                        app.builder.encryption_algo == "AES",
                        true,
                    ));
                    list_items.push(create_item(
                        "   ChaCha20",
                        app.builder.selected_field == BuilderField::ImpactEncryptAlgoChacha,
                        app.builder.encryption_algo == "CHACHA",
                        true,
                    ));
                }

                list_items.push(create_item(
                    "wipe",
                    app.builder.selected_field == BuilderField::ImpactWipe,
                    app.builder.impact_wipe,
                    false,
                ));
            }

            f.render_widget(
                List::new(list_items).block(Block::default().borders(Borders::ALL)),
                chunks[1],
            );
        }
        BuilderCategory::Exfiltration => {
            let mut items = vec![(
                "enable exfiltration",
                app.builder.enable_exfil,
                BuilderField::ExfilToggle,
            )];

            if app.builder.enable_exfil {
                items.extend_from_slice(&[
                    (
                        "method: http",
                        app.builder.exfil_http,
                        BuilderField::ExfilHttp,
                    ),
                    ("method: dns", app.builder.exfil_dns, BuilderField::ExfilDns),
                ]);
            }

            render_checkbox_list(f, &app.builder.selected_field, items, chunks[1]);
        }
    }

    let btn_style = if app.builder.selected_field == BuilderField::Submit {
        Style::default()
            .bg(Color::Red)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().bg(Color::DarkGray).fg(Color::Gray)
    };

    f.render_widget(
        Paragraph::new("[ COMPILE PAYLOAD ]")
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).style(btn_style)),
        chunks[2],
    );

    let status_color = if app.builder.build_status_msg.contains("ERROR") {
        Color::Red
    } else if app.builder.build_status_msg.contains("SUCCESS") {
        Color::Green
    } else {
        Color::Cyan
    };

    f.render_widget(
        Paragraph::new(app.builder.build_status_msg.clone())
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .title(" BUILD OUTPUT "),
            )
            .style(Style::default().fg(status_color)),
        chunks[3],
    );
}

fn render_loot(f: &mut Frame, app: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(area);

    let border_color = if app.loot.search_mode {
        Color::Yellow
    } else {
        Color::DarkGray
    };
    let title = if app.loot.search_mode {
        "SEARCH EXFILTRATED DATA (TYPING) "
    } else {
        " SEARCH (Press '/' to type) "
    };
    let cursor = if app.loot.search_mode { "█" } else { "" };

    let search_input = Paragraph::new(format!("> {}{}", app.loot.search_query, cursor)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(title),
    );

    f.render_widget(search_input, chunks[0]);

    let filtered = app.loot.filtered_files();
    let items: Vec<ListItem> = filtered
        .iter()
        .map(|f_name| ListItem::new(Line::from(Span::raw(f_name))))
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" EXFILTRATED FILES (Press [ENTER] to download) "),
        )
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::REVERSED)
                .fg(Color::Yellow),
        )
        .highlight_symbol(">> ");

    let mut state = app.loot.list_state.clone();
    f.render_stateful_widget(list, chunks[1], &mut state);
}

fn render_checkbox_list(
    f: &mut Frame,
    selected: &BuilderField,
    items: Vec<(&str, bool, BuilderField)>,
    area: Rect,
) {
    let list_items: Vec<ListItem> = items
        .iter()
        .map(|(label, active, field)| {
            let check = if *active { "[x]" } else { "[ ]" };
            let content = format!("{} {}", check, label);
            let style = if *selected == *field {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(content).style(style)
        })
        .collect();

    f.render_widget(
        List::new(list_items).block(Block::default().borders(Borders::ALL)),
        area,
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
            "STATUS {} | [q] quit | [x] actions | [←/→] change tabs",
            app.status_message
        ))
        .style(status_style)
        .block(Block::default().borders(Borders::ALL)),
        area,
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
        Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title(" HELP "))
            .style(Style::default().bg(Color::DarkGray)),
        area,
    );
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
        Line::from(Span::styled(
            " [!] KILL SWITCH ",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("Press [ENTER] to confirm kill"),
        Line::from("Press [ESC] to cancer"),
    ];

    f.render_widget(
        Paragraph::new(text)
            .block(block)
            .alignment(Alignment::Center),
        area,
    );
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
