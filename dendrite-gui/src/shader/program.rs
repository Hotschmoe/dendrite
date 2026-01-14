use crate::Message;
use iced::advanced::mouse;
use iced::widget::shader::{self, Action, Primitive};
use iced::{wgpu, Event, Point, Rectangle};

use super::pipeline::GraphPipeline;

#[derive(Default)]
pub struct GraphShader;

#[derive(Debug)]
pub struct ShaderState {
    pub zoom: f32,
    pub pan: [f32; 2],
    pub dragging: bool,
    pub last_cursor: Option<Point>,
}

impl Default for ShaderState {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan: [0.0, 0.0],
            dragging: false,
            last_cursor: None,
        }
    }
}

#[derive(Debug)]
pub struct GraphPrimitive {
    pub zoom: f32,
    pub pan: [f32; 2],
    pub bounds: Rectangle,
}

impl Primitive for GraphPrimitive {
    type Pipeline = GraphPipeline;

    fn prepare(
        &self,
        pipeline: &mut Self::Pipeline,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        _bounds: &Rectangle,
        _viewport: &shader::Viewport,
    ) {
        pipeline.update_uniforms(queue, self.zoom, self.pan, &self.bounds);
    }

    fn draw(
        &self,
        pipeline: &Self::Pipeline,
        render_pass: &mut wgpu::RenderPass<'_>,
    ) -> bool {
        pipeline.draw(render_pass);
        true
    }
}

impl shader::Program<Message> for GraphShader {
    type State = ShaderState;
    type Primitive = GraphPrimitive;

    fn draw(
        &self,
        state: &Self::State,
        _cursor: mouse::Cursor,
        bounds: Rectangle,
    ) -> Self::Primitive {
        GraphPrimitive {
            zoom: state.zoom,
            pan: state.pan,
            bounds,
        }
    }

    fn update(
        &self,
        state: &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<Message>> {
        match event {
            Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    if let Some(position) = cursor.position_in(bounds) {
                        state.dragging = true;
                        state.last_cursor = Some(position);
                        return Some(Action::request_redraw());
                    }
                }
                mouse::Event::ButtonReleased(mouse::Button::Left) => {
                    state.dragging = false;
                    state.last_cursor = None;
                    return Some(Action::request_redraw());
                }
                mouse::Event::CursorMoved { .. } => {
                    if state.dragging {
                        if let Some(position) = cursor.position_in(bounds) {
                            if let Some(last) = state.last_cursor {
                                let delta = [
                                    (position.x - last.x) / bounds.width,
                                    -(position.y - last.y) / bounds.height,
                                ];
                                state.pan[0] += delta[0] * 2.0;
                                state.pan[1] += delta[1] * 2.0;
                            }
                            state.last_cursor = Some(position);
                            return Some(Action::request_redraw());
                        }
                    }
                }
                mouse::Event::WheelScrolled { delta } => {
                    let zoom_delta = match delta {
                        mouse::ScrollDelta::Lines { y, .. } => y * 0.1,
                        mouse::ScrollDelta::Pixels { y, .. } => y * 0.01,
                    };

                    state.zoom = (state.zoom + zoom_delta).clamp(0.1, 10.0);
                    return Some(Action::request_redraw());
                }
                _ => {}
            },
            _ => {}
        }

        None
    }

    fn mouse_interaction(
        &self,
        state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if cursor.is_over(bounds) {
            if state.dragging {
                mouse::Interaction::Grabbing
            } else {
                mouse::Interaction::Grab
            }
        } else {
            mouse::Interaction::default()
        }
    }
}
