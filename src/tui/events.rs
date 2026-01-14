//! Event handling for the TUI.
//!
//! Processes keyboard input and updates application state accordingly.

use super::app::App;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handle a keyboard event and update the app state.
pub fn handle_key_event(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') => {
            app.quit();
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.quit();
        }
        _ => {
            // Other keys will be handled in future milestones
        }
    }
}
