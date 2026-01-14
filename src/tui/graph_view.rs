//! Graph rendering for the TUI.
//!
//! Renders the dependency graph as ASCII art with boxes and arrows.

use super::app::App;
use super::layout::GraphLayout;
use crate::graph::Layer;
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use petgraph::Direction;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};
use std::collections::HashSet;

/// Widget for rendering the graph view.
pub struct GraphView<'a> {
    app: &'a App,
    layout: &'a GraphLayout,
}

impl<'a> GraphView<'a> {
    pub fn new(app: &'a App, layout: &'a GraphLayout) -> Self {
        Self { app, layout }
    }

    /// Get nodes that are part of cycles.
    fn get_cycle_nodes(&self) -> HashSet<String> {
        let mut cycle_nodes = HashSet::new();
        for cycle in &self.app.analysis.cycles {
            for node in &cycle.nodes {
                cycle_nodes.insert(node.clone());
            }
        }
        cycle_nodes
    }

    /// Draw a box for a file node.
    fn draw_node(
        &self,
        buffer: &mut Buffer,
        node_idx: NodeIndex,
        x: u16,
        y: u16,
        area: Rect,
        cycle_nodes: &HashSet<String>,
    ) {
        let node = &self.app.graph[node_idx];
        let is_selected = self.app.selected_node == Some(node_idx);
        let in_cycle = cycle_nodes.contains(&node.relative_path);

        // Determine box style
        let border_color = if in_cycle {
            Color::Red
        } else if is_selected {
            Color::Yellow
        } else {
            layer_color(node.layer)
        };

        let border_style = if is_selected {
            Style::default()
                .fg(border_color)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(border_color)
        };

        let text_style = if is_selected {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else if in_cycle {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::White)
        };

        // Check if node is visible in the viewport
        let node_width = self.layout.node_width;

        if x >= area.width || y >= area.height {
            return;
        }

        // Draw top border
        if y < area.height {
            let top_left = '\u{250C}'; // ┌
            let top_right = '\u{2510}'; // ┐
            let horizontal = '\u{2500}'; // ─

            if x < area.width {
                buffer[(area.x + x, area.y + y)]
                    .set_char(top_left)
                    .set_style(border_style);
            }

            for i in 1..node_width.saturating_sub(1) {
                if x + i < area.width {
                    buffer[(area.x + x + i, area.y + y)]
                        .set_char(horizontal)
                        .set_style(border_style);
                }
            }

            if x + node_width - 1 < area.width {
                buffer[(area.x + x + node_width - 1, area.y + y)]
                    .set_char(top_right)
                    .set_style(border_style);
            }
        }

        // Draw middle section with text
        if y + 1 < area.height {
            let vertical = '\u{2502}'; // │

            if x < area.width {
                buffer[(area.x + x, area.y + y + 1)]
                    .set_char(vertical)
                    .set_style(border_style);
            }

            // Truncate filename to fit in box
            let max_text_width = node_width.saturating_sub(4) as usize;
            let display_name = truncate_filename(&node.relative_path, max_text_width);

            for (i, ch) in display_name.chars().enumerate() {
                let text_x = x + 2 + i as u16;
                if text_x < area.width.min(x + node_width - 2) && text_x < area.width {
                    buffer[(area.x + text_x, area.y + y + 1)]
                        .set_char(ch)
                        .set_style(text_style);
                }
            }

            // Fill remaining space with background color if selected
            if is_selected {
                for i in (2 + display_name.len() as u16)..(node_width - 2) {
                    if x + i < area.width {
                        buffer[(area.x + x + i, area.y + y + 1)]
                            .set_char(' ')
                            .set_style(text_style);
                    }
                }
            }

            if x + node_width - 1 < area.width {
                buffer[(area.x + x + node_width - 1, area.y + y + 1)]
                    .set_char(vertical)
                    .set_style(border_style);
            }
        }

        // Draw bottom border
        if y + 2 < area.height {
            let bottom_left = '\u{2514}'; // └
            let bottom_right = '\u{2518}'; // ┘
            let horizontal = '\u{2500}'; // ─

            if x < area.width {
                buffer[(area.x + x, area.y + y + 2)]
                    .set_char(bottom_left)
                    .set_style(border_style);
            }

            for i in 1..node_width.saturating_sub(1) {
                if x + i < area.width {
                    buffer[(area.x + x + i, area.y + y + 2)]
                        .set_char(horizontal)
                        .set_style(border_style);
                }
            }

            if x + node_width - 1 < area.width {
                buffer[(area.x + x + node_width - 1, area.y + y + 2)]
                    .set_char(bottom_right)
                    .set_style(border_style);
            }
        }
    }

