use crate::app::{App, InputMode};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
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
                            app.status_message = format!("✗ Error: {}", e);
                        }
                    }
                    KeyCode::Char('d') => {
                        if let Err(e) = app.delete_config() {
                            app.status_message = format!("✗ Error: {}", e);
                        }
                    }
                    KeyCode::Char('s') => {
                        app.input_mode = InputMode::Saving;
                        app.input_buffer.clear();
                        app.status_message = String::from("Enter config name (without .conf): ");
                    }
                    _ => {}
                },
                InputMode::Saving => match key.code {
                    KeyCode::Enter => {
                        if app.input_buffer.trim().is_empty() {
                            app.status_message = String::from("✗ Error: Name cannot be empty");
                            app.input_mode = InputMode::Normal;
                            app.input_buffer.clear();
                        } else {
                            let name = if app.input_buffer.ends_with(".conf") {
                                app.input_buffer.clone()
                            } else {
                                format!("{}.conf", app.input_buffer)
                            };
                            if let Err(e) = app.save_current_config(&name) {
                                app.status_message = format!("✗ Error: {}", e);
                            }
                            app.input_mode = InputMode::Normal;
                            app.input_buffer.clear();
                        }
                    }
                    KeyCode::Esc => {
                        app.input_mode = InputMode::Normal;
                        app.input_buffer.clear();
                        app.status_message = String::from("Use ↑/↓ to navigate, Enter to apply config, 's' to save current, 'd' to delete, 'q' to quit");
                    }
                    KeyCode::Char(c) => {
                        app.input_buffer.push(c);
                    }
                    KeyCode::Backspace => {
                        app.input_buffer.pop();
                    }
                    _ => {}
                },
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.size());

    // Title
    let title = Paragraph::new("Tmucks - Tmux Config Manager")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Config list
    if app.config_manager.configs.is_empty() {
        let empty_message = Paragraph::new("No configs found.\n\nAdd .conf files to ~/.config/tmucks/\nExample: cp ~/.tmux.conf ~/.config/tmucks/default.conf")
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::ALL).title("Configurations"));
        f.render_widget(empty_message, chunks[1]);
    } else {
        let items: Vec<ListItem> = app
            .config_manager
            .configs
            .iter()
            .map(|name| {
                let content = Line::from(vec![Span::raw(name)]);
                ListItem::new(content)
            })
            .collect();

        let items = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Configurations"))
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        f.render_stateful_widget(items, chunks[1], &mut app.list_state);
    }

    // Status bar / Input
    let status_text = match app.input_mode {
        InputMode::Normal => app.status_message.clone(),
        InputMode::Saving => format!("{}{}", app.status_message, app.input_buffer),
    };
    
    let status = Paragraph::new(status_text)
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("Status"));
    f.render_widget(status, chunks[2]);
}
