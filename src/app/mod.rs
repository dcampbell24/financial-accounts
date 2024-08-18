mod account;
mod accounts;
mod chart;
mod crypto;
mod import_boa;
mod message;
mod metal;
mod money;
mod screen;
pub mod solarized;
mod stocks;

use std::{borrow::BorrowMut, cmp::Ordering, fs, path::PathBuf, str::FromStr, sync::Arc};

use account::{transaction::Transaction, transactions::Transactions};
use anyhow::Context;
use chart::Chart;
use chrono::Utc;
use clap::{arg, command, Parser};
use iced::{
    executor, theme,
    widget::{
        button, column,
        combo_box::{ComboBox, State},
        row, text, text_input, Button, Column, Row, Scrollable,
    },
    window, Alignment, Application, Element, Length, Theme,
};
use money::{Currency, Fiat};
use plotters_iced::ChartWidget;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::Deserialize;
use stocks::StockPlus;
use thousands::Separable;

use crate::app::{account::Account, accounts::Accounts, message::Message, screen::Screen};

const TITLE_FILE_PICKER: &str = "Financial Accounts";
const EDGE_PADDING: usize = 4;
const PADDING: u16 = 1;
const COLUMN_SPACING: f32 = 0.3;
const ROW_SPACING: u16 = 4;
const TEXT_SIZE: u16 = 24;

/// The financial-accounts application.
#[derive(Clone, Debug)]
pub struct App {
    accounts: Accounts,
    file_path: Option<PathBuf>,
    account_name: String,
    currency: Option<Currency>,
    currency_selector: State<Currency>,
    duration: Duration,
    project_months: Option<u16>,
    screen: Screen,
    errors: Option<Arc<Vec<anyhow::Error>>>,
}

enum File {
    Load(PathBuf),
    New(PathBuf),
    None,
}

impl App {
    fn get_configuration_file() -> File {
        let args = Args::parse();

        if let Some(arg) = args.load {
            File::Load(PathBuf::from(arg))
        } else if let Some(arg) = args.new {
            File::New(PathBuf::from(arg))
        } else {
            File::None
        }
    }

    fn load_file(&mut self) {
        let result = rfd::FileDialog::new()
            .set_title(TITLE_FILE_PICKER)
            .add_filter("ron", &["ron"])
            .pick_file()
            .context("You must choose a file name for your configuration file.");

        match result {
            Ok(file_path) => match Accounts::load(&file_path) {
                Ok(accounts) => {
                    self.accounts = accounts;
                    self.file_path = Some(file_path);
                }
                Err(error) => self.errors = Some(Arc::new(vec![error])),
            },
            Err(error) => self.errors = Some(Arc::new(vec![error])),
        }
    }

    fn save_file(&mut self) {
        let result = rfd::FileDialog::new()
            .set_title(TITLE_FILE_PICKER)
            .add_filter("ron", &["ron"])
            .save_file()
            .context("You must choose a file name for your configuration file.");

        match result {
            Ok(file_path) => match self.accounts.save(Some(&file_path)) {
                Ok(()) => self.file_path = Some(file_path),
                Err(error) => self.errors = Some(Arc::new(vec![error])),
            },
            Err(error) => self.errors = Some(Arc::new(vec![error])),
        }
    }

    fn save(&mut self) {
        self.accounts
            .save(self.file_path.as_ref())
            .unwrap_or_else(|error| match self.errors.borrow_mut() {
                Some(ref mut errors) => {
                    let errors = Arc::get_mut(errors).unwrap();
                    errors.push(error);
                }
                None => self.errors = Some(Arc::new(vec![error])),
            });
    }

    fn new(accounts: Accounts, file_path: Option<PathBuf>) -> Self {
        let currencies = accounts.get_currencies();

        Self {
            accounts,
            file_path,
            account_name: String::new(),
            currency: None,
            currency_selector: State::new(currencies),
            duration: Duration::All,
            project_months: None,
            screen: Screen::Accounts,
            errors: None,
        }
    }

