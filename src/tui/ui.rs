//! UI rendering for the TUI.
//!
//! Handles layout and drawing of the TUI interface using ratatui.

use super::app::{ActivePanel, App, ViewMode};
use super::graph_view::GraphView;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs},
    Frame,
};

/// Render the TUI interface.
///
/// This is called every frame to draw the current state of the application.
pub fn render(f: &mut Frame, app: &App) {
    let size = f.area();

    // Split vertically: tab bar + main content + status bar
    let main_chunks = Layout::vertical([
        Constraint::Length(1), // Tab bar
        Constraint::Min(3),    // Main content area
        Constraint::Length(1), // Status bar
    ])
    .split(size);

    // Render tab bar at top
    render_tab_bar(f, app, main_chunks[0]);

    // Split main content horizontally based on terminal width and active panel
    if app.panel == ActivePanel::Graph && size.width >= 80 {
        // Show both graph and details panels side-by-side
        let content_chunks = Layout::horizontal([
            Constraint::Percentage(70), // Graph panel
            Constraint::Length(35),     // Details panel
        ])
        .split(main_chunks[1]);

        render_graph_panel(f, app, content_chunks[0]);
        render_details_panel(f, app, content_chunks[1]);
    } else {
        // Show single active panel in full width
        match app.panel {
            ActivePanel::Graph => {
                render_graph_panel(f, app, main_chunks[1]);
            }
            ActivePanel::Details => {
                render_details_panel(f, app, main_chunks[1]);
            }
            ActivePanel::Alerts => {
                render_alerts_panel(f, app, main_chunks[1]);
            }
        }
    }

    // Render status bar at bottom
    render_status_bar(f, app, main_chunks[2]);
}

/// Render the tab bar for panel switching.
fn render_tab_bar(f: &mut Frame, app: &App, area: Rect) {
    let titles = vec!["[G]raph", "[D]etails", "[A]lerts"];
    let selected = match app.panel {
        ActivePanel::Graph => 0,
        ActivePanel::Details => 1,
        ActivePanel::Alerts => 2,
    };

    let tabs = Tabs::new(titles)
        .select(selected)
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .divider(" | ");

    f.render_widget(tabs, area);
}

/// Render the graph panel showing the visual dependency graph.
fn render_graph_panel(f: &mut Frame, app: &App, area: Rect) {
    let title = format!(" Graph ({} files) ", app.graph.node_count());

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(if matches!(app.panel, ActivePanel::Graph) {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });

    // Render block first
    let inner_area = block.inner(area);
    f.render_widget(block, area);

    // Render the graph view
    let graph_view = GraphView::new(app, &app.layout);
    f.render_widget(graph_view, inner_area);
}

/// Render the details panel showing selected file information.
fn render_details_panel(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Details ")
        .borders(Borders::ALL)
        .border_style(if matches!(app.panel, ActivePanel::Details) {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });

    let text = if let Some(selected_idx) = app.selected_node {
        let node = &app.graph[selected_idx];

        let mut lines = vec![
            Line::from(vec![
                Span::styled("File: ", Style::default().bold()),
                Span::raw(&node.relative_path),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Layer: ", Style::default().bold()),
                Span::styled(
                    node.layer.to_string(),
                    Style::default().fg(layer_color(node.layer)),
                ),
            ]),
            Line::from(vec![
                Span::styled("LOC: ", Style::default().bold()),
                Span::raw(node.loc.to_string()),
            ]),
            Line::from(vec![
                Span::styled("Depth: ", Style::default().bold()),
                Span::raw(node.depth.to_string()),
            ]),
            Line::from(""),
        ];

        // Add imports count
        let imports_count = app
            .graph
            .neighbors_directed(selected_idx, petgraph::Direction::Outgoing)
            .count();
        lines.push(Line::from(vec![
            Span::styled("Imports: ", Style::default().bold()),
            Span::raw(imports_count.to_string()),
        ]));

        // Add dependents count
        let dependents_count = app
            .graph
            .neighbors_directed(selected_idx, petgraph::Direction::Incoming)
            .count();
        lines.push(Line::from(vec![
            Span::styled("Dependents: ", Style::default().bold()),
            Span::raw(dependents_count.to_string()),
        ]));

        // Add exports if any
        if !node.exports.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Exports:",
                Style::default().bold(),
            )));
            for export in node.exports.iter().take(5) {
                lines.push(Line::from(format!("  - {}", export)));
            }
            if node.exports.len() > 5 {
                lines.push(Line::from(format!(
                    "  ... and {} more",
                    node.exports.len() - 5
                )));
            }
        }

        // Add summary if available
        if let Some(summary) = &node.summary {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Summary:",
                Style::default().bold(),
            )));
            lines.push(Line::from(summary.as_str()));
        }

        lines
    } else {
        vec![
            Line::from(""),
            Line::from("No file selected"),
            Line::from(""),
            Line::from("Use arrow keys to"),
            Line::from("navigate files"),
        ]
    };

    let paragraph = Paragraph::new(text).block(block);

    f.render_widget(paragraph, area);
}

