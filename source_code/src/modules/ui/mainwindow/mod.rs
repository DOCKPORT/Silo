//! MainWindow: the Iced Application skeleton for Silo.
//!
//! This holds the Iced application entry point (the struct that implements the
//! Application trait). It is kept as a skeleton for the foundation scaffold;
//! the model, view, and update wiring are implemented in a later step.

use iced::Application;

/// The Silo application model. Empty placeholder for the foundation scaffold.
#[derive(Debug, Default)]
pub struct SiloApp {
    /// Reserved for future silo settings loaded from the config store.
    _placeholder: (),
}

/// Messages handled by the Silo application.
#[derive(Debug, Clone, Copy)]
pub enum Message {
    /// No-op placeholder message.
    Noop,
}

impl Application for SiloApp {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(_flags: ()) -> (SiloApp, iced::Command<Message>) {
        (SiloApp::default(), iced::Command::none())
    }

    fn title(&self) -> String {
        "Silo".to_string()
    }

    fn update(&mut self, _message: Message) -> iced::Command<Message> {
        iced::Command::none()
    }

    fn view(&self) -> iced::Element<'_, Message, iced::Theme, iced::Renderer> {
        iced::widget::text("Silo").into()
    }
}
