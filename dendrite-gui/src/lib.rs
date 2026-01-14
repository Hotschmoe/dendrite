use iced::{window, Element, Length, Task, Theme};

mod shader;

use shader::GraphShader;

#[derive(Default)]
pub struct App {
    graph_shader: GraphShader,
}

#[derive(Debug, Clone)]
pub enum Message {
    // Placeholder variant to avoid empty enum
    _Placeholder,
}

fn init_app() -> (App, Task<Message>) {
    (App::default(), Task::none())
}

fn update_app(state: &mut App, message: Message) -> Task<Message> {
    state.update(message)
}

fn view_app(state: &App) -> Element<'_, Message> {
    state.view()
}

fn theme_app(state: &App) -> Theme {
    state.theme()
}

impl App {
    fn update(&mut self, _message: Message) -> Task<Message> {
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        use iced::widget::{column, container, shader, text};

        let title = text("Dendrite - Dependency Graph Viewer").size(20);

        let graph_view = shader(&self.graph_shader)
            .width(Length::Fill)
            .height(Length::Fill);

        let content = column![title, graph_view].spacing(10);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(10)
            .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

pub fn run() -> iced::Result {
    iced::application(init_app, update_app, view_app)
        .theme(theme_app)
        .window(window::Settings {
            size: iced::Size::new(1200.0, 800.0),
            ..Default::default()
        })
        .run()
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn wasm_main() {
    console_error_panic_hook::set_once();
}
