mod account;
mod accounts;
mod app;
mod file_picker;
mod message;
mod transaction;

use app::App;
use iced::Application;

/// The size of padding.
const PADDING: u16 = 1;
const EDGE_PADDING: usize = 4;
/// The size of text widgets.
const TEXT_SIZE: u16 = 24;

/// Runs the ledger application.
pub fn main() -> Result<(), iced::Error> {
    App::run(iced::Settings {
        window: iced::window::Settings {
            size: (1280, 720),
            ..iced::window::Settings::default()
        },
        ..iced::Settings::default()
    })
}