    fn change_project_months(&mut self, months: &str) {
        if months.is_empty() {
            self.project_months = None;
        } else if let Ok(months) = months.parse() {
            self.project_months = Some(months);
        }
    }

    fn check_monthly(&mut self) {
        self.accounts.check_monthly();
        match self.accounts.save(self.file_path.as_ref()) {
            Ok(()) => {}
            Err(error) => self.errors = Some(Arc::new(vec![error])),
        }
    }

    fn import_investor_360(&mut self, file_xls: &PathBuf) -> anyhow::Result<()> {
        let file_csv = file_xls.file_stem().unwrap();
        let mut file_csv = PathBuf::from_str(file_csv.to_str().unwrap())?;
        file_csv.set_extension("csv");

        if fs::exists(&file_csv)? {
            return Err(anyhow::Error::msg(format!(
                "\"{:?}\" already exists!",
                &file_csv
            )));
        }

        std::process::Command::new("libreoffice")
            .arg("--convert-to")
            .arg("csv")
            .arg(file_xls)
            .status()
            .context(
                r#"Couldn't execute "libreoffice --convert-to csv", you must install libreoffice."#,
            )?;

        for investor_360_record in csv::Reader::from_path(&file_csv)?.deserialize() {
            let investor_360_record: Investor360 = investor_360_record?;

            // Skip some junk records.
            if investor_360_record.symbol.is_empty() {
                continue;
            }

            let balance = investor_360_record.quantity.unwrap();
            let mut tx = Transaction {
                amount: dec!(0),
                balance,
                comment: investor_360_record.description.clone(),
                date: Utc::now(),
            };

            let name = format!("Investor 360: {}", &investor_360_record.symbol);
            let mut name_matches = false;
            for account in &mut self.accounts.inner {
                if account.name == name {
                    if investor_360_record.price.unwrap() == dec!(1) {
                        tx.amount = balance - account.txs_1st.txs.last().unwrap().balance;
                        account.txs_1st.txs.push(tx.clone());
                    } else {
                        tx.amount = balance
                            - account
                                .txs_2nd
                                .as_ref()
                                .unwrap()
                                .txs
                                .last()
                                .unwrap()
                                .balance;
                        account.txs_2nd.as_mut().unwrap().txs.push(tx.clone());
                    }
                    name_matches = true;
                    break;
                }
            }

            if !name_matches {
                tx.amount = balance;
                let txs = vec![tx];

                if investor_360_record.price.unwrap() == dec!(1) {
                    let currency = Fiat::Usd;
                    let transactions = Transactions {
                        currency: currency.clone(),
                        txs,
                    };
                    let mut account = Account::new(name, Currency::Fiat(currency));
                    account.txs_1st = transactions;
                    self.accounts.inner.push(account);
                } else {
                    let stock = StockPlus {
                        description: investor_360_record.description,
                        symbol: investor_360_record.symbol,
                    };
                    let currency = Currency::StockPlus(stock);
                    let transactions = Transactions {
                        currency: currency.clone(),
                        txs,
                    };
                    let mut account = Account::new(name, currency);
                    account.txs_2nd = Some(transactions);
                    self.accounts.inner.push(account);
                }
            }
        }

        fs::remove_file(&file_csv)?;
        Ok(())
    }

