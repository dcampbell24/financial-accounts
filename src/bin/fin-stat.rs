use fin_stat::app::App;
use iced::{Application, Pixels, Size};

fn main() -> Result<(), iced::Error> {
    App::run(iced::Settings {
        default_text_size: Pixels(24.0),
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
