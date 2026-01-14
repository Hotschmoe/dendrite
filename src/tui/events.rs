//! Event handling for the TUI.
//!
//! Processes keyboard input and updates application state accordingly.

use super::app::{ActivePanel, App};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handle a keyboard event and update the app state.
pub fn handle_key_event(app: &mut App, key: KeyEvent) {
    match key.code {
        // Quit
        KeyCode::Char('q') | KeyCode::Char('Q') => {
            app.quit();
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.quit();
        }
        KeyCode::Esc => {
            app.quit();
        }

        // Panel switching
        KeyCode::Char('g') | KeyCode::Char('G') => {
            app.panel = ActivePanel::Graph;
        }
        KeyCode::Char('d') | KeyCode::Char('D') => {
            app.panel = ActivePanel::Details;
        }
        KeyCode::Char('a') | KeyCode::Char('A') => {
            app.panel = ActivePanel::Alerts;
        }

        // Tab key to cycle through panels
        KeyCode::Tab => {
            app.panel = match app.panel {
                ActivePanel::Graph => ActivePanel::Details,
                ActivePanel::Details => ActivePanel::Alerts,
                ActivePanel::Alerts => ActivePanel::Graph,
            };
        }

        // Navigation (arrows)
        KeyCode::Up => {
            if app.panel == ActivePanel::Graph {
                app.select_prev_in_layer();
            }
        }
        KeyCode::Down => {
            if app.panel == ActivePanel::Graph {
                app.select_next_in_layer();
            }
        }
        KeyCode::Left => {
            if app.panel == ActivePanel::Graph {
                app.select_prev_layer();
            }
        }
        KeyCode::Right => {
            if app.panel == ActivePanel::Graph {
                app.select_next_layer();
            }
        }

        _ => {
            // Other keys will be handled in future milestones
        }
    }
}
