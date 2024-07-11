use financial_accounts::app::App;
use iced::{Application, Size};

fn main() -> Result<(), iced::Error> {
    App::run(iced::Settings {
        window: iced::window::Settings {
            size: Size {
                width: 1280.0,
                height: 720.0,
            },
            ..iced::window::Settings::default()
        },
        ..iced::Settings::default()
    })
}
