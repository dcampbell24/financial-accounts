use chrono::{offset::Utc, Datelike, DateTime, Months, NaiveDate, TimeZone};
use iced::widget::{button, column, row, text, text_input, Column};
use iced::{Element, Sandbox};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use thousands::Separable;

use std::fs::File;
use std::io::prelude::*;
use std::mem;

use crate::ledger::{Ledger, Transaction, TransactionToSubmit};

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
    selected: Option<usize>,
    list_monthly: bool,

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
            }
        }
        self.checked_up_to = now;
    }

    pub fn list_accounts(&self) -> Column<Message> {
        let mut col_1 = column![text("Account\n\n").size(25)].padding(5);
        let mut col_2 = column![text("Balance\n\n").size(25)].padding(5);
        let mut col_3 = column![text("\n".repeat(3))].padding(5);
        let mut col_4 = column![text("\n".repeat(3))].padding(5);
        let mut col_5 = column![text("\n".repeat(3))].padding(5);

        let mut total = dec!(0.00);
        for (i, account) in self.accounts.iter().enumerate() {
            let sum = account.ledger.sum();
            total += sum;
            col_1 = col_1.push(text(&account.name).size(25));
            col_2 = col_2.push(text(sum.separate_with_commas()).size(25));
            col_3 = col_3.push(button(" Select ").on_press(Message::SelectAccount(i)));
            col_4 = col_4.push(button(" Monthly ").on_press(Message::SelectMonthly(i)));
            col_5 = col_5.push(button(" Delete ").on_press(Message::DeleteAccount(i)));
        }

        let rows = row![col_1, col_2, col_3, col_4, col_5];
        let cols = column![
            rows,
            text(format!("\ntotal: {:}", total.separate_with_commas())).size(25),
            row![
                text("Add Account ").size(25),
                text_input("", &self.name)
                    .on_submit(Message::NewAccount)
                    .on_input(|name| Message::ChangeAccountName(name))
            ],
            text(format!("Checked Up To: {}", self.checked_up_to.to_string())),
        ];
        cols
    }

    pub fn save(&self) {
        let j = serde_json::to_string_pretty(&self).unwrap();
        let mut file = File::create("data/ledger.json").unwrap();
        file.write_all(j.as_bytes()).unwrap()
    }

    pub fn load() -> Self {
        let mut buf = String::new();
        let mut f = File::open("data/ledger.json").unwrap();
        f.read_to_string(&mut buf).unwrap();
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
    DeleteAccount(usize),
    NewAccount,
    SelectAccount(usize),
    SelectMonthly(usize),
    SubmitTx,
}

impl Sandbox for Accounts {
    type Message = Message;

    fn new() -> Self {
        // Accounts { name: "".to_string(), accounts: Vec::new(), checked_up_to: DateTime::<Utc>::default(), selected: None, list_monthly: false }
        let mut self_ = Accounts::load();
        self_.check_monthly();
        self_
    }

    fn title(&self) -> String {
        String::from("Ledger")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Back => {
                self.selected = None;
            }
            Message::ChangeAccountName(name) => {
                self.name = name;
            }
            Message::ChangeTx(tx) => {
                self.accounts[self.selected.unwrap()].ledger.tx.amount = tx;
            }
            Message::ChangeDate(date) => {
                self.accounts[self.selected.unwrap()].ledger.tx.date = date;
            }
            Message::ChangeComment(comment) => {
                self.accounts[self.selected.unwrap()].ledger.tx.comment = comment
            }
            Message::DeleteAccount(i) => {
                self.accounts.remove(i);
            }
            Message::NewAccount => self.accounts.push(Account::new(mem::take(&mut self.name))),
            Message::SelectAccount(i) => {
                self.selected = Some(i);
                self.list_monthly = false;
            }
            Message::SelectMonthly(i) => {
                self.selected = Some(i);
                self.list_monthly = true;
            }
            Message::SubmitTx => {
                let account = &mut self.accounts[self.selected.unwrap()];
                let amount_str = account.ledger.tx.amount.clone(); //.clone();
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
                if self.list_monthly {
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
                }
                account.ledger.tx = TransactionToSubmit::new();
                account.error_str = String::new();
            }
        }
        self.save();
    }

    fn view(&self) -> Element<Message> {
        match self.selected {
            None => self.list_accounts().into(),
            Some(i) => {
                if self.list_monthly {
                    let account = &self.accounts[i];
                    let columns = account.ledger.list_monthly();
                    let columns = columns.push(text(account.error_str.clone()));
                    columns.into()
                } else {
                    let account = &self.accounts[i];
                    let columns = account.ledger.list_transactions();
                    let columns = columns.push(text(account.error_str.clone()));
                    columns.into()
                }
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
