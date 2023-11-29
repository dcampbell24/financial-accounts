//! All of the financial accounts owned by an entity.

use chrono::{offset::Utc, DateTime, Datelike, TimeZone};
use clap::Parser;
use iced::widget::{button, column, row, text, text_input, Column};
use iced::{Element, Sandbox, Alignment};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use thousands::Separable;

use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::{mem, u64};

use crate::account::Account;
use crate::message::Message;
use crate::transaction::{Transaction, TransactionToSubmit};
use crate::{PADDING, TEXT_SIZE};

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

#[derive(Clone, Debug, Deserialize, Serialize)]
enum Screen {
    Accounts,
    Account(usize),
    Monthly(usize),
}

impl Accounts {
    fn check_monthly(&mut self) {
        let past = self.checked_up_to;
        let now = Utc::now();
        let day_1 = TimeZone::with_ymd_and_hms(&Utc, now.year(), now.month(), 1, 0, 0, 0).unwrap();

        if day_1 >= past && day_1 < now {
            for account in self.accounts.iter_mut() {
                for tx in account.monthly.iter() {
                    account.data.push(Transaction {
                        amount: tx.amount,
                        comment: tx.comment.clone(),
                        date: day_1,
                    });
                }
                account.data.sort_by_key(|tx| tx.date);
            }
        }
        self.checked_up_to = now;
    }

    fn empty_accounts(filepath: &str) -> Self {
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

    fn total(&self) -> Decimal {
        let mut total = dec!(0);
        for account in self.accounts.iter() {
            let sum = account.sum();
            total += sum;
        }
        total
    }

    fn total_for_months(&self) -> Decimal {
        let mut total = dec!(0);
        for account in self.accounts.iter() {
            let sum = account.sum_monthly();
            let times: Decimal = self.project_months.into();
            total += sum * times
        }
        total
    }

    fn total_for_current_month(&self) -> Decimal {
        let mut total = dec!(0);
        for account in self.accounts.iter() {
            let sum = account.sum_current_month();
            total += sum
        }
        total
    }

    fn total_for_last_month(&self) -> Decimal {
        let mut total = dec!(0);
        for account in self.accounts.iter() {
            let sum = account.sum_last_month();
            total += sum
        }
        total
    }

    fn total_for_current_year(&self) -> Decimal {
        let mut total = dec!(0);
        for account in self.accounts.iter() {
            let sum = account.sum_current_year();
            total += sum
        }
        total
    }

    fn total_for_last_year(&self) -> Decimal {
        let mut total = dec!(0);
        for account in self.accounts.iter() {
            let sum = account.sum_last_year();
            total += sum
        }
        total
    }

    #[rustfmt::skip]
    fn list_accounts(&self) -> Column<Message> {
        let mut col_0 = column![text("Account").size(TEXT_SIZE)].padding(PADDING);
        let mut col_1 = column![text("Current Month").size(TEXT_SIZE)].padding(PADDING).align_items(Alignment::End);
        let mut col_2 = column![text("Last Month").size(TEXT_SIZE)].padding(PADDING).align_items(Alignment::End);
        let mut col_3 = column![text("Current Year").size(TEXT_SIZE)].padding(PADDING).align_items(Alignment::End);
        let mut col_4 = column![text("Last Year").size(TEXT_SIZE)].padding(PADDING).align_items(Alignment::End);
        let mut col_5 = column![text("Balance").size(TEXT_SIZE)].padding(PADDING).align_items(Alignment::End);
        let mut col_6 = column![text("").size(TEXT_SIZE)].padding(PADDING);
        let mut col_7 = column![text("").size(TEXT_SIZE)].padding(PADDING);
        let mut col_8 = column![text("").size(TEXT_SIZE)].padding(PADDING);
        let mut col_9 = column![text("").size(TEXT_SIZE)].padding(PADDING);

        for (i, account) in self.accounts.iter().enumerate() {
            let total = account.sum();
            let current_month = account.sum_current_month();
            let last_month = account.sum_last_month();
            let current_year = account.sum_current_year();
            let last_year = account.sum_last_year();
            col_0 = col_0.push(text(&account.name).size(TEXT_SIZE));
            col_1 = col_1.push(text(current_month.separate_with_commas()).size(TEXT_SIZE));
            col_2 = col_2.push(text(last_month.separate_with_commas()).size(TEXT_SIZE));
            col_3 = col_3.push(text(current_year.separate_with_commas()).size(TEXT_SIZE));
            col_4 = col_4.push(text(last_year.separate_with_commas()).size(TEXT_SIZE));
            col_5 = col_5.push(text(total.separate_with_commas()).size(TEXT_SIZE));
            col_6 = col_6.push(button("Tx").on_press(Message::SelectAccount(i)));
            col_7 = col_7.push(button("Monthly Tx").on_press(Message::SelectMonthly(i)));
            col_8 = col_8.push(button("Update Name").on_press(Message::UpdateAccount(i)));
            col_9 = col_9.push(button("Delete").on_press(Message::Delete(i)));
        }
        let rows = row![col_0, col_1, col_2, col_3, col_4, col_5, col_6, col_7, col_8, col_9];

        let col_1 = column![
            text("total current month: ").size(TEXT_SIZE),
            text("total last month: ").size(TEXT_SIZE),
            text("total current year: ").size(TEXT_SIZE),
            text("total last year: ").size(TEXT_SIZE),
            text("total: ").size(TEXT_SIZE),
        ];
        let col_2 = column![
            text(self.total_for_current_month().separate_with_commas()).size(TEXT_SIZE),
            text(self.total_for_last_month().separate_with_commas()).size(TEXT_SIZE),
            text(self.total_for_current_year().separate_with_commas()).size(TEXT_SIZE),
            text(self.total_for_last_year().separate_with_commas()).size(TEXT_SIZE),
            text(self.total().separate_with_commas()).size(TEXT_SIZE),
        ].align_items(Alignment::End);
        let totals = row![col_1, col_2];

        let cols = column![
            rows,
            totals,
            row![
                text("Account ").size(TEXT_SIZE),
                text_input("Name", &self.name)
                    .on_submit(Message::NewAccount)
                    .on_input(Message::ChangeAccountName)
            ],
            row![
                text("Project ").size(TEXT_SIZE),
                text_input("Months", &self.project_months_str)
                    .on_input(Message::ChangeProjectMonths)
                    .on_submit(Message::ProjectMonths),
                text((self.total() + self.total_for_months()).separate_with_commas()).size(TEXT_SIZE),
            ],
            text(&self.error_str).size(TEXT_SIZE),
            // text(format!("Checked Up To: {}", self.checked_up_to.to_string())).size(TEXT_SIZE),
        ];
        cols
    }

    fn save_first(&self) {
        let j = serde_json::to_string_pretty(&self).unwrap();
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&self.filepath)
            .unwrap();
        file.write_all(j.as_bytes()).unwrap()
    }

