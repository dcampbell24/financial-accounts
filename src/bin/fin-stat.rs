use fin_stat::app::App;
use iced::Application;

fn main() -> Result<(), iced::Error> {
    App::run(iced::Settings {
        window: iced::window::Settings {
            size: (1280, 720),
            ..iced::window::Settings::default()
        },
        ..iced::Settings::default()
    })
}
