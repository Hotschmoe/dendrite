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

    // Track layout areas for mouse event handling
    let mut layout_cache: Option<LayoutCache> = None;

    loop {
        terminal.draw(|f| {
            // Cache layout areas for mouse handling
            layout_cache = Some(capture_layout(f, &app));
            ui::render(f, &app);
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    events::handle_key_event(&mut app, key);
                }
                Event::Mouse(mouse) => {
                    if let Some(ref cache) = layout_cache {
                        events::handle_mouse_event(
                            &mut app,
                            mouse,
                            cache.tab_bar,
                            cache.graph_area,
                            cache.details_area,
                            cache.alerts_area,
                        );
                    }
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

/// Cache of layout areas for mouse event handling.
struct LayoutCache {
    tab_bar: ratatui::layout::Rect,
    graph_area: Option<ratatui::layout::Rect>,
    details_area: Option<ratatui::layout::Rect>,
    alerts_area: Option<ratatui::layout::Rect>,
}

/// Capture layout areas from the frame for mouse event handling.
fn capture_layout(f: &ratatui::Frame, app: &App) -> LayoutCache {
    use ratatui::layout::{Constraint, Layout};

    let size = f.area();

    // Replicate the layout logic from ui::render
    let main_chunks = if app.mode == app::ViewMode::Search {
        Layout::vertical([
            Constraint::Length(1), // Tab bar
            Constraint::Min(3),    // Main content area
            Constraint::Length(1), // Status bar
            Constraint::Length(1), // Search bar
        ])
        .split(size)
    } else {
        Layout::vertical([
            Constraint::Length(1), // Tab bar
            Constraint::Min(3),    // Main content area
            Constraint::Length(1), // Status bar
        ])
        .split(size)
    };

    let tab_bar = main_chunks[0];
    let content_area = main_chunks[1];

    // Determine which panels are visible
    let (graph_area, details_area, alerts_area) = if app.panel == app::ActivePanel::Graph && size.width >= 80 {
        // Show both graph and details panels side-by-side
        let content_chunks = Layout::horizontal([
            Constraint::Percentage(70), // Graph panel
            Constraint::Length(35),     // Details panel
        ])
        .split(content_area);

        (Some(content_chunks[0]), Some(content_chunks[1]), None)
    } else {
        // Show single active panel in full width
        match app.panel {
            app::ActivePanel::Graph => (Some(content_area), None, None),
            app::ActivePanel::Details => (None, Some(content_area), None),
            app::ActivePanel::Alerts => (None, None, Some(content_area)),
        }
    };

    LayoutCache {
        tab_bar,
        graph_area,
        details_area,
        alerts_area,
    }
}