    fn save(&self) {
        let j = serde_json::to_string_pretty(&self).unwrap();
        let mut file = File::create(&self.filepath).unwrap();
        file.write_all(j.as_bytes()).unwrap()
    }

    fn load(filepath: &str) -> Self {
        let mut buf = String::new();
        let mut file = File::open(filepath).unwrap();
        file.read_to_string(&mut buf).unwrap();
        serde_json::from_str(&buf).unwrap()
    }

    fn selected_account(&self) -> Option<usize> {
        match self.screen {
            Screen::Accounts => None,
            Screen::Account(account) | Screen::Monthly(account) => Some(account),
        }
    }

    fn list_monthly(&self) -> bool {
        match self.screen {
            Screen::Accounts | Screen::Account(_) => false,
            Screen::Monthly(_) => true,
        }
    }
}

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

impl Sandbox for Accounts {
    type Message = Message;

    fn new() -> Self {
        let args = Args::parse();

        if !args.load.is_empty() {
            let mut accounts = Accounts::load(&args.load);
            accounts.check_monthly();
            accounts.save();
            return accounts;
        }
        if !args.new.is_empty() {
            let accounts = Accounts::empty_accounts(&args.new);
            accounts.save_first();
            return accounts;
        }
        panic!("You must choose '--new' or '--load'");
    }

    fn title(&self) -> String {
        String::from("Ledger")
    }

    fn update(&mut self, message: Message) {
        let list_monthly = self.list_monthly();
        let selected_account = self.selected_account();

        match message {
            Message::Back => self.screen = Screen::Accounts,
            Message::ChangeAccountName(name) => self.name = name,
            Message::ChangeTx(tx) => self.accounts[selected_account.unwrap()].tx.amount = tx,
            Message::ChangeDate(date) => self.accounts[selected_account.unwrap()].tx.date = date,
            Message::ChangeComment(comment) => {
                self.accounts[selected_account.unwrap()].tx.comment = comment;
            }
            Message::ChangeProjectMonths(i) => self.project_months_str = i,
            Message::ChangeFilterDateYear(date) => {
                self.accounts[selected_account.unwrap()].filter_date_year = date;
            }
            Message::ChangeFilterDateMonth(date) => {
                self.accounts[selected_account.unwrap()].filter_date_month = date;
            }
            Message::Delete(i) => match self.screen {
                Screen::Accounts => {
                    self.accounts.remove(i);
                }
                Screen::Account(j) => {
                    self.accounts[j].data.remove(i);
                }
                Screen::Monthly(j) => {
                    self.accounts[j].monthly.remove(i);
                }
            },
            Message::NewAccount => self.accounts.push(Account::new(mem::take(&mut self.name))),
            Message::UpdateAccount(i) => self.accounts[i].name = mem::take(&mut self.name),
            Message::ProjectMonths => match self.project_months_str.parse() {
                Ok(i) => {
                    self.project_months = i;
                    self.error_str = String::new();
                }
                Err(err) => {
                    let mut msg = "Parse Project Months error: ".to_string();
                    msg.push_str(&err.to_string());
                    self.error_str = msg;
                }
            },
            Message::SelectAccount(i) => self.screen = Screen::Account(i),
            Message::SelectMonthly(i) => self.screen = Screen::Monthly(i),
            Message::SubmitTx => {
                let account = &mut self.accounts[selected_account.unwrap()];
                match account.submit_tx() {
                    Ok(tx) => {
                        if list_monthly {
                            account.monthly.push(tx);
                        } else {
                            account.data.push(tx);
                            account.data.sort_by_key(|tx| tx.date);
                        }
                        account.error_str = String::new();
                        account.tx = TransactionToSubmit::new();
                    }
                    Err(err) => {
                        account.error_str = err;
                    }
                }
            }
            Message::SubmitFilterDate => {
                let account = &mut self.accounts[selected_account.unwrap()];
                match account.submit_filter_date() {
                    Ok(date) => {
                        account.filter_date = date;
                        account.error_str = String::new();
                    }
                    Err(err) => {
                        account.filter_date = DateTime::<Utc>::default();
                        account.error_str = err;
                    }
                }
            }
        }
        self.save();
    }

    fn view(&self) -> Element<Message> {
        match self.screen {
            Screen::Accounts => self.list_accounts().into(),
            Screen::Account(i) => self.accounts[i].list_transactions().into(),
            Screen::Monthly(i) => self.accounts[i].list_monthly().into(),
        }
    }
}
