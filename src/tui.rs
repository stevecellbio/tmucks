use crate::app::{App, InputMode};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Padding, Wrap},
    Frame, Terminal,
};
use std::io;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run
    let mut app = App::new()?;
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error: {:?}", err);
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        app.update_status_message();
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            match app.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('j')=> app.next(),
                    KeyCode::Char('k') => app.previous(),
                    KeyCode::Down => app.next(),
                    KeyCode::Up => app.previous(),
                    KeyCode::Enter => {
                        if let Err(e) = app.apply_config() {
                            app.set_status_message(format!("- error: {}", e));
                        }
                    }
                    KeyCode::Char('d') => {
                        if let Err(e) = app.delete_config() {
                            app.set_status_message(format!("- error: {}", e));
                        }
                    }
                    KeyCode::Char('s') => {
                        app.input_mode = InputMode::Saving;
                        app.input_buffer.clear();
                        app.status_message = String::from("enter config name (without .conf): ");
                    }
                    KeyCode::Char('u') => {
                        app.start_update_mode();
                    }
                    _ => {}
                },
                InputMode::Saving => match key.code {
                    KeyCode::Enter => {
                        if app.input_buffer.trim().is_empty() {
                            app.set_status_message(String::from("- error: name cannot be empty"));
                            app.input_mode = InputMode::Normal;
                            app.input_buffer.clear();
                        } else {
                            let name = if app.input_buffer.ends_with(".conf") {
                                app.input_buffer.clone()
                            } else {
                                format!("{}.conf", app.input_buffer)
                            };
                            if let Err(e) = app.save_current_config(&name) {
                                app.set_status_message(format!("- error: {}", e));
                            }
                            app.input_mode = InputMode::Normal;
                            app.input_buffer.clear();
                        }
                    }
                    KeyCode::Esc => {
                        app.input_mode = InputMode::Normal;
                        app.input_buffer.clear();
                        app.status_message = app.default_status_message.clone();
                    }
                    KeyCode::Char(c) => {
                        app.input_buffer.push(c);
                    }
                    KeyCode::Backspace => {
                        app.input_buffer.pop();
                    }
                    _ => {}
                },
                InputMode::UpdateConfirm => match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        if let Err(e) = app.confirm_update() {
                            app.set_status_message(format!("- error: {}", e));
                        }
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                        app.cancel_update();
                    }
                    _ => {}
                },
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    // Create main layout with padding
    let main_area = f.size().inner(&Margin::new(2, 1));
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),  // Header
            Constraint::Min(0),     // Main content
            Constraint::Length(4),  // Footer/Status
        ])
        .split(main_area);

    // Header section with title and stats
    render_header(f, app, chunks[0]);

    // Main content area
    render_main_content(f, app, chunks[1]);

    // Footer status bar
    render_footer(f, app, chunks[2]);

    // Confirmation popup (rendered on top of everything)
    if app.input_mode == InputMode::UpdateConfirm {
        render_update_popup(f, app);
    }
}

fn render_header(f: &mut Frame, app: &mut App, area: Rect) {
    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(40),
            Constraint::Length(30),
        ])
        .split(area);

    // Title
    let title = Paragraph::new("tmux config manager")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .border_type(BorderType::Rounded)
                .title("tmucks")
                .title_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                )
        )
        .alignment(Alignment::Center);
    f.render_widget(title, header_chunks[0]);

    // Stats
    let stats_text = if app.config_manager.configs.is_empty() {
        vec![
            Line::from(Span::styled("no configs", Style::default().fg(Color::Red))),
        ]
    } else {
        let selected = app.list_state.selected().unwrap_or(0) + 1;
        vec![
            Line::from(vec![
                Span::styled("configs: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{}", app.config_manager.configs.len()),
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                ),
            ]),
            Line::from(vec![
                Span::styled("selected: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{}/{}", selected, app.config_manager.configs.len()),
                    Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)
                ),
            ]),
        ]
    };

    let stats = Paragraph::new(stats_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue))
                .border_type(BorderType::Rounded)
                .title("stats")
        )
        .alignment(Alignment::Center);
    f.render_widget(stats, header_chunks[1]);
}

