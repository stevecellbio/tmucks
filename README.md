# Tmucks - Tmux Config Manager

A Rust TUI application for managing and switching between multiple tmux configurations.

## Features

- 📁 Store multiple tmux configs in `~/.config/tmucks/`
- 🔄 Quick switching between configurations
- ⌨️ Simple keyboard navigation
- 🎨 Clean terminal UI with ratatui

## Installation

```bash
cargo build --release
cargo install --path .
```

## Usage

### TUI Mode (default)

Run without arguments to launch the interactive TUI:
```bash
tmucks
```

Keyboard shortcuts:
   - `↑/↓` - Navigate through configs
   - `Enter` - Apply selected config (copies to `~/.tmux.conf` and reloads tmux)
   - `s` - Save current config with a new name
   - `d` - Delete selected config
   - `q` - Quit
   - `Esc` - Cancel input (when entering a name)

### CLI Mode

Use commands for quick operations:

```bash
# List all saved configs
tmucks list

# Apply a config
tmucks apply work

# Save current config
tmucks save my-config

# Delete a config
tmucks delete old-config
```

## Setup

1. Create your config directory:
   ```bash
   mkdir -p ~/.config/tmucks
   ```

2. Add your tmux configs to `~/.config/tmucks/`:
   ```bash
   cp ~/.tmux.conf ~/.config/tmucks/default.conf
   # Add more configs as needed
   ```

3. Run tmucks to manage and switch between them!

## How it Works

When you apply a config:
1. Tmucks copies the selected file from `~/.config/tmucks/` to `~/.tmux.conf`
2. If tmux is running, it automatically reloads the config with `tmux source-file`

## License

MIT
