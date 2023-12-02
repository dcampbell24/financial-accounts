mod account;
mod accounts;
mod error;
mod file_picker;
mod message;
mod transaction;

use iced::Sandbox;

use crate::accounts::Accounts;

/// The size of padding.
const PADDING: u16 = 1;
/// The size of text widgets.
const TEXT_SIZE: u16 = 24;

/// Runs the ledger application.
pub fn main() -> Result<(), iced::Error> {
    Accounts::run(iced::Settings {
        window: iced::window::Settings {
            size: (1280, 720),
            ..iced::window::Settings::default()
        },
        ..iced::Settings::default()
    })
}