    #[rustfmt::skip]
    fn rows(&self) -> Row<Message> {
        let mut col_0 = column![text_cell(" Account "), text_cell("")];
        let mut col_1 = column![button_cell(button("Week").on_press(Message::ChartWeek)), text_cell("")].align_items(Alignment::End);
        let mut col_2 = column![button_cell(button("Month").on_press(Message::ChartMonth)), text_cell("")].align_items(Alignment::End);
        let mut col_3 = column![button_cell(button("Year").on_press(Message::ChartYear)), text_cell("")].align_items(Alignment::End);
        let mut col_4 = column![button_cell(button("Balance").on_press(Message::ChartAll)), text_cell("")].align_items(Alignment::End);
        let mut col_5 = column![text_cell("Quantity"), text_cell("")].align_items(Alignment::End);
        let mut col_6 = column![text_cell(""), text_cell("")].spacing(COLUMN_SPACING);
        let mut col_7 = column![text_cell(""), text_cell("")].spacing(COLUMN_SPACING);
        let mut col_8 = column![text_cell(""), text_cell("")].spacing(COLUMN_SPACING);
        let mut col_9 = column![text_cell(""), text_cell("")].spacing(COLUMN_SPACING);
        let mut col_a = column![text_cell(""), text_cell("")].spacing(COLUMN_SPACING);
        let mut col_b = column![text_cell(""), text_cell("")].spacing(COLUMN_SPACING);
        let mut col_c = column![text_cell(""), text_cell("")].spacing(COLUMN_SPACING);

        for (i, account) in self.accounts.inner.iter().enumerate() {
            let mut last_week = account.sum_last_week();
            let mut last_month = account.sum_last_month();
            let mut last_year = account.sum_last_year();
            let mut balance_1st = account.balance_1st();

            let balance_2nd = account.balance_2nd().map_or_else(|| text_cell(""), |mut balance| {
                balance.rescale(8);
                number_cell(balance)
            });

            last_week.rescale(2);
            last_month.rescale(2);
            last_year.rescale(2);
            balance_1st.rescale(2);

            col_0 = col_0.push(text_cell(&account.name));
            col_1 = col_1.push(number_cell(last_week));
            col_2 = col_2.push(number_cell(last_month));
            col_3 = col_3.push(number_cell(last_year));
            col_4 = col_4.push(number_cell(balance_1st));
            col_5 = col_5.push(balance_2nd);
            col_6 = col_6.push(button_cell(button("Tx").on_press(Message::SelectAccount(i))));
            let mut txs_2nd = button("Tx 2nd");
            if let Some(account) = &account.txs_2nd {
                if account.has_txs_2nd() {
                    txs_2nd = txs_2nd.on_press(Message::SelectAccountSecondary(i));
                }
            }
            col_7 = col_7.push(button_cell(txs_2nd));
            col_8 = col_8.push(button_cell(button("Monthly Tx").on_press(Message::SelectMonthly(i))));
            let mut update_name = button("Update Name");
            if !self.account_name.is_empty() {
                update_name = update_name.on_press(Message::UpdateAccount(i));
            }
            col_9 = col_9.push(button_cell(update_name));
            let mut import_boa = button("Import BoA");
            if account.txs_2nd.is_none() {
                import_boa = import_boa.on_press(Message::ImportBoa(i));
            }
            col_a = col_a.push(button_cell(import_boa));
            let mut get_price = button("Get Price");
            if account.txs_2nd.is_some() {
                get_price = get_price.on_press(Message::GetPrice(i));
            }
            col_b = col_b.push(button_cell(get_price));
            col_c = col_c.push(button_cell(button("Delete").on_press(Message::Delete(i))));
        }
        row![col_0, col_1, col_2, col_3, col_4, col_5, col_6, col_7, col_8, col_9, col_a, col_b, col_c]
    }

