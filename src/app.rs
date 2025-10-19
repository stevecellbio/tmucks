use crate::config::ConfigManager;
use ratatui::widgets::ListState;
use std::time::{Duration, Instant};

#[derive(PartialEq)]
pub enum InputMode {
    Normal,
    Saving,
    UpdateConfirm,
}

pub struct App {
    pub config_manager: ConfigManager,
    pub list_state: ListState,
    pub status_message: String,
    pub input_mode: InputMode,
    pub input_buffer: String,
    pub pending_update_config: Option<String>,
    pub status_message_time: Option<Instant>,
    pub default_status_message: String,
}

impl App {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config_manager = ConfigManager::new()?;
        let mut list_state = ListState::default();
        if !config_manager.configs.is_empty() {
            list_state.select(Some(0));
        }
        let default_status_message = String::from("use j/k to navigate, enter to apply config, s to save current, u to update existing, d to delete, q to quit");
        Ok(Self {
            config_manager,
            list_state,
            status_message: default_status_message.clone(),
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            pending_update_config: None,
            status_message_time: None,
            default_status_message,
        })
    }

    pub fn next(&mut self) {
        if self.config_manager.configs.is_empty() {
            return;
        }
        
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

    pub fn previous(&mut self) {
        if self.config_manager.configs.is_empty() {
            return;
        }
        
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

    pub fn apply_config(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(selected) = self.list_state.selected() {
            if let Some(config_name) = self.config_manager.configs.get(selected) {
                self.config_manager.apply_config(config_name)?;
                self.set_status_message(format!("+ applied config: {}", config_name));
            }
        }
        Ok(())
    }

    pub fn delete_config(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(selected) = self.list_state.selected() {
            if let Some(config_name) = self.config_manager.configs.get(selected).cloned() {
                self.config_manager.delete_config(&config_name)?;
                self.set_status_message(format!("+ deleted config: {}", config_name));
                
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

    pub fn save_current_config(&mut self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.config_manager.save_current_config(name)?;
        self.set_status_message(format!("+ saved current config as: {}", name));
        
        // Refresh config list
        self.config_manager = ConfigManager::new()?;
        if !self.config_manager.configs.is_empty() {
            self.list_state.select(Some(0));
        }
        Ok(())
    }

    pub fn start_update_mode(&mut self) {
        if let Some(selected) = self.list_state.selected() {
            if let Some(config_name) = self.config_manager.configs.get(selected) {
                self.pending_update_config = Some(config_name.clone());
                self.input_mode = InputMode::UpdateConfirm;
            } else {
                self.set_status_message(String::from("- no config selected to update"));
            }
        } else {
            self.set_status_message(String::from("- no config selected to update"));
        }
    }

    pub fn confirm_update(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(config_name) = self.pending_update_config.take() {
            self.config_manager.update_config(&config_name)?;
            self.set_status_message(format!("+ updated config '{}' with current ~/.tmux.conf", config_name));
        }
        self.input_mode = InputMode::Normal;
        Ok(())
    }

    pub fn cancel_update(&mut self) {
        self.pending_update_config = None;
        self.input_mode = InputMode::Normal;
        self.status_message = self.default_status_message.clone();
        self.status_message_time = None;
    }

    pub fn set_status_message(&mut self, message: String) {
        self.status_message = message;
        self.status_message_time = Some(Instant::now());
    }

    pub fn update_status_message(&mut self) {
        if let Some(message_time) = self.status_message_time {
            if message_time.elapsed() >= Duration::from_secs(5) {
                self.status_message = self.default_status_message.clone();
                self.status_message_time = None;
            }
        }
    }
}