    /// Draw an edge between two nodes.
    fn draw_edge(
        &self,
        buffer: &mut Buffer,
        from_idx: NodeIndex,
        to_idx: NodeIndex,
        area: Rect,
        cycle_nodes: &HashSet<String>,
    ) {
        let from_node = &self.app.graph[from_idx];
        let to_node = &self.app.graph[to_idx];

        // Check if this edge is part of a cycle
        let in_cycle = cycle_nodes.contains(&from_node.relative_path)
            && cycle_nodes.contains(&to_node.relative_path);

        let edge_color = if in_cycle { Color::Red } else { Color::DarkGray };

        // Get edge start and end points
        if let (Some((x1, y1)), Some((x2, y2))) = (
            self.layout.get_node_right(from_idx),
            self.layout.get_node_left(to_idx),
        ) {
            // Simple L-shaped edge: horizontal then vertical
            let horizontal = '\u{2500}'; // ─
            let vertical = '\u{2502}'; // │
            let corner_down = '\u{2510}'; // ┐
            let corner_up = '\u{2514}'; // └
            let arrow = '\u{25B6}'; // ▶

            // Draw horizontal line from source
            for x in x1..x2.saturating_sub(1) {
                if x < area.width && y1 < area.height {
                    let cell = &buffer[(area.x + x, area.y + y1)];
                    if cell.symbol() == " " {
                        buffer[(area.x + x, area.y + y1)]
                            .set_char(horizontal)
                            .set_fg(edge_color);
                    }
                }
            }

            // Draw arrow at the end
            if x2 > 0 && x2 - 1 < area.width && y2 < area.height {
                buffer[(area.x + x2 - 1, area.y + y2)]
                    .set_char(arrow)
                    .set_fg(edge_color);
            }

            // Draw vertical line if needed
            if y1 != y2 {
                let (start_y, end_y) = if y1 < y2 {
                    (y1 + 1, y2)
                } else {
                    (y2 + 1, y1)
                };

                for y in start_y..end_y {
                    if y < area.height && x2 > 0 && x2 - 1 < area.width {
                        let cell = &buffer[(area.x + x2 - 1, area.y + y)];
                        if cell.symbol() == " " {
                            buffer[(area.x + x2 - 1, area.y + y)]
                                .set_char(vertical)
                                .set_fg(edge_color);
                        }
                    }
                }

                // Draw corner
                if x2 > 0 && x2 - 1 < area.width && y1 < area.height {
                    let corner = if y1 < y2 { corner_down } else { corner_up };
                    buffer[(area.x + x2 - 1, area.y + y1)]
                        .set_char(corner)
                        .set_fg(edge_color);
                }
            }
        }
    }
}

impl<'a> Widget for GraphView<'a> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let cycle_nodes = self.get_cycle_nodes();

        // First pass: draw all edges (so they appear behind nodes)
        for edge in self.app.graph.edge_references() {
            self.draw_edge(buffer, edge.source(), edge.target(), area, &cycle_nodes);
        }

        // Second pass: draw all nodes
        for node_idx in self.app.graph.node_indices() {
            if let Some((x, y)) = self.layout.get_position(node_idx) {
                // Apply viewport offset here when implemented
                self.draw_node(buffer, node_idx, x, y, area, &cycle_nodes);
            }
        }
    }
}

/// Truncate a filename to fit within the given width.
fn truncate_filename(name: &str, max_width: usize) -> String {
    if name.len() <= max_width {
        name.to_string()
    } else if max_width <= 3 {
        "...".to_string()
    } else {
        let keep = max_width - 3;
        format!("{}...", &name[..keep])
    }
}

/// Get the color for a layer.
fn layer_color(layer: Layer) -> Color {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_filename_no_truncation() {
        assert_eq!(truncate_filename("main.zig", 20), "main.zig");
    }

    #[test]
    fn test_truncate_filename_exact_fit() {
        assert_eq!(truncate_filename("main.zig", 8), "main.zig");
    }

    #[test]
    fn test_truncate_filename_truncated() {
        assert_eq!(
            truncate_filename("scheduler_interrupt_handler.zig", 20),
            "scheduler_interru..."
        );
    }

    #[test]
    fn test_truncate_filename_very_short() {
        assert_eq!(truncate_filename("main.zig", 3), "...");
    }
}
