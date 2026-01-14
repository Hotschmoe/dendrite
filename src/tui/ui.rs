//! UI rendering for the TUI.
//!
//! Handles layout and drawing of the TUI interface using ratatui.

use super::app::App;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Render the TUI interface.
///
/// This is called every frame to draw the current state of the application.
pub fn render(f: &mut Frame, app: &App) {
    let size = f.area();

    // Create a centered block with instructions
    let block = Block::default()
        .title("Dendrite TUI")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black));

    let text = vec![
        Line::from(""),
        Line::from("Welcome to Dendrite TUI!"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ", Style::default()),
            Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(" to quit", Style::default()),
        ]),
        Line::from(""),
        Line::from(format!("Graph: {} nodes, {} edges", app.graph.node_count(), app.graph.edge_count())),
        Line::from(format!("Cycles: {}", app.analysis.cycles.len())),
        Line::from(format!("Violations: {}", app.analysis.violations.len())),
    ];

    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center);

    f.render_widget(paragraph, size);
}
