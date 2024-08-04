mod account;
mod accounts;
mod chart;
mod crypto;
mod file_picker;
mod houses;
mod import_boa;
mod message;
mod metals;
mod money;
mod mutual_funds;
mod screen;
pub mod solarized;
mod stocks;
mod zillow_cookies;

use std::{cmp::Ordering, path::PathBuf, sync::Arc};

use iced::{
    event, executor,
    keyboard::{self, Key, Modifiers},
    theme,
    widget::{
        button, column,
        combo_box::{ComboBox, State},
        row, text, text_input, Button, Row, Scrollable,
    },
    window, Alignment, Application, Command, Element, Event, Length, Theme,
};
use money::Currency;
use plotters_iced::ChartWidget;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use thousands::Separable;

use crate::app::{
    account::transaction::TransactionToSubmit, account::Account, accounts::Accounts,
    file_picker::FilePicker, import_boa::import_boa, message::Message, screen::Screen,
};

const EDGE_PADDING: usize = 4;
const PADDING: u16 = 1;
const ROW_SPACING: u16 = 4;
const TEXT_SIZE: u16 = 24;

/// The financial-accounts application.
#[derive(Clone, Debug)]
pub struct App {
    accounts: Accounts,
    file_path: PathBuf,
    file_picker: FilePicker,
    account_name: String,
    currency: Option<Currency>,
    currency_selector: State<Currency>,
    project_months: Option<u16>,
    screen: Screen,
    error: Option<Arc<anyhow::Error>>,
}

impl App {
    fn new(accounts: Accounts, file_path: PathBuf, screen: Screen) -> Self {
        let currencies = accounts.get_currencies();

        Self {
            accounts,
            file_path,
            file_picker: FilePicker::new(),
            account_name: String::new(),
            currency: None,
            currency_selector: State::new(currencies),
            project_months: None,
            screen,
            error: None,
        }
    }

    #[rustfmt::skip]
    fn list_accounts(&self) -> Scrollable<Message> {
        let my_chart = self.accounts.all_accounts_txs_1st();
        let chart = ChartWidget::new(my_chart).height(Length::Fixed(400.0));

        let mut col_0 = column![text_cell(" Account "), text_cell("")];
        let mut col_1 = column![text_cell(" Current Month "), text_cell("")].align_items(Alignment::End);
        let mut col_2 = column![text_cell(" Last Month "), text_cell("")].align_items(Alignment::End);
        let mut col_3 = column![text_cell(" Current Year "), text_cell("")].align_items(Alignment::End);
        let mut col_4 = column![text_cell(" Last Year "), text_cell("")].align_items(Alignment::End);
        let mut col_5 = column![text_cell(" Balance "), text_cell("")].align_items(Alignment::End);
        let mut col_6 = column![text_cell(" Quantity "), text_cell("")].align_items(Alignment::End);
        let mut col_7 = column![text_cell(""), text_cell("")];
        let mut col_8 = column![text_cell(""), text_cell("")];
        let mut col_9 = column![text_cell(""), text_cell("")];
        let mut col_10 = column![text_cell(""), text_cell("")];
        let mut col_11 = column![text_cell(""), text_cell("")];
        let mut col_12 = column![text_cell(""), text_cell("")];
        let mut col_13 = column![text_cell(""), text_cell("")];

        for (i, account) in self.accounts.inner.iter().enumerate() {
            let mut current_month = account.sum_current_month();
            let mut last_month = account.sum_last_month();
            let mut current_year = account.sum_current_year();
            let mut last_year = account.sum_last_year();
            let mut balance_1st = account.balance_1st();

            let balance_2nd = account.balance_2nd().map_or_else(|| text_cell(""), |mut balance| {
                balance.rescale(8);
                number_cell(balance)
            });

            current_month.rescale(2);
            last_month.rescale(2);
            current_year.rescale(2);
            last_year.rescale(2);
            balance_1st.rescale(2);

            col_0 = col_0.push(text_cell(&account.name));
            col_1 = col_1.push(number_cell(current_month));
            col_2 = col_2.push(number_cell(last_month));
            col_3 = col_3.push(number_cell(current_year));
            col_4 = col_4.push(number_cell(last_year));
            col_5 = col_5.push(number_cell(balance_1st));
            col_6 = col_6.push(balance_2nd);
            col_7 = col_7.push(button_cell(button("Tx").on_press(Message::SelectAccount(i))));
            let mut txs_2nd = button("Tx 2nd");
            if let Some(account) = &account.txs_2nd {
                if account.has_txs_2nd() {
                    txs_2nd = txs_2nd.on_press(Message::SelectAccountSecondary(i));
                }
            }
            col_8 = col_8.push(button_cell(txs_2nd));
            col_9 = col_9.push(button_cell(button("Monthly Tx").on_press(Message::SelectMonthly(i))));
            let mut update_name = button("Update Name");
            if !self.account_name.is_empty() {
                update_name = update_name.on_press(Message::UpdateAccount(i));
            }
            col_10 = col_10.push(button_cell(update_name));
            let mut import_boa = button("Import BoA");
            if account.txs_2nd.is_none() {
                import_boa = import_boa.on_press(Message::ImportBoaScreen(i));
            }
            col_11 = col_11.push(button_cell(import_boa));
            let mut get_ohlc = button("Get Price");
            if account.txs_2nd.is_some() {
                get_ohlc = get_ohlc.on_press(Message::GetOhlc(i));
            }
            col_12 = col_12.push(button_cell(get_ohlc));
            col_13 = col_13.push(button_cell(button("Delete").on_press(Message::Delete(i))));
        }
        let rows = row![col_0, col_1, col_2, col_3, col_4, col_5, col_6, col_7, col_8, col_9, col_10, col_11, col_12, col_13];

        let error = self.error.as_ref().map_or_else(|| row![], |error| row![text_cell(error)]);

        let col_1 = column![
            text_cell("total current month USD: "),
            text_cell("total last month USD: "),
            text_cell("total current year USD: "),
            text_cell("total last year USD: "),
            text_cell("balance USD: "),
        ];

        let mut total_for_current_month_usd = self.accounts.total_for_current_month_usd();
        let mut total_for_last_month_usd = self.accounts.total_for_last_month_usd();
        let mut total_for_current_year_usd = self.accounts.total_for_current_year_usd();
        let mut total_for_last_year_usd = self.accounts.total_for_last_year_usd();
        let mut balance = self.accounts.balance_usd();

        total_for_current_month_usd.rescale(2);
        total_for_last_month_usd.rescale(2);
        total_for_current_year_usd.rescale(2);
        total_for_last_year_usd.rescale(2);
        balance.rescale(2);

        let col_2 = column![
            number_cell(total_for_current_month_usd),
            number_cell(total_for_last_month_usd),
            number_cell(total_for_current_year_usd),
            number_cell(total_for_last_year_usd),
            number_cell(balance),
            text_cell(""),
        ].align_items(Alignment::End);
        let totals = row![col_1, col_2];

        let name = text_input("Name", &self.account_name)
            .on_input(Message::ChangeAccountName);

        let months = text_input("Months", &some_or_empty(&self.project_months))
            .on_input(Message::ChangeProjectMonths);

        let mut add = button("Add");
        if !self.account_name.is_empty() && self.currency.is_some() {
            add = add.on_press(Message::SubmitAccount);
        }
        let cols = column![
            chart,
            rows,
            error,
            text_cell(""),
            totals,
            text_cell(""),
            row![
                text("Account").size(TEXT_SIZE),
                name,
                ComboBox::new(&self.currency_selector, "currency", self.currency.as_ref(), |currency|  { Message::UpdateCurrency(currency) }),
                add,
                text(" ".repeat(EDGE_PADDING)),

            ].padding(PADDING).spacing(ROW_SPACING),
            row![
                text("Project").size(TEXT_SIZE),
                months,
                text((self.accounts.project_months(self.project_months)).separate_with_commas()).size(TEXT_SIZE),
                text(" ".repeat(EDGE_PADDING)),
            ].padding(PADDING).spacing(ROW_SPACING),
            button_cell(button("Exit").on_press(Message::Exit)),
            // text_(format!("Checked Up To: {}", self.checked_up_to.to_string())).size(TEXT_SIZE),
        ];

        Scrollable::new(cols)
    }

