use financial_accounts::app::App;

fn main() -> iced::Result {
    iced::application("Financial Accounts", App::update, App::view)
        .window_size(iced::Size::INFINITY)
        .theme(App::theme)
        .run()
}
