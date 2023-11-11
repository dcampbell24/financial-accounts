mod accounts;
mod ledger;

use iced::Sandbox;

use crate::accounts::Accounts;

const PADDING: u16 = 8;
const TEXT_SIZE: u16 = 25;

fn main() -> Result<(), iced::Error> {
    Accounts::run(iced::Settings {
        window: iced::window::Settings {
            size: (1280, 720),
            ..iced::window::Settings::default()
        },
        ..iced::Settings::default()
    })
    // println!("{accounts:#?}")
    // Ok(())
}
