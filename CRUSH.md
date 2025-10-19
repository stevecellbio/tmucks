# tmucks - Development Commands & Style Guide

## Build/Test Commands
- `cargo build` - Build the project
- `cargo run` - Run the application
- `cargo run -- --help` - Show CLI options
- `cargo test` - Run all tests
- `cargo test <test_name>` - Run specific test
- `cargo clippy` - Lint with Clippy
- `cargo fmt` - Format code

## Code Style Guidelines

### General
- Use `Box<dyn std::error::Error>` for error handling
- Import external crates first, then local modules
- Use `Result<(), Box<dyn std::error::Error>>` for fallible functions

### Naming Conventions
- Use snake_case for variables and functions
- Use PascalCase for types and structs
- Use SCREAMING_SNAKE_CASE for constants
- Module names follow snake_case

### Imports
- Group imports: std imports first, then external crates, then local modules
- Use `use crate::module::item;` for internal imports
- Keep imports at file top, ordered alphabetically within groups

### Structure
- Main logic in `main.rs`, split into modules: `app`, `cli`, `config`, `tui`
- Use `impl` blocks for methods
- Keep functions small and focused
- Use derive macros where appropriate (`#[derive(Parser)]`, `#[derive(Subcommand)]`)

### Error Handling
- Use `?` operator for error propagation
- Return descriptive error messages as strings
- Handle file operations with proper existence checks

### TUI Specific
- Use ratatui for terminal UI
- Handle keyboard events with match statements
- Use `InputMode` enum for different UI states
- Keep UI rendering separate from app logic