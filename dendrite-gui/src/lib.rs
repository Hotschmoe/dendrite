use iced::{window, Element, Task, Theme};

#[derive(Default)]
pub struct App {
    // Will hold graph, analysis, UI state
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
        iced::widget::text("Dendrite - Dependency Graph Viewer").into()
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
