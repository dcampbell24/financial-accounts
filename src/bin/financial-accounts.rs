use financial_accounts::app::App;

fn main() -> iced::Result {
    iced::application("Financial Accounts", App::update, App::view)
        .theme(App::theme)
        .run()
}