    #[rustfmt::skip]
    fn list_accounts(&self) -> Scrollable<Message> {
        let chart = Chart {
            txs: self.accounts.all_accounts_txs_1st(),
            duration: self.duration.clone(),
        };
        let chart = ChartWidget::new(chart).height(Length::Fixed(400.0));
        let rows = self.rows();

        let mut column_errors = Column::new();
        if let Some(errors) = &self.errors {
            for error in errors.iter() {
                column_errors = column_errors.push(text_cell_red(error));
            }
        }

        let col_1 = column![
            text_cell("total last week USD: "),
            text_cell("total last month USD: "),
            text_cell("total last year USD: "),
            text_cell("balance USD: "),
        ];

        let mut total_for_last_week_usd = self.accounts.total_for_last_week_usd();
        let mut total_for_last_month_usd = self.accounts.total_for_last_month_usd();
        let mut total_for_last_year_usd = self.accounts.total_for_last_year_usd();
        let mut balance = self.accounts.balance_usd();

        total_for_last_week_usd.rescale(2);
        total_for_last_month_usd.rescale(2);
        total_for_last_year_usd.rescale(2);
        balance.rescale(2);

        let col_2 = column![
            number_cell(total_for_last_week_usd),
            number_cell(total_for_last_month_usd),
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
            column_errors,
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
            row![
                button_cell(button("Exit").on_press(Message::Exit)),
                button_cell(button("Load").on_press(Message::FileLoad)),
                button_cell(button("Save As").on_press(Message::FileSaveAs)),
                button_cell(button("Import Investor 360").on_press(Message::ImportInvestor360)),
                button_cell(button("Get All Prices").on_press(Message::GetPriceAll)),
                button_cell(button("Check Monthly").on_press(Message::CheckMonthly)),
            ].padding(PADDING).spacing(ROW_SPACING),
            // text_(format!("Checked Up To: {}", self.checked_up_to.to_string())).size(TEXT_SIZE),
        ];

        Scrollable::new(cols)
    }

    fn select_account(&mut self, message: account::Message) {
        if let Some(account) = match self.screen {
            Screen::Accounts => None,
            Screen::Account(account)
            | Screen::AccountSecondary(account)
            | Screen::Monthly(account) => Some(account),
        } {
            if self.accounts[account].update(&self.screen, message) {
                self.accounts.save(self.file_path.as_ref()).unwrap();
            }
        }
    }

    fn submit_account(&mut self) {
        self.accounts.inner.push(Account::new(
            self.account_name.trim().to_string(),
            self.currency.clone().unwrap(),
        ));
        self.accounts.sort();
        self.save();
    }
}