fn render_main_content(f: &mut Frame, app: &mut App, area: Rect) {
    if app.config_manager.configs.is_empty() {
        let empty_content = vec![
            Line::from(""),
            Line::from(vec![
                Span::raw(" no configuration files found"),
            ]),
            Line::from(""),
            Line::from("add your first config:"),
            Line::from(vec![
                Span::styled("  $ ", Style::default().fg(Color::Cyan)),
                Span::styled("cp ~/.tmux.conf ~/.config/tmucks/default.conf", 
                    Style::default().fg(Color::Green)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("press ", Style::default().fg(Color::Yellow)),
                Span::styled("s", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(" to save current config"),
            ]),
        ];

        let empty_message = Paragraph::new(empty_content)
            .style(Style::default().fg(Color::Gray))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Gray))
                    .border_type(BorderType::Rounded)
                    .title(" configurations ")
                    .title_style(Style::default().fg(Color::Yellow))
            )
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        f.render_widget(empty_message, area);
    } else {
        let items: Vec<ListItem> = app
            .config_manager
            .configs
            .iter()
            .enumerate()
            .map(|(i, name)| {
                let is_selected = app.list_state.selected() == Some(i);
                let (icon, style) = if is_selected {
                    ("▶", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
                } else {
                    ("  ", Style::default().fg(Color::White))
                };

                let content = Line::from(vec![
                    Span::styled(icon, style),
                    Span::raw(" "),
                    Span::styled(
                        name,
                        Style::default().fg(if name.ends_with(".conf") { 
                            Color::Cyan 
                        } else { 
                            Color::White 
                        }).add_modifier(if is_selected { Modifier::BOLD } else { Modifier::empty() })
                    ),
                ]);
                ListItem::new(content)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Blue))
                    .border_type(BorderType::Rounded)
                    .title(" configurations ")
                    .title_style(Style::default().fg(Color::Yellow))
            )
            .highlight_style(
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("");

        f.render_stateful_widget(list, area, &mut app.list_state);
    }
}

fn render_footer(f: &mut Frame, app: &mut App, area: Rect) {
    let footer_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(30),
        ])
        .split(area);

    // Status / Input area
    let status_content = match app.input_mode {
        InputMode::Normal => app.status_message.clone(),
        InputMode::Saving => format!("save as: {}", app.input_buffer),
        InputMode::UpdateConfirm => {
            if let Some(config_name) = &app.pending_update_config {
                format!("update '{}' with current ~/.tmux.conf? (y/n)", config_name)
            } else {
                "no config selected for update".to_string()
            }
        }
    };

    let status_color = match app.input_mode {
        InputMode::UpdateConfirm => Color::Yellow,
        InputMode::Saving => Color::Green,
        InputMode::Normal => {
            if app.status_message.starts_with("+") {
                Color::Green
            } else if app.status_message.starts_with("-") {
                Color::Red
            } else {
                Color::Cyan
            }
        }
    };

    let status = Paragraph::new(status_content)
        .style(Style::default().fg(status_color))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(status_color))
                .border_type(BorderType::Rounded)
                .title(" status ")
                .title_style(Style::default().fg(Color::Yellow))
        );
    f.render_widget(status, footer_chunks[0]);

    // Help / Keybindings
    let help_text = if app.input_mode == InputMode::Normal {
        vec![
            Line::from(vec![
                Span::styled("j/k", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(" navigate "),
                Span::styled("enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw(" apply "),
                Span::styled("s", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(" save "),
                Span::styled("u", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
                Span::raw(" update "),
                Span::styled("d", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::raw(" delete "),
                Span::styled("q", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                Span::raw(" quit"),
            ])
        ]
    } else if app.input_mode == InputMode::Saving {
        vec![
            Line::from(vec![
                Span::styled("Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw(" save "),
                Span::styled("esc", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::raw(" cancel"),
            ])
        ]
    } else { // UpdateConfirm
        vec![
            Line::from(vec![
                Span::styled("y", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw(" confirm "),
                Span::styled("n/esc", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::raw(" cancel"),
            ])
        ]
    };

    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Gray))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Gray))
                .border_type(BorderType::Rounded)
                .title(" keys ")
                .title_style(Style::default().fg(Color::Yellow))
        )
        .alignment(Alignment::Center);
    f.render_widget(help, footer_chunks[1]);
}

fn render_update_popup(f: &mut Frame, app: &mut App) {
    let popup_area = centered_rect(60, 25, f.size());
    
    // Create a subtle background overlay
    let background = Block::default()
        .style(Style::default().bg(Color::Black));
    f.render_widget(background, f.size());
    
    // Popup content
    let popup_content = if let Some(config_name) = &app.pending_update_config {
        vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("confirm update", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("config: ", Style::default().fg(Color::Gray)),
                Span::styled(config_name, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from("this will overwrite the saved config with:"),
            Line::from(vec![
                Span::styled("~/.tmux.conf", Style::default().fg(Color::Green)),
            ]),
            Line::from(""),
            Line::from(""),
            Line::from(vec![
                Span::styled("[", Style::default().fg(Color::Gray)),
                Span::styled("y", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled("]es", Style::default().fg(Color::White)),
                Span::raw("  "),
                Span::styled("[", Style::default().fg(Color::Gray)),
                Span::styled("n", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::styled("]o", Style::default().fg(Color::White)),
                Span::raw("  "),
                Span::styled("[esc]", Style::default().fg(Color::Red)),
            ]),
        ]
    } else {
        vec![
            Line::from(""),
            Line::from("no config selected"),
            Line::from("press any key to continue"),
        ]
    };

    let popup = Paragraph::new(popup_content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
                .border_type(BorderType::Thick)
                .title(" confirmation ")
                .title_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                )
                .padding(Padding::new(1, 0, 1, 0))
        )
        .style(Style::default().bg(Color::Black).fg(Color::White))
        .alignment(Alignment::Center);
    
    f.render_widget(Clear, popup_area); // Clear the area for the popup
    f.render_widget(popup, popup_area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: ratatui::layout::Rect) -> ratatui::layout::Rect {
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
