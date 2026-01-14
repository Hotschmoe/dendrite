//! Terminal User Interface (TUI) for interactive dependency graph exploration.
//!
//! This module provides an interactive terminal interface using ratatui and crossterm.
//! Users can navigate the dependency graph, inspect file details, and view analysis results.

mod app;
mod events;
mod graph_view;
mod layout;
mod ui;

pub use app::{App, ViewMode, ActivePanel};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::panic;

use crate::graph::analysis::AnalysisResult;
use crate::graph::DepGraph;

/// Setup the terminal for TUI rendering.
///
/// Enables raw mode, enters alternate screen, and enables mouse capture.
/// Returns a configured Terminal instance.
pub fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

/// Restore the terminal to normal state.
///
/// Disables raw mode, leaves alternate screen, and disables mouse capture.
pub fn restore_terminal(mut terminal: Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

/// Install a panic hook that restores the terminal before showing the panic message.
///
/// This ensures that if the application panics, the terminal is left in a usable state.
fn install_panic_hook() {
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
        original_hook(panic_info);
    }));
}

/// Run the TUI event loop.
///
/// This is the main entry point for the TUI. It sets up the terminal,
/// installs the panic hook, runs the event loop, and cleans up on exit.
pub fn run(mut app: App) -> io::Result<()> {
    install_panic_hook();

    let mut terminal = setup_terminal()?;

    loop {
        terminal.draw(|f| ui::render(f, &app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    events::handle_key_event(&mut app, key);
                }
                Event::Resize(_, _) => {
                    // Terminal was resized, next draw will handle it
                }
                _ => {}
            }
        }

        if app.should_quit {
            break;
        }
    }

    restore_terminal(terminal)
}
