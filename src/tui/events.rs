//! Event handling for the TUI.
//!
//! Processes keyboard and mouse input and updates application state accordingly.

use super::app::{ActivePanel, App};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;

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
        KeyCode::Char('p') | KeyCode::Char('P') => {
            if app.mode == ViewMode::PathTrace {
                app.exit_path_trace();
            } else {
                app.trace_path_to_entry();
            }
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

/// Handle a mouse event and update the app state.
/// Requires layout information to determine which UI element was clicked.
pub fn handle_mouse_event(
    app: &mut App,
    mouse: MouseEvent,
    tab_bar_area: Rect,
    graph_area: Option<Rect>,
    details_area: Option<Rect>,
    alerts_area: Option<Rect>,
) {
    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            let (col, row) = (mouse.column, mouse.row);

            // Check if click was on tab bar
            if tab_bar_area.contains(ratatui::layout::Position { x: col, y: row }) {
                handle_tab_bar_click(app, col, tab_bar_area);
                return;
            }

            // Check if click was on graph panel
            if let Some(area) = graph_area {
                if area.contains(ratatui::layout::Position { x: col, y: row }) {
                    app.panel = ActivePanel::Graph;
                    return;
                }
            }

            // Check if click was on details panel
            if let Some(area) = details_area {
                if area.contains(ratatui::layout::Position { x: col, y: row }) {
                    app.panel = ActivePanel::Details;
                    return;
                }
            }

            // Check if click was on alerts panel
            if let Some(area) = alerts_area {
                if area.contains(ratatui::layout::Position { x: col, y: row }) {
                    app.panel = ActivePanel::Alerts;
                }
            }
        }
        MouseEventKind::ScrollUp => {
            // Scroll up in the current panel
            match app.panel {
                ActivePanel::Details => app.scroll_details_up(),
                ActivePanel::Alerts => app.prev_alert(),
                _ => {}
            }
        }
        MouseEventKind::ScrollDown => {
            // Scroll down in the current panel
            match app.panel {
                ActivePanel::Details => app.scroll_details_down(),
                ActivePanel::Alerts => app.next_alert(),
                _ => {}
            }
        }
        _ => {}
    }
}

/// Handle a click on the tab bar to switch panels.
fn handle_tab_bar_click(app: &mut App, col: u16, tab_bar_area: Rect) {
    // Tab bar format: "[G]raph | [D]etails | [A]lerts"
    // Approximate positions: Graph (0-8), Details (10-20), Alerts (22-30)
    let relative_col = col.saturating_sub(tab_bar_area.x);

    if relative_col < 8 {
        app.panel = ActivePanel::Graph;
    } else if (10..20).contains(&relative_col) {
        app.panel = ActivePanel::Details;
    } else if (22..30).contains(&relative_col) {
        app.panel = ActivePanel::Alerts;
    }
}
