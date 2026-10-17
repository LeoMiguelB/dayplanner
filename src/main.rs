mod app;
mod config;
mod db;
mod event;
mod model;
mod ui;

use std::io::stdout;
use std::path::PathBuf;
use clap::Parser;
use crossterm::{
    cursor::{Hide, Show},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::app::App;
use crate::event::EventHandler;

#[derive(Parser, Debug)]
#[command(name = "dayplanner")]
#[command(author = "Dayplanner Authors")]
#[command(version = "0.1.0")]
#[command(about = "A high-performance Day Planner TUI built with Ratatui and native Vim motions")]
struct Cli {
    /// Path to SQLite database file
    #[arg(short, long)]
    db: Option<PathBuf>,

    /// Run with an ephemeral in-memory or temporary database in /tmp
    #[arg(short, long)]
    temp: bool,
}

fn setup_panic_hook() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen, Show);
        original_hook(panic_info);
    }));
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    setup_panic_hook();

    let db_path = if cli.temp {
        Some(config::ConfigManager::temp_fallback_path())
    } else {
        cli.db
    };

    let mut app = App::new(db_path);
    let mut event_handler = EventHandler::new();

    // Terminal Initialization
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, Hide)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Main TUI Event Loop
    while !app.should_quit {
        terminal.draw(|f| ui::render(f, &app))?;
        event_handler.handle_events(&mut app)?;
    }

    // Terminal Teardown
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, Show)?;
    terminal.show_cursor()?;

    Ok(())
}