/// Render the alerts panel showing cycles and violations.
fn render_alerts_panel(f: &mut Frame, app: &App, area: Rect) {
    let cycles_count = app.analysis.cycles.len();
    let violations_count = app.analysis.violations.len();

    let title = format!(" Alerts ({} issues) ", cycles_count + violations_count);

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(if matches!(app.panel, ActivePanel::Alerts) {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });

    let mut lines = vec![];

    if cycles_count > 0 {
        lines.push(Line::from(Span::styled(
            format!("Cycles ({})", cycles_count),
            Style::default().fg(Color::Red).bold(),
        )));
        lines.push(Line::from(""));

        for (i, cycle) in app.analysis.cycles.iter().take(5).enumerate() {
            lines.push(Line::from(format!("{}. Cycle with {} files:", i + 1, cycle.nodes.len())));
            for node_path in cycle.nodes.iter().take(3) {
                lines.push(Line::from(format!("   - {}", node_path)));
            }
            if cycle.nodes.len() > 3 {
                lines.push(Line::from(format!("   ... and {} more", cycle.nodes.len() - 3)));
            }
            lines.push(Line::from(""));
        }

        if cycles_count > 5 {
            lines.push(Line::from(format!("... and {} more cycles", cycles_count - 5)));
            lines.push(Line::from(""));
        }
    }

    if violations_count > 0 {
        lines.push(Line::from(Span::styled(
            format!("Layer Violations ({})", violations_count),
            Style::default().fg(Color::Red).bold(),
        )));
        lines.push(Line::from(""));

        for (i, violation) in app.analysis.violations.iter().take(5).enumerate() {
            lines.push(Line::from(vec![
                Span::raw(format!("{}. ", i + 1)),
                Span::styled(&violation.file, Style::default().fg(Color::Cyan)),
            ]));
            lines.push(Line::from(format!(
                "   {} -> {} imports {}",
                violation.from_layer, violation.to_layer, violation.imports
            )));
            lines.push(Line::from(format!("   Reason: {}", violation.reason)));
            lines.push(Line::from(""));
        }

        if violations_count > 5 {
            lines.push(Line::from(format!(
                "... and {} more violations",
                violations_count - 5
            )));
        }
    }

    if cycles_count == 0 && violations_count == 0 {
        lines = vec![
            Line::from(""),
            Line::from(Span::styled(
                "No issues found!",
                Style::default().fg(Color::Green).bold(),
            )),
            Line::from(""),
            Line::from("Your codebase has:"),
            Line::from("  - No circular dependencies"),
            Line::from("  - No layer violations"),
        ];
    }

    let paragraph = Paragraph::new(lines).block(block);

    f.render_widget(paragraph, area);
}

/// Render the status bar at the bottom.
fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let cycles = app.analysis.cycles.len();
    let violations = app.analysis.violations.len();
    let total_issues = cycles + violations;

    // Left: file and edge counts
    let left_text = format!(
        "{} files, {} edges",
        app.graph.node_count(),
        app.graph.edge_count()
    );

    // Center: alert summary
    let center_text = if total_issues > 0 {
        format!("{} cycles, {} violations", cycles, violations)
    } else {
        "No issues".to_string()
    };

    let center_style = if total_issues > 0 {
        Style::default().fg(Color::Red).bold()
    } else {
        Style::default().fg(Color::Green)
    };

    // Right: help hint
    let right_text = "Press ? for help";

    // Build status bar with three sections
    let status_line = Line::from(vec![
        Span::raw(left_text),
        Span::raw(" | "),
        Span::styled(center_text, center_style),
        Span::raw(" "),
        Span::raw(" ".repeat(
            area.width
                .saturating_sub(
                    (app.graph.node_count().to_string().len()
                        + app.graph.edge_count().to_string().len()
                        + cycles.to_string().len()
                        + violations.to_string().len()
                        + right_text.len()
                        + 30) as u16,
                )
                .into(),
        )),
        Span::styled(right_text, Style::default().fg(Color::Gray)),
    ]);

    let paragraph = Paragraph::new(status_line);

    f.render_widget(paragraph, area);
}

/// Get the color for a layer.
fn layer_color(layer: crate::graph::Layer) -> Color {
    use crate::graph::Layer;
    match layer {
        Layer::Entry => Color::Magenta,
        Layer::App => Color::Cyan,
        Layer::Core => Color::Blue,
        Layer::Platform => Color::Green,
        Layer::Driver => Color::Yellow,
        Layer::Arch => Color::Red,
        Layer::Unknown => Color::Gray,
    }
}
