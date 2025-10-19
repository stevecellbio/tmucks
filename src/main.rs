use clap::Parser;

mod app;
mod cli;
mod config;
mod tui;

use cli::{Cli, Commands, ensure_conf_extension};
use config::ConfigManager;

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
        Some(Commands::Update { name }) => {
            let config_manager = ConfigManager::new()?;
            let config_name = ensure_conf_extension(name);
            config_manager.update_config(&config_name)?;
            println!("+ updated config: {}", config_name);
        }
        Some(Commands::Delete { name }) => {
            let config_manager = ConfigManager::new()?;
            let config_name = ensure_conf_extension(name);
            config_manager.delete_config(&config_name)?;
            println!("✓ Deleted config: {}", config_name);
        }
        None => {
            // No command provided, run TUI
            tui::run()?;
        }
    }

    Ok(())
}