impl Application for App {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Message>) {
        match App::get_configuration_file() {
            File::Load(file_path) => {
                let accounts = Accounts::load(&file_path)
                    .unwrap_or_else(|err| panic!("error loading {:?}: {}", &file_path, err));
                (
                    Self::new(accounts, Some(file_path)),
                    window::maximize(window::Id::MAIN, true),
                )
            }
            File::New(file_path) => {
                let accounts = Accounts::new();
                accounts
                    .save_first(&file_path)
                    .unwrap_or_else(|err| panic!("error creating {:?}: {}", &file_path, err));
                (
                    Self::new(accounts, Some(file_path)),
                    window::maximize(window::Id::MAIN, true),
                )
            }
            File::None => (
                Self::new(Accounts::new(), None),
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

    fn update(&mut self, message: Message) -> iced::Command<Message> {
        self.errors = None;

        match message {
            Message::Account(message) => self.select_account(message),
            Message::Back => self.screen = Screen::Accounts,
            Message::ChangeAccountName(name) => self.account_name = name,
            Message::ChangeProjectMonths(months) => self.change_project_months(&months),
            Message::ChartWeek => self.duration = Duration::Week,
            Message::ChartMonth => self.duration = Duration::Month,
            Message::ChartYear => self.duration = Duration::Year,
            Message::ChartAll => self.duration = Duration::All,
            Message::CheckMonthly => self.check_monthly(),
            Message::Delete(i) => {
                match self.screen {
                    Screen::Accounts => {
                        self.accounts.inner.remove(i);
                    }
                    Screen::Account(j) => {
                        self.accounts[j].txs_1st.txs.remove(i);
                    }
                    Screen::AccountSecondary(j) => {
                        self.accounts[j].txs_2nd.as_mut().unwrap().txs.remove(i);
                    }
                    Screen::Monthly(j) => {
                        self.accounts[j].txs_monthly.remove(i);
                    }
                };
                self.accounts.save(self.file_path.as_ref()).unwrap();
            }
            Message::FileLoad => self.load_file(),
            Message::FileSaveAs => self.save_file(),
            Message::GetPrice(i) => {
                let account = &mut self.accounts[i];

                match futures::executor::block_on(account.submit_price_as_transaction()) {
                    Ok(tx) => {
                        account.txs_1st.txs.push(tx);
                        account.txs_1st.sort();
                        self.accounts.save(self.file_path.as_ref()).unwrap();
                    }
                    Err(error) => {
                        self.errors = Some(Arc::new(vec![error]));
                    }
                }
            }
            Message::GetPriceAll => {
                let errors = futures::executor::block_on(self.accounts.get_all_prices());
                if !errors.is_empty() {
                    self.errors = Some(Arc::new(errors));
                }
                self.accounts.save(self.file_path.as_ref()).unwrap();
            }
            Message::ImportBoa(i) => {
                let account = &mut self.accounts[i];

                if let Some(file_path) = rfd::FileDialog::new()
                    .set_title(TITLE_FILE_PICKER)
                    .add_filter("csv", &["csv"])
                    .pick_file()
                {
                    if let Err(err) = account.import_boa(file_path) {
                        self.errors = Some(Arc::new(vec![err]));
                    } else {
                        self.accounts.save(self.file_path.as_ref()).unwrap();
                    }
                    self.screen = Screen::Accounts;
                }
            }
            Message::ImportInvestor360 => {
                if let Some(file_path) = rfd::FileDialog::new()
                    .set_title(TITLE_FILE_PICKER)
                    .add_filter("xls", &["xls"])
                    .pick_file()
                {
                    if let Err(err) = self.import_investor_360(&file_path) {
                        self.errors = Some(Arc::new(vec![err]));
                    } else {
                        self.accounts.sort();
                        self.accounts.save(self.file_path.as_ref()).unwrap();
                    }
                }
            }
            Message::UpdateAccount(i) => {
                self.accounts[i].name = self.account_name.trim().to_string();
                self.accounts.sort();
                self.accounts.save(self.file_path.as_ref()).unwrap();
            }
            Message::UpdateCurrency(currency) => {
                self.currency = Some(currency);
            }
            Message::SelectAccount(i) => self.screen = Screen::Account(i),
            Message::SelectAccountSecondary(i) => self.screen = Screen::AccountSecondary(i),
            Message::SelectMonthly(i) => self.screen = Screen::Monthly(i),
            Message::SubmitAccount => self.submit_account(),
            Message::Exit => {
                return window::close(window::Id::MAIN);
            }
        }
        iced::Command::none()
    }

    fn view(&self) -> Element<Message> {
        match self.screen {
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
        }
    }
}

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Load FILE
    #[arg(long, value_name = "FILE", exclusive = true)]
    load: Option<String>,

    /// Create a new FILE
    #[arg(long, value_name = "FILE", exclusive = true)]
    new: Option<String>,
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

fn text_cell_red<'a>(s: impl ToString) -> Row<'a, Message> {
    row![text(s)
        .style(theme::Text::Color(solarized::red()))
        .size(TEXT_SIZE)]
    .padding(PADDING)
}

#[derive(Clone, Debug)]
enum Duration {
    Week,
    Month,
    Year,
    All,
}

#[derive(Debug, Deserialize)]
struct Investor360 {
    #[serde(rename = "Description")]
    description: String,
    #[serde(rename = "Symbol")]
    symbol: String,
    #[serde(rename = "Quantity")]
    quantity: Option<Decimal>,
    #[serde(rename = "Price ($)")]
    price: Option<Decimal>,
    #[serde(rename = "Value ($)")]
    _value: Option<Decimal>,
    #[serde(rename = "Assets (%)")]
    _assets: Option<Decimal>,
}
