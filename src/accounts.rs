use chrono::{offset::Utc, DateTime, Datelike, LocalResult, Months, NaiveDate, TimeZone};
use clap::Parser;
use iced::widget::{button, column, row, text, text_input, Column};
use iced::{Element, Sandbox};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use thousands::Separable;

use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::{mem, u64};

use crate::TEXT_SIZE;
use crate::ledger::{Ledger, Transaction, TransactionToSubmit};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the file to load
    #[arg(long, default_value_t = String::new())]
    load: String,

    /// Name of the new file
    #[arg(long, default_value_t = String::new())]
    new: String,
}

pub struct DateRange(DateTime<Utc>, DateTime<Utc>);

impl Iterator for DateRange {
    type Item = DateTime<Utc>;

    fn next(&mut self) -> Option<Self::Item> {
        let old = self.0;
        self.0 = self.0.checked_add_months(Months::new(1)).unwrap();
        if old < self.1 {
            Some(old)
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Accounts {
    name: String,
    screen: Screen,
    project_months: u64,
    project_months_str: String,
    error_str: String,
    filepath: String,

    accounts: Vec<Account>,
    checked_up_to: DateTime<Utc>,
}

impl Accounts {
    pub fn check_monthly(&mut self) {
        let past = self.checked_up_to;
        let now = Utc::now();
        let day_1 = TimeZone::with_ymd_and_hms(&Utc, now.year(), now.month(), 1, 0, 0, 0).unwrap();

        if day_1 >= past && day_1 < now {
            for account in self.accounts.iter_mut() {
                for tx in account.ledger.monthly.iter() {
                    account.ledger.data.push(Transaction {
                        amount: tx.amount,
                        comment: tx.comment.clone(),
                        date: day_1,
                    });
                }
                account.ledger.data.sort_by_key(|tx| tx.date);
            }
        }
        self.checked_up_to = now;
    }

    pub fn empty_accounts(filepath: &str) -> Self {
        Self {
            name: String::new(),
            screen: Screen::Accounts,
            project_months: 0,
            project_months_str: String::new(),
            error_str: String::new(),
            filepath: filepath.to_string(),

            accounts: Vec::new(),
            checked_up_to: DateTime::<Utc>::default(),
        }
    }

    pub fn total(&self) -> Decimal {
        let mut total = dec!(0.00);
        for account in self.accounts.iter() {
            let sum = account.ledger.sum();
            total += sum;
        }
        total
    }

    pub fn total_for_months(&self) -> Decimal {
        let mut total = dec!(0.00);
        for account in self.accounts.iter() {
            let sum = account.ledger.sum_monthly();
            let times: Decimal = self.project_months.into();
            total += sum * times
        }
        total
    }

    pub fn list_accounts(&self) -> Column<Message> {
        let mut col_1 = column![text("Account").size(TEXT_SIZE)].padding(5);
        let mut col_2 = column![text("Balance").size(TEXT_SIZE)].padding(5);
        let mut col_3 = column![text("").size(TEXT_SIZE)].padding(5);
        let mut col_4 = column![text("").size(TEXT_SIZE)].padding(5);
        let mut col_5 = column![text("").size(TEXT_SIZE)].padding(5);

        let mut total = dec!(0.00);
        for (i, account) in self.accounts.iter().enumerate() {
            let sum = account.ledger.sum();
            total += sum;
            col_1 = col_1.push(text(&account.name).size(TEXT_SIZE));
            col_2 = col_2.push(text(sum.separate_with_commas()).size(TEXT_SIZE));
            col_3 = col_3.push(button(" Select ").on_press(Message::SelectAccount(i)));
            col_4 = col_4.push(button(" Monthly ").on_press(Message::SelectMonthly(i)));
            col_5 = col_5.push(button(" Delete ").on_press(Message::Delete(i)));
        }

        let rows = row![col_1, col_2, col_3, col_4, col_5];
        let cols = column![
            rows,
            text(format!("\ntotal: {:}", total.separate_with_commas())).size(25),
            row![
                text("Add Account ").size(TEXT_SIZE),
                text_input("Name", &self.name)
                    .on_submit(Message::NewAccount)
                    .on_input(|name| Message::ChangeAccountName(name))
            ],
            row![
                text_input("Project Months", &self.project_months_str)
                    .on_input(|i| Message::ChangeProjectMonths(i))
                    .on_submit(Message::ProjectMonths),
                text((self.total() + self.total_for_months()).separate_with_commas()).size(TEXT_SIZE),
            ],
            // text(format!("Checked Up To: {}", self.checked_up_to.to_string())).size(TEXT_SIZE),
        ];
        cols
    }

    pub fn save_first(&self) {
        let j = serde_json::to_string_pretty(&self).unwrap();
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&self.filepath)
            .unwrap();
        file.write_all(j.as_bytes()).unwrap()
    }

    pub fn save(&self) {
        let j = serde_json::to_string_pretty(&self).unwrap();
        let mut file = File::create(&self.filepath).unwrap();
        file.write_all(j.as_bytes()).unwrap()
    }

    pub fn load(filepath: &str) -> Self {
        let mut buf = String::new();
        let mut file = File::open(filepath).unwrap();
        file.read_to_string(&mut buf).unwrap();
        serde_json::from_str(&buf).unwrap()
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    Back,
    ChangeAccountName(String),
    ChangeTx(String),
    ChangeDate(String),
    ChangeComment(String),
    ChangeProjectMonths(String),
    ChangeFilterDateYear(String),
    ChangeFilterDateMonth(String),
    Delete(usize),
    NewAccount,
    ProjectMonths,
    SelectAccount(usize),
    SelectMonthly(usize),
    SubmitTx,
    SubmitFilterDate,
}

impl Sandbox for Accounts {
    type Message = Message;

    fn new() -> Self {
        let args = Args::parse();

        let mut accounts: Accounts;
        if args.load != "" {
            accounts = Accounts::load(&args.load);
        } else if args.new != "" {
            accounts = Accounts::empty_accounts(&args.new);
            accounts.save_first();
        } else {
            panic!("You must choose '--new' or '--load'")
        }

        accounts.check_monthly();
        accounts
    }

    fn title(&self) -> String {
        String::from("Ledger")
    }

    fn update(&mut self, message: Message) {
        let mut list_monthly = false;
        let selected_account = match self.screen {
            Screen::Accounts => 0,
            Screen::Account(account) => account,
            Screen::Monthly(account) => {
                list_monthly = true;
                account
            }
        };

        match message {
            Message::Back => {
                self.screen = Screen::Accounts;
            }
            Message::ChangeAccountName(name) => {
                self.name = name;
            }
            Message::ChangeTx(tx) => {
                self.accounts[selected_account].ledger.tx.amount = tx;
            }
            Message::ChangeDate(date) => {
                self.accounts[selected_account].ledger.tx.date = date;
            }
            Message::ChangeComment(comment) => {
                self.accounts[selected_account].ledger.tx.comment = comment;
            }
            Message::ChangeProjectMonths(i) => {
                self.project_months_str = i;
            }
            Message::ChangeFilterDateYear(date) => {
                self.accounts[selected_account].ledger.filter_date_year = date;
            }
            Message::ChangeFilterDateMonth(date) => {
                self.accounts[selected_account].ledger.filter_date_month = date;
            }
            Message::Delete(i) => match self.screen {
                Screen::Accounts => {
                    self.accounts.remove(i);
                }
                Screen::Account(_) => {
                    self.accounts[selected_account].ledger.data.remove(i);
                }
                Screen::Monthly(_) => {
                    self.accounts[selected_account].ledger.monthly.remove(i);
                }
            },
            Message::NewAccount => self.accounts.push(Account::new(mem::take(&mut self.name))),
            Message::ProjectMonths => match self.project_months_str.parse() {
                Ok(i) => {
                    self.project_months = i;
                    self.error_str = String::new();
                }
                Err(err) => {
                    let mut msg = "Parse Project Months error: ".to_string();
                    msg.push_str(&err.to_string());
                    self.error_str = msg;
                    return;
                }
            },
            Message::SelectAccount(i) => {
                self.screen = Screen::Account(i);
            }
            Message::SelectMonthly(i) => {
                self.screen = Screen::Monthly(i);
            }
            Message::SubmitTx => {
                let account = &mut self.accounts[selected_account];
                let amount_str = account.ledger.tx.amount.clone();
                let mut _amount = dec!(0.00);
                match Decimal::from_str_exact(&amount_str) {
                    Ok(tx) => {
                        _amount = tx;
                    }
                    Err(err) => {
                        let mut msg = "Parse Amount error: ".to_string();
                        msg.push_str(&err.to_string());
                        account.error_str = msg;
                        return;
                    }
                }
                let mut date = Utc::now();
                if account.ledger.tx.date != "" {
                    match NaiveDate::parse_from_str(&account.ledger.tx.date, "%Y-%m-%d") {
                        Ok(naive_date) => {
                            date = naive_date.and_hms_opt(0, 0, 0).unwrap().and_utc();
                        }
                        Err(err) => {
                            let mut msg = "Parse Date error: ".to_string();
                            msg.push_str(&err.to_string());
                            account.error_str = msg;
                            return;
                        }
                    }
                }
                if list_monthly {
                    account.ledger.monthly.push(Transaction {
                        amount: _amount,
                        comment: account.ledger.tx.comment.clone(),
                        date,
                    });
                } else {
                    account.ledger.data.push(Transaction {
                        amount: _amount,
                        comment: account.ledger.tx.comment.clone(),
                        date,
                    });
                    account.ledger.data.sort_by_key(|tx| tx.date);
                }
                account.ledger.tx = TransactionToSubmit::new();
                account.error_str = String::new();
            }
            Message::SubmitFilterDate => {
                let account = &mut self.accounts[selected_account];
                let mut _year = 0;
                let mut _month = 0;

                if account.ledger.filter_date_year == "" && account.ledger.filter_date_month == "" {
                    account.ledger.filter_date = DateTime::<Utc>::default();
                    account.error_str = String::new();
                    return;
                }

                match account.ledger.filter_date_year.parse::<i32>() {
                    Ok(year_input) => _year = year_input,
                    Err(err) => {
                        let mut msg = "Parse Year error: ".to_string();
                        msg.push_str(&err.to_string());
                        account.error_str = msg;
                        return;
                    }
                }

                match account.ledger.filter_date_month.parse::<u32>() {
                    Ok(month_input) => _month = month_input,
                    Err(err) => {
                        let mut msg = "Parse Month error: ".to_string();
                        msg.push_str(&err.to_string());
                        account.error_str = msg;
                        return;
                    }
                }

                match TimeZone::with_ymd_and_hms(&Utc, _year, _month, 1, 0, 0, 0) {
                    LocalResult::None | LocalResult::Ambiguous(_, _) => {
                        account.ledger.filter_date = DateTime::<Utc>::default();
                        account.error_str = "Filter Date Month: invalid string passed".to_string();
                        return;
                    }
                    LocalResult::Single(date) => {
                        account.error_str = String::new();
                        account.ledger.filter_date = date
                    }
                }
            }
        }
        self.save();
    }

    fn view(&self) -> Element<Message> {
        match self.screen {
            Screen::Accounts => {
                let mut columns = self.list_accounts();
                columns = columns.push(text(&self.error_str).size(TEXT_SIZE));
                columns.into()
            }
            Screen::Account(i) => {
                let account = &self.accounts[i];
                let columns = account.ledger.list_transactions();
                let columns = columns.push(text(account.error_str.clone()).size(TEXT_SIZE));
                columns.into()
            }
            Screen::Monthly(i) => {
                let account = &self.accounts[i];
                let columns = account.ledger.list_monthly();
                let columns = columns.push(text(account.error_str.clone()).size(TEXT_SIZE));
                columns.into()
            }
        }
    }

    fn theme(&self) -> iced::Theme {
        iced::Theme::default()
    }

    fn style(&self) -> iced::theme::Application {
        iced::theme::Application::default()
    }

    fn scale_factor(&self) -> f64 {
        1.0
    }

    fn run(settings: iced::Settings<()>) -> Result<(), iced::Error>
    where
        Self: 'static + Sized,
    {
        <Self as iced::Application>::run(settings)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Account {
    name: String,
    ledger: Ledger,
    error_str: String,
}

impl Account {
    pub fn new(name: String) -> Self {
        Account {
            name,
            ledger: Ledger::new(),
            error_str: String::new(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
enum Screen {
    Accounts,
    Account(usize),
    Monthly(usize),
}
