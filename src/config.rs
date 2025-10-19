use std::{fs, path::PathBuf};

pub struct ConfigManager {
    pub configs: Vec<String>,
    config_dir: PathBuf,
    tmux_config_path: PathBuf,
}

impl ConfigManager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Get config directory: ~/.config/tmucks/
        let home = dirs::home_dir().ok_or("Could not find home directory")?;
        let config_dir = home.join(".config").join("tmucks");

        // Create config directory if it doesn't exist
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }

        // Get tmux config path
        let tmux_config_path = home.join(".tmux.conf");

        // Read available configs
        let configs = Self::read_configs(&config_dir)?;

        Ok(Self {
            configs,
            config_dir,
            tmux_config_path,
        })
    }

    fn read_configs(dir: &PathBuf) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut configs = Vec::new();

        if dir.exists() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_file() {
                    if let Some(name) = path.file_name() {
                        if let Some(name_str) = name.to_str() {
                            configs.push(name_str.to_string());
                        }
                    }
                }
            }
        }

        configs.sort();
        Ok(configs)
    }

    pub fn apply_config(&self, config_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let source_path = self.config_dir.join(config_name);

        if !source_path.exists() {
            return Err(format!("Config file not found: {}", config_name).into());
        }

        // Use cp command to copy the config
        fs::copy(&source_path, &self.tmux_config_path)?;

        // Reload tmux config if tmux is running
        if let Some(path_str) = self.tmux_config_path.to_str() {
            let _ = std::process::Command::new("tmux")
                .args(["source-file", path_str])
                .output();
        }

        Ok(())
    }

    pub fn delete_config(&self, config_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = self.config_dir.join(config_name);

        if !config_path.exists() {
            return Err(format!("Config file not found: {}", config_name).into());
        }

        fs::remove_file(config_path)?;
        Ok(())
    }

    pub fn save_current_config(&self, config_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        if !self.tmux_config_path.exists() {
            return Err("No tmux config file found at ~/.tmux.conf".into());
        }

        let dest_path = self.config_dir.join(config_name);
        
        // Check if config already exists
        if dest_path.exists() {
            return Err(format!("Config '{}' already exists. Use 'update' command to overwrite an existing config.", config_name).into());
        }
        
        fs::copy(&self.tmux_config_path, &dest_path)?;

        Ok(())
    }

    pub fn update_config(&self, config_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        if !self.tmux_config_path.exists() {
            return Err("No tmux config file found at ~/.tmux.conf".into());
        }

        let dest_path = self.config_dir.join(config_name);
        
        // Check if config exists
        if !dest_path.exists() {
            return Err(format!("Config '{}' does not exist. Use 'save' command to create a new config.", config_name).into());
        }
        
        // Copy current .tmux.conf to the selected config file (overwriting it)
        fs::copy(&self.tmux_config_path, &dest_path)?;

        Ok(())
    }
}
