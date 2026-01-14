//! Event handling for the TUI.
//!
//! Processes keyboard input and updates application state accordingly.

use super::app::{ActivePanel, App};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handle a keyboard event and update the app state.
pub fn handle_key_event(app: &mut App, key: KeyEvent) {
    use super::app::ViewMode;

    // Handle help overlay first (it overlays everything)
    if app.show_help {
        if matches!(key.code, KeyCode::Char('?') | KeyCode::Esc | KeyCode::Char('q')) {
            app.toggle_help();
        }
        return;
    }

    // Handle search mode
    if app.mode == ViewMode::Search {
        match key.code {
            KeyCode::Char(c) => {
                app.search_input(c);
            }
            KeyCode::Backspace => {
                app.search_backspace();
            }
            KeyCode::Enter => {
                app.select_search_result();
            }
            KeyCode::Esc => {
                app.exit_search_mode();
            }
            _ => {}
        }
        return;
    }

    match key.code {
        // Quit
        KeyCode::Char('q') | KeyCode::Char('Q') => {
            app.quit();
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.quit();
        }
        KeyCode::Esc => {
            // Exit special modes
            if app.mode != ViewMode::Normal {
                app.mode = ViewMode::Normal;
            } else {
                app.quit();
            }
        }

        // Help overlay
        KeyCode::Char('?') => {
            app.toggle_help();
        }

        // Panel switching
        KeyCode::Char('g') if !key.modifiers.contains(KeyModifiers::SHIFT) => {
            app.panel = ActivePanel::Graph;
        }
        KeyCode::Char('G') => {
            app.panel = ActivePanel::Graph;
        }
        KeyCode::Char('d') if !key.modifiers.contains(KeyModifiers::SHIFT) => {
            // In normal mode, 'd' switches to Details panel
            // But we need to handle ViewMode::Dependents separately
            if app.mode == ViewMode::Normal {
                app.panel = ActivePanel::Details;
            }
        }
        KeyCode::Char('D') => {
            app.panel = ActivePanel::Details;
        }
        KeyCode::Char('a') if !key.modifiers.contains(KeyModifiers::SHIFT) => {
            app.panel = ActivePanel::Alerts;
        }
        KeyCode::Char('A') => {
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

        // Navigation (arrows and j/k)
        KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('K') => {
            match app.panel {
                ActivePanel::Graph => app.select_prev_in_layer(),
                ActivePanel::Details => app.scroll_details_up(),
                ActivePanel::Alerts => app.prev_alert(),
            }
        }
        KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') => {
            match app.panel {
                ActivePanel::Graph => app.select_next_in_layer(),
                ActivePanel::Details => app.scroll_details_down(),
                ActivePanel::Alerts => app.next_alert(),
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

        // Enter key
        KeyCode::Enter => {
            match app.panel {
                ActivePanel::Alerts => app.jump_to_selected_alert(),
                ActivePanel::Graph => app.switch_to_details(),
                _ => {}
            }
        }

        // View modes
        KeyCode::Char('i') | KeyCode::Char('I') => {
            app.enter_imports_mode();
        }
        // Note: 'd' conflicts with Details panel switch
        // We handle dependents mode with Shift+D in a custom way
        // Using 'w' (who imports this) for dependents instead
        KeyCode::Char('w') | KeyCode::Char('W') => {
            app.enter_dependents_mode();
        }

        // Search mode
        KeyCode::Char('/') => {
            app.enter_search_mode();
        }

        // Cycle navigation
        KeyCode::Char('c') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.jump_to_next_cycle();
        }

        // Toggle layer colors
        KeyCode::Char('l') | KeyCode::Char('L') => {
            app.toggle_layer_colors();
        }

        _ => {
            // Unhandled keys are ignored
        }
    }
}
