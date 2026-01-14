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

    // Split vertically: tab bar + main content + status bar + search bar (if active)
    let main_chunks = if app.mode == ViewMode::Search {
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

    // Render search bar if in search mode
    if app.mode == ViewMode::Search {
        render_search_bar(f, app, main_chunks[3]);
    }

    // Render help overlay on top if active
    if app.show_help {
        render_help_overlay(f, app, size);
    }
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

        let mut lines = vec![];

        // Check if this file is in a cycle
        let in_cycle = app.analysis.cycles.iter().any(|cycle| {
            cycle.nodes.contains(&node.relative_path)
        });

        if in_cycle {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "  WARNING: This file is in a cycle!  ",
                Style::default().fg(Color::Red).bold().bg(Color::DarkGray),
            )));
            lines.push(Line::from(""));
        }

        // File path and full path
        lines.push(Line::from(vec![
            Span::styled("File: ", Style::default().bold()),
            Span::raw(&node.relative_path),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Path: ", Style::default().bold()),
            Span::styled(
                node.path.display().to_string(),
                Style::default().fg(Color::DarkGray),
            ),
        ]));
        lines.push(Line::from(""));

        // Layer and metadata
        lines.push(Line::from(vec![
            Span::styled("Layer: ", Style::default().bold()),
            Span::styled(
                node.layer.to_string(),
                Style::default().fg(layer_color(node.layer)),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Depth: ", Style::default().bold()),
            Span::raw(node.depth.to_string()),
        ]));
        lines.push(Line::from(vec![
            Span::styled("LOC: ", Style::default().bold()),
            Span::raw(node.loc.to_string()),
        ]));
        lines.push(Line::from(""));

        // Imports list
        let imports: Vec<_> = app
            .graph
            .neighbors_directed(selected_idx, petgraph::Direction::Outgoing)
            .collect();
        lines.push(Line::from(vec![
            Span::styled("Imports (", Style::default().bold()),
            Span::styled(imports.len().to_string(), Style::default().bold()),
            Span::styled("):", Style::default().bold()),
        ]));

        if imports.is_empty() {
            lines.push(Line::from("  (none)"));
        } else {
            for import_idx in imports.iter().take(10) {
                let import_node = &app.graph[*import_idx];
                lines.push(Line::from(vec![
                    Span::raw("  - "),
                    Span::styled(
                        &import_node.relative_path,
                        Style::default().fg(Color::Cyan),
                    ),
                ]));
            }
            if imports.len() > 10 {
                lines.push(Line::from(format!("  ... and {} more", imports.len() - 10)));
            }
        }
        lines.push(Line::from(""));

        // Dependents list
        let dependents: Vec<_> = app
            .graph
            .neighbors_directed(selected_idx, petgraph::Direction::Incoming)
            .collect();
        lines.push(Line::from(vec![
            Span::styled("Dependents (", Style::default().bold()),
            Span::styled(dependents.len().to_string(), Style::default().bold()),
            Span::styled("):", Style::default().bold()),
        ]));

        if dependents.is_empty() {
            lines.push(Line::from("  (none)"));
        } else {
            for dependent_idx in dependents.iter().take(10) {
                let dependent_node = &app.graph[*dependent_idx];
                lines.push(Line::from(vec![
                    Span::raw("  - "),
                    Span::styled(
                        &dependent_node.relative_path,
                        Style::default().fg(Color::Magenta),
                    ),
                ]));
            }
            if dependents.len() > 10 {
                lines.push(Line::from(format!("  ... and {} more", dependents.len() - 10)));
            }
        }
        lines.push(Line::from(""));

        // Exports
        if !node.exports.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("Exports (", Style::default().bold()),
                Span::styled(node.exports.len().to_string(), Style::default().bold()),
                Span::styled("):", Style::default().bold()),
            ]));
            for export in node.exports.iter().take(10) {
                lines.push(Line::from(format!("  - {}", export)));
            }
            if node.exports.len() > 10 {
                lines.push(Line::from(format!(
                    "  ... and {} more",
                    node.exports.len() - 10
                )));
            }
            lines.push(Line::from(""));
        }

        // Summary/Doc comment
        if let Some(summary) = &node.summary {
            lines.push(Line::from(Span::styled(
                "Summary:",
                Style::default().bold(),
            )));
            // Wrap long summaries
            for line in summary.lines().take(5) {
                lines.push(Line::from(format!("  {}", line)));
            }
            if summary.lines().count() > 5 {
                lines.push(Line::from("  ..."));
            }
        }

        // Apply scroll offset
        if app.details_scroll > 0 && app.details_scroll < lines.len() {
            lines = lines.into_iter().skip(app.details_scroll).collect();
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
    let total_issues = cycles_count + violations_count;

    let title = format!(" Alerts ({} issues) ", total_issues);

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(if matches!(app.panel, ActivePanel::Alerts) {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });

    if total_issues == 0 {
        let lines = vec![
            Line::from(""),
            Line::from(Span::styled(
                "No issues found!",
                Style::default().fg(Color::Green).bold(),
            )),
            Line::from(""),
            Line::from("Your codebase has:"),
            Line::from("  - No circular dependencies"),
            Line::from("  - No layer violations"),
            Line::from(""),
            Line::from("Additional metrics:"),
            Line::from(format!("  - Max depth: {}", app.analysis.max_depth)),
            Line::from(format!("  - Entry points: {}", app.analysis.entry_points.len())),
            Line::from(format!("  - High fan-out files: {}", app.analysis.high_fan_out.len())),
            Line::from(format!("  - High fan-in files: {}", app.analysis.high_fan_in.len())),
        ];

        let paragraph = Paragraph::new(lines).block(block);
        f.render_widget(paragraph, area);
        return;
    }

    let mut items = vec![];
    let mut alert_index = 0;

    // Cycles section
    if cycles_count > 0 {
        items.push(ListItem::new(Line::from(Span::styled(
            format!("CYCLES ({})", cycles_count),
            Style::default().fg(Color::Red).bold(),
        ))));
        items.push(ListItem::new(Line::from("")));

        for (i, cycle) in app.analysis.cycles.iter().enumerate() {
            let is_selected = app.panel == ActivePanel::Alerts && app.selected_alert == alert_index;
            let style = if is_selected {
                Style::default().bg(Color::DarkGray).bold()
            } else {
                Style::default()
            };

            // Cycle header
            let cycle_path = cycle.nodes.join(" -> ");
            let short_path = if cycle_path.len() > 60 {
                format!("{}...", &cycle_path[..57])
            } else {
                cycle_path
            };

            items.push(ListItem::new(Line::from(vec![
                Span::styled(format!("{}. ", i + 1), style),
                Span::styled(short_path, style.fg(Color::Red)),
            ])));

            // Show files in cycle
            for node_path in cycle.nodes.iter().take(4) {
                items.push(ListItem::new(Line::from(vec![
                    Span::styled("   ", style),
                    Span::styled(node_path, style.fg(Color::Cyan)),
                ])));
            }
            if cycle.nodes.len() > 4 {
                items.push(ListItem::new(Line::from(Span::styled(
                    format!("   ... and {} more", cycle.nodes.len() - 4),
                    style,
                ))));
            }

            // Fix suggestion
            if let Some((from, _to, line)) = cycle.edges.first() {
                items.push(ListItem::new(Line::from(vec![
                    Span::styled("   Fix: ", style.fg(Color::Yellow)),
                    Span::styled(format!("Remove import at {}:{}", from, line), style),
                ])));
            }

            items.push(ListItem::new(Line::from("")));
            alert_index += 1;
        }
    }

    // Violations section
    if violations_count > 0 {
        items.push(ListItem::new(Line::from(Span::styled(
            format!("LAYER VIOLATIONS ({})", violations_count),
            Style::default().fg(Color::Red).bold(),
        ))));
        items.push(ListItem::new(Line::from("")));

        for (i, violation) in app.analysis.violations.iter().enumerate() {
            let is_selected = app.panel == ActivePanel::Alerts && app.selected_alert == alert_index;
            let style = if is_selected {
                Style::default().bg(Color::DarkGray).bold()
            } else {
                Style::default()
            };

            items.push(ListItem::new(Line::from(vec![
                Span::styled(format!("{}. ", i + 1), style),
                Span::styled(&violation.file, style.fg(Color::Cyan)),
                Span::styled(format!(" (line {})", violation.line), style.fg(Color::DarkGray)),
            ])));

            items.push(ListItem::new(Line::from(vec![
                Span::styled("   ", style),
                Span::styled(
                    format!("{} ", violation.from_layer),
                    style.fg(layer_color(violation.from_layer)),
                ),
                Span::styled("imports ", style),
                Span::styled(
                    format!("{} ", violation.to_layer),
                    style.fg(layer_color(violation.to_layer)),
                ),
                Span::styled(format!("({})", violation.imports), style.fg(Color::DarkGray)),
            ])));

            items.push(ListItem::new(Line::from(vec![
                Span::styled("   Reason: ", style),
                Span::styled(&violation.reason, style.fg(Color::Gray)),
            ])));

            items.push(ListItem::new(Line::from(vec![
                Span::styled("   Fix: ", style.fg(Color::Yellow)),
                Span::styled(violation.fix_suggestion(), style),
            ])));

            items.push(ListItem::new(Line::from("")));
            alert_index += 1;
        }
    }

    // Threshold warnings section
    let high_fan_out_count = app.analysis.high_fan_out.len();
    let high_fan_in_count = app.analysis.high_fan_in.len();

    if high_fan_out_count > 0 || high_fan_in_count > 0 {
        items.push(ListItem::new(Line::from(Span::styled(
            "THRESHOLD WARNINGS",
            Style::default().fg(Color::Yellow).bold(),
        ))));
        items.push(ListItem::new(Line::from("")));

        for (file, count) in app.analysis.high_fan_out.iter().take(3) {
            items.push(ListItem::new(Line::from(vec![
                Span::styled("  High fan-out: ", Style::default().fg(Color::Yellow)),
                Span::styled(file, Style::default().fg(Color::Cyan)),
                Span::styled(format!(" ({} imports)", count), Style::default()),
            ])));
        }

        for (file, count) in app.analysis.high_fan_in.iter().take(3) {
            items.push(ListItem::new(Line::from(vec![
                Span::styled("  High fan-in: ", Style::default().fg(Color::Yellow)),
                Span::styled(file, Style::default().fg(Color::Cyan)),
                Span::styled(format!(" ({} dependents)", count), Style::default()),
            ])));
        }
    }

    let list = List::new(items).block(block);

    f.render_widget(list, area);
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

    // Center: alert summary or mode indicator
    let (center_text, center_style) = match app.mode {
        ViewMode::Normal => {
            if total_issues > 0 {
                (
                    format!("{} cycles, {} violations", cycles, violations),
                    Style::default().fg(Color::Red).bold(),
                )
            } else {
                (
                    "No issues".to_string(),
                    Style::default().fg(Color::Green),
                )
            }
        }
        ViewMode::Imports => (
            "Mode: IMPORTS".to_string(),
            Style::default().fg(Color::Cyan).bold(),
        ),
        ViewMode::Dependents => (
            "Mode: DEPENDENTS".to_string(),
            Style::default().fg(Color::Magenta).bold(),
        ),
        ViewMode::Search => (
            "Mode: SEARCH".to_string(),
            Style::default().fg(Color::Yellow).bold(),
        ),
        ViewMode::PathTrace => (
            format!(
                "Mode: PATH TRACE ({} steps)",
                app.traced_path.len().saturating_sub(1)
            ),
            Style::default().fg(Color::Green).bold(),
        ),
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

/// Render the search bar at the bottom.
fn render_search_bar(f: &mut Frame, app: &App, area: Rect) {
    let search_text = format!(
        "Search: {} ({} results)",
        app.search_query,
        app.search_results.len()
    );

    let paragraph = Paragraph::new(search_text)
        .style(Style::default().fg(Color::Yellow).bg(Color::DarkGray));

    f.render_widget(paragraph, area);
}

/// Render the help overlay.
fn render_help_overlay(f: &mut Frame, _app: &App, area: Rect) {
    // Calculate centered popup area
    let popup_width = area.width.min(80);
    let popup_height = area.height.min(30);
    let x = (area.width.saturating_sub(popup_width)) / 2;
    let y = (area.height.saturating_sub(popup_height)) / 2;

    let popup_area = Rect {
        x: area.x + x,
        y: area.y + y,
        width: popup_width,
        height: popup_height,
    };

    // Help content
    let help_lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "DENDRITE - KEYBOARD SHORTCUTS",
            Style::default().fg(Color::Yellow).bold(),
        )),
        Line::from(""),
        Line::from(Span::styled("Navigation:", Style::default().bold())),
        Line::from("  Arrow keys / j/k  Navigate nodes in current layer"),
        Line::from("  Left/Right        Move between layers"),
        Line::from("  Enter             View file details (from Graph)"),
        Line::from("  Tab               Cycle through panels"),
        Line::from(""),
        Line::from(Span::styled("Panels:", Style::default().bold())),
        Line::from("  g                 Go to Graph panel"),
        Line::from("  d                 Go to Details panel"),
        Line::from("  a                 Go to Alerts panel"),
        Line::from(""),
        Line::from(Span::styled("View Modes:", Style::default().bold())),
        Line::from("  i                 Toggle Imports mode (highlight what file imports)"),
        Line::from("  w                 Toggle Dependents mode (who imports this file)"),
        Line::from("  p                 Toggle Path trace mode (path to entry point)"),
        Line::from("  /                 Enter search mode"),
        Line::from("  Esc               Exit view mode / Return to normal"),
        Line::from(""),
        Line::from(Span::styled("Special Navigation:", Style::default().bold())),
        Line::from("  c                 Jump to next cycle node"),
        Line::from(""),
        Line::from(Span::styled("Display:", Style::default().bold())),
        Line::from("  l                 Toggle layer colors"),
        Line::from("  ?                 Toggle this help"),
        Line::from(""),
        Line::from(Span::styled("Mouse Support:", Style::default().bold())),
        Line::from("  Click             Switch panels or focus clicked area"),
        Line::from("  Scroll wheel      Scroll details/alerts panels"),
        Line::from(""),
        Line::from(Span::styled("Quit:", Style::default().bold())),
        Line::from("  q / Esc           Quit application"),
        Line::from("  Ctrl+C            Force quit"),
        Line::from(""),
        Line::from(Span::styled(
            "Press ? or Esc to close this help",
            Style::default().fg(Color::Gray),
        )),
    ];

    let block = Block::default()
        .title(" Help ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .style(Style::default().bg(Color::Black));

    let paragraph = Paragraph::new(help_lines).block(block);

    // Clear the area behind the popup
    f.render_widget(ratatui::widgets::Clear, popup_area);
    f.render_widget(paragraph, popup_area);
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