    const fn selected_account(&self) -> Option<usize> {
        match self.screen {
            Screen::NewOrLoadFile | Screen::Accounts => None,
            Screen::Account(account)
            | Screen::AccountSecondary(account)
            | Screen::Monthly(account)
            | Screen::ImportBoa(account) => Some(account),
        }
    }
}

impl Application for App {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        match FilePicker::load_or_new_file() {
            Some((accounts, path_buf)) => (
                Self::new(accounts, path_buf, Screen::Accounts),
                window::maximize(window::Id::MAIN, true),
            ),
            None => (
                Self::new(Accounts::new(), PathBuf::new(), Screen::NewOrLoadFile),
                window::maximize(window::Id::MAIN, true),
            ),
        }
    }

    fn theme(&self) -> Self::Theme {
        Theme::SolarizedLight
    }

    fn title(&self) -> String {
        String::from("Financial Accounts")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        let selected_account = self.selected_account();
        self.error = None;

        match message {
            Message::Account(message) => {
                if self.accounts[selected_account.unwrap()].update(&self.screen, message) {
                    self.accounts.save(&self.file_path).unwrap();
                }
            }
            Message::Back => self.screen = Screen::Accounts,
            Message::ChangeAccountName(name) => self.account_name = name,
            Message::ChangeProjectMonths(months) => {
                if months.is_empty() {
                    self.project_months = None;
                }
                if let Ok(months) = months.parse() {
                    self.project_months = Some(months);
                }
            }
            Message::Delete(i) => {
                match self.screen {
                    Screen::NewOrLoadFile => {
                        panic!("Screen::NewOrLoadFile can't be reached here");
                    }
                    Screen::Accounts => {
                        self.accounts.inner.remove(i);
                    }
                    Screen::Account(j) => {
                        self.accounts[j].txs_1st.txs.remove(i);
                    }
                    Screen::AccountSecondary(j) => {
                        self.accounts[j].txs_2nd.as_mut().unwrap().txs.remove(i);
                    }
                    Screen::ImportBoa(_j) => {
                        panic!("Screen::ImportBoa can't be reached here");
                    }
                    Screen::Monthly(j) => {
                        self.accounts[j].txs_monthly.remove(i);
                    }
                };
                self.accounts.save(&self.file_path).unwrap();
            }
            Message::FilePicker(message) => {
                if let Some(app) = self.file_picker.update(message) {
                    *self = app;
                }
            }
            Message::GetOhlc(i) => {
                let account = &mut self.accounts[i];

                match account.submit_ohlc() {
                    Ok(tx) => {
                        account.txs_1st.txs.push(tx);
                        account.txs_1st.txs.sort_by_key(|tx| tx.date);
                        self.accounts.save(&self.file_path).unwrap();
                    }
                    Err(error) => {
                        self.error = Some(Arc::new(error));
                    }
                }
            }
            Message::ImportBoa(i, file_path) => {
                let account = &mut self.accounts[i];
                let mut boa = import_boa(file_path).unwrap();

                let mut tx_1st = boa.pop_front().unwrap();
                tx_1st.amount = tx_1st.balance - account.balance_1st();
                boa.push_front(tx_1st);

                for tx in boa {
                    account.txs_1st.txs.push(tx);
                }
                account.txs_1st.txs.sort_by_key(|tx| tx.date);
                account.tx = TransactionToSubmit::new();
                self.accounts.save(&self.file_path).unwrap();
                self.screen = Screen::Accounts;
            }
            Message::ImportBoaScreen(i) => self.screen = Screen::ImportBoa(i),
            Message::UpdateAccount(i) => {
                self.accounts[i].name = self.account_name.trim().to_string();
                self.accounts
                    .inner
                    .sort_by_key(|account| account.name.clone());
                self.accounts.save(&self.file_path).unwrap();
            }
            Message::UpdateCurrency(currency) => {
                self.currency = Some(currency);
            }
            Message::SelectAccount(i) => self.screen = Screen::Account(i),
            Message::SelectAccountSecondary(i) => self.screen = Screen::AccountSecondary(i),
            Message::SelectMonthly(i) => self.screen = Screen::Monthly(i),
            Message::SubmitAccount => {
                self.accounts.inner.push(Account::new(
                    self.account_name.trim().to_string(),
                    self.currency.clone().unwrap(),
                ));
                self.accounts
                    .inner
                    .sort_by_key(|account| account.name.clone());
                self.accounts.save(&self.file_path).unwrap();
            }
            Message::Exit => {
                return window::close(window::Id::MAIN);
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        match self.screen {
            Screen::NewOrLoadFile => self.file_picker.view(None).into(),
            Screen::Accounts => self.list_accounts().into(),
            Screen::Account(i) => {
                let account = &self.accounts[i];
                self.accounts[i]
                    .list_transactions(
                        account.txs_1st.clone(),
                        account.total_1st(),
                        account.balance_1st(),
                    )
                    .into()
            }
            Screen::AccountSecondary(i) => {
                let account = &self.accounts[i];
                let txs = account.txs_2nd.as_ref().unwrap();
                self.accounts[i]
                    .list_transactions(
                        txs.clone(),
                        account.total_2nd(),
                        account.balance_2nd().unwrap(),
                    )
                    .into()
            }
            Screen::Monthly(i) => self.accounts[i].list_monthly().into(),
            Screen::ImportBoa(i) => self.file_picker.view(Some(i)).into(),
        }
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        event::listen_with(|event, _status| {
            let mut subscription = None;
            if let Event::Keyboard(keyboard::Event::KeyPressed {
                key,
                location: _location,
                modifiers,
                text: _text,
            }) = event
            {
                if key == Key::Character("h".into()) && modifiers == Modifiers::CTRL {
                    subscription =
                        Some(Message::FilePicker(file_picker::Message::HiddenFilesToggle));
                }
            }
            subscription
        })
    }
}

fn some_or_empty<T: ToString>(value: &Option<T>) -> String {
    value
        .as_ref()
        .map_or_else(String::new, std::string::ToString::to_string)
}

fn set_amount(amount: &mut Option<Decimal>, string: &str) {
    if string.is_empty() {
        *amount = None;
    } else if let Ok(amount_) = string.parse() {
        *amount = Some(amount_);
    }
}

fn button_cell(button: Button<Message>) -> Row<Message> {
    row![button].padding(PADDING)
}

fn number_cell<'a>(num: Decimal) -> Row<'a, Message> {
    let text = match num.cmp(&dec!(0)) {
        Ordering::Greater => {
            text(num.separate_with_commas()).style(theme::Text::Color(solarized::green()))
        }
        Ordering::Less => {
            text(num.separate_with_commas()).style(theme::Text::Color(solarized::red()))
        }
        Ordering::Equal => text(num.separate_with_commas()),
    };

    row![text.size(TEXT_SIZE)].padding(PADDING)
}

fn text_cell<'a>(s: impl ToString) -> Row<'a, Message> {
    row![text(s).size(TEXT_SIZE)].padding(PADDING)
}
