use clap::{Parser, Subcommand};
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
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use std::io;

mod config;
use config::ConfigManager;

fn ensure_conf_extension(name: String) -> String {
    if name.ends_with(".conf") {
        name
    } else {
        format!("{}.conf", name)
    }
}

#[derive(Parser)]
#[command(name = "tmucks")]
#[command(about = "Tmux config manager", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// List all saved configs
    List,
    /// Apply a config by name
    Apply { name: String },
    /// Save current tmux config with a name
    Save { name: String },
    /// Delete a config by name
    Delete { name: String },
}

enum InputMode {
    Normal,
    Saving,
}

struct App {
    config_manager: ConfigManager,
    list_state: ListState,
    status_message: String,
    input_mode: InputMode,
    input_buffer: String,
}

impl App {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config_manager = ConfigManager::new()?;
        let mut list_state = ListState::default();
        if !config_manager.configs.is_empty() {
            list_state.select(Some(0));
        }
        Ok(Self {
            config_manager,
            list_state,
            status_message: String::from("Use ↑/↓ to navigate, Enter to apply config, 's' to save current, 'd' to delete, 'q' to quit"),
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
        })
    }

    fn next(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.config_manager.configs.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.config_manager.configs.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn apply_config(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(selected) = self.list_state.selected() {
            if let Some(config_name) = self.config_manager.configs.get(selected) {
                self.config_manager.apply_config(config_name)?;
                self.status_message = format!("✓ Applied config: {}", config_name);
            }
        }
        Ok(())
    }

    fn delete_config(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(selected) = self.list_state.selected() {
            if let Some(config_name) = self.config_manager.configs.get(selected).cloned() {
                self.config_manager.delete_config(&config_name)?;
                self.status_message = format!("✓ Deleted config: {}", config_name);
                
                // Refresh config list
                self.config_manager = ConfigManager::new()?;
                if self.config_manager.configs.is_empty() {
                    self.list_state.select(None);
                } else if selected >= self.config_manager.configs.len() {
                    self.list_state.select(Some(self.config_manager.configs.len() - 1));
                }
            }
        }
        Ok(())
    }

    fn save_current_config(&mut self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.config_manager.save_current_config(name)?;
        self.status_message = format!("✓ Saved current config as: {}", name);
        
        // Refresh config list
        self.config_manager = ConfigManager::new()?;
        if !self.config_manager.configs.is_empty() {
            self.list_state.select(Some(0));
        }
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::List) => {
            let config_manager = ConfigManager::new()?;
            if config_manager.configs.is_empty() {
                println!("No configs found in ~/.config/tmucks/");
            } else {
                println!("Available configs:");
                for config in &config_manager.configs {
                    println!("  - {}", config);
                }
            }
        }
        Some(Commands::Apply { name }) => {
            let config_manager = ConfigManager::new()?;
            let config_name = ensure_conf_extension(name);
            config_manager.apply_config(&config_name)?;
            println!("✓ Applied config: {}", config_name);
        }
        Some(Commands::Save { name }) => {
            let config_manager = ConfigManager::new()?;
            let config_name = ensure_conf_extension(name);
            config_manager.save_current_config(&config_name)?;
            println!("✓ Saved current config as: {}", config_name);
        }
        Some(Commands::Delete { name }) => {
            let config_manager = ConfigManager::new()?;
            let config_name = ensure_conf_extension(name);
            config_manager.delete_config(&config_name)?;
            println!("✓ Deleted config: {}", config_name);
        }
        None => {
            // No command provided, run TUI
            run_tui()?;
        }
    }

    Ok(())
}

fn run_tui() -> Result<(), Box<dyn std::error::Error>> {
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
