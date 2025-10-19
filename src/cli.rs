use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "tmucks")]
#[command(about = "Tmux config manager", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// List all saved configs
    List,
    /// Apply a config by name
    Apply { name: String },
    /// Save current tmux config with a name
    Save { name: String },
    /// Update an existing config with current tmux config
    Update { name: String },
    /// Delete a config by name
    Delete { name: String },
}

pub fn ensure_conf_extension(name: String) -> String {
    if name.ends_with(".conf") {
        name
    } else {
        format!("{}.conf", name)
    }
}
