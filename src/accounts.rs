use chrono::{offset::Utc, DateTime, Months};
use iced::widget::{button, column, row, text, text_input, Column};
use iced::{Sandbox, Element};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use thousands::Separable;

use std::cmp::max;
use std::fs::File;
use std::io::prelude::*;
use std::io::Stdin;

use crate::ledger::Ledger;

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
    account_name: String,
    view_account: Option<usize>,

    accounts: Vec<Account>,
    checked_up_to: DateTime<Utc>,
}

impl Accounts {
    pub fn list_accounts(&self) -> Column<Message> {
        let mut account_name_len = 0;
        let mut account_balance_len = 0;
        for account in self.accounts.iter() {
            let name_len = account.name.len();
            if name_len > account_name_len {
                account_name_len = name_len;
            }
            let balance = account.ledger.sum();
            let balance_len = balance.separate_with_commas().len();
            if balance_len > account_balance_len {
                account_balance_len = balance_len;
            }
        }
        let account_str = "Account";
        account_name_len = max(account_str.len(), account_name_len);
        let balance_str = "Balance";
        account_balance_len = max(balance_str.len(), account_balance_len);

        let header = format!(
            "{:^account_name_len$} {:^account_balance_len$}",
            account_str,
            balance_str,
            account_name_len = account_name_len,
            account_balance_len = account_balance_len
        );

        let seperator = format!(
            "{}-----{}----",
            "-".repeat(account_name_len),
            "-".repeat(account_balance_len)
        );
        
        let mut table = column![text(header).size(25), text(seperator).size(50)];

        let mut total = dec!(0.00);
        for (i, account) in self.accounts.iter().enumerate() {
            let sum = account.ledger.sum();
            total += sum;
            let row = format!(
                "{:<account_name_len$} {:>account_balance_len$}",
                account.name,
                sum.separate_with_commas(),
                account_name_len = account_name_len,
                account_balance_len = account_balance_len,
            );
            table = table.push(row![
                text(row).size(25),
                button(" Select ").on_press(Message::SelectAccount(i)),
                button(" Delete ").on_press(Message::DeleteAccount(i)),
            ]);
        }

        let total = format!("\ntotal: {:}\n", total.separate_with_commas());
        table = table.push(text(total).size(25));

        table
    }

    pub fn project_months(&mut self, stdin: &mut Stdin) {
        println!("months:");
        let string;
        if let Some(Ok(line)) = stdin.lock().lines().next() {
            string = line;
        } else {
            println!("expected input");
            return;
        }

        let months = match string.parse::<u32>() {
            Ok(i) => i,
            Err(_) => {
                println!("expected an integer");
                return;
            }
        };

        let mut accounts = self.clone();
        let mut transactions = Vec::new();
        for account in accounts.accounts.iter_mut() {
            for tx in account.ledger.data.iter_mut() {
                if tx.repeats_monthly {
                    for date in DateRange(
                        tx.date,
                        self.checked_up_to
                            .checked_add_months(Months::new(months))
                            .unwrap(),
                    )
                    .skip(1)
                    {
                        tx.repeats_monthly = false;
                        let mut tx_copy = tx.clone();
                        tx_copy.date = date;
                        transactions.push(tx_copy)
                    }
                    let len = transactions.len();
                    if len > 0 {
                        transactions[len - 1].repeats_monthly = true;
                    }
                }
            }
            account.ledger.data.append(&mut transactions);
        }

        accounts.list_accounts();
    }

    pub fn save(&self) -> std::io::Result<()> {
        let j = serde_json::to_string_pretty(&self).unwrap();
        let mut file = File::create("data/ledger.json")?;
        file.write_all(j.as_bytes())
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
    ChangeComment(String),
    DeleteAccount(usize),
    NewAccount,
    SelectAccount(usize),
    SubmitTx,
}

impl Sandbox for Accounts {
    type Message = Message;

    fn new() -> Self {
        // Accounts { account_name: "".to_string(), accounts: Vec::new(), checked_up_to: DateTime::<Utc>::default(), view_account: None }
        Accounts::load()
    }

    fn title(&self) -> String {
        String::from("Ledger")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Back => {
                self.view_account = None;
            },
            Message::ChangeAccountName(name) => {
                self.account_name = name;
            },
            // TODO: Make handling of the '.' nicer.
            Message::ChangeTx(tx) => {
                if tx.len() == 2 && tx.ends_with('.') {
                    self.accounts[self.view_account.unwrap()].ledger.amount.push('.');
                } else {
                    match  Decimal::from_str_exact(&tx) {
                        Ok(tx) =>  {
                            self.accounts[self.view_account.unwrap()].ledger.tx.amount = tx;
                            self.accounts[self.view_account.unwrap()].ledger.amount = tx.to_string();
                        },
                        Err(_) => {
                            self.accounts[self.view_account.unwrap()].ledger.amount = String::new();
                        }, 
                    }
                }
            },
            Message::ChangeComment(comment) => {
                self.accounts[self.view_account.unwrap()].ledger.tx.comment = comment;
            },
            Message::DeleteAccount(i) => {
                self.accounts.remove(i);
            },
            Message::NewAccount => {
                self.accounts.push(Account::new(self.account_name.clone()));
                self.account_name = "".to_string();
            },
            Message::SelectAccount(i) => {
                self.view_account = Some(i);
            },
            Message::SubmitTx => {
                let account = &mut self.accounts[self.view_account.unwrap()];
                account.ledger.data.push(account.ledger.tx.clone());
            }
        }
        // TODO: print a message and loop on error..
        self.save().unwrap();
    }

    fn view(&self) -> Element<Message> {
        match self.view_account {
            None => {
                let rows = self.list_accounts();
                let rows= rows.push(
                    row![
                        text_input("", &self.account_name)
                        .on_submit(Message::NewAccount)
                        .on_input(|name| Message::ChangeAccountName(name)),
                    ]
                );
                rows.into()
            },
            Some(i) => {
                let rows = self.accounts[i].ledger.list_transactions();
                let rows = rows.push(button("Back").on_press(Message::Back));
                rows.into()
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
}

impl Account {
    pub fn new(name: String) -> Self {
        Account {
            name,
            ledger: Ledger::new(),
        }
    }
}
