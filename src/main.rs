mod account;
mod accounts;
mod ledger;
mod message;

use std::path::Path;

use iced::{Sandbox, window};
use image::io::Reader as ImageReader;

use crate::accounts::Accounts;

/// The size of padding.
const PADDING: u16 = 8;
/// The size of text widgets.
const TEXT_SIZE: u16 = 24;

/// Runs the ledger application.
pub fn main() -> Result<(), iced::Error> {
    let icon = ImageReader::open(Path::new("./icon.ico"))
        .unwrap()
        .decode()
        .unwrap()
        .to_rgba8();
    let icon = match window::icon::from_rgba(icon.to_vec(), icon.width(), icon.height()) {
        Ok(icon) => Some(icon),
        Err(err) => {
            println!("{}", err);
            None
        }
    };

    Accounts::run(iced::Settings {
        window: iced::window::Settings {
            icon,
            size: (1280, 720),
            ..iced::window::Settings::default()
        },
        ..iced::Settings::default()
    })
    // println!("{accounts:#?}")
    // Ok(())
}
