use crate::config::ConfigManager;
use ratatui::widgets::ListState;

pub enum InputMode {
    Normal,
    Saving,
}

pub struct App {
    pub config_manager: ConfigManager,
    pub list_state: ListState,
    pub status_message: String,
    pub input_mode: InputMode,
    pub input_buffer: String,
}

impl App {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config_manager = ConfigManager::new()?;
        let mut list_state = ListState::default();
        if !config_manager.configs.is_empty() {
            list_state.select(Some(0));
        }
        Ok(Self {
            config_manager,
            list_state,
            status_message: String::from("Use j/k to navigate, Enter to apply config, 's' to save current, 'd' to delete, 'q' to quit"),
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
        })
    }

    pub fn next(&mut self) {
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
                self.status_message = format!("✓ Applied config: {}", config_name);
            }
        }
        Ok(())
    }

    pub fn delete_config(&mut self) -> Result<(), Box<dyn std::error::Error>> {
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

    pub fn save_current_config(&mut self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
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
