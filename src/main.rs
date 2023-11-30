mod account;
mod accounts;
mod message;
mod transaction;

use std::path::Path;

use iced::{window, Sandbox};
use image::io::Reader as ImageReader;

use crate::accounts::Accounts;

/// The size of padding.
const PADDING: u16 = 8;
/// The size of text widgets.
const TEXT_SIZE: u16 = 24;

/// Runs the ledger application.
pub fn main() -> Result<(), iced::Error> {
    let icon = match ImageReader::open(Path::new("./icons/fin-stat_64x64.png")) {
        Ok(icon) => icon,
        Err(_) => ImageReader::open(Path::new(
            "/usr/share/icons/hicolor/64x64/apps/fin-stat.png",
        ))
        .unwrap(),
    };
    let icon = icon.decode().unwrap().to_rgba8();
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
