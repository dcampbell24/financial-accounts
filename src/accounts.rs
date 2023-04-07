use chrono::{offset::Utc, DateTime, Months};
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use std::cmp::max;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, Stdin};

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
    accounts: Vec<Account>,
    checked_up_to: DateTime<Utc>,
}

impl Accounts {
    pub fn prompt(&mut self) {
        self.checked_up_to = Utc::now();

        let mut transactions = Vec::new();
        for account in self.accounts.iter_mut() {
            for tx in account.ledger.data.iter_mut() {
                if tx.repeats_monthly {
                    for date in DateRange(tx.date, self.checked_up_to).skip(1) {
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

        let mut stdin = io::stdin();

        'menu: loop {
            for (i, operation) in [
                "create account",
                "list accounts",
                "select account",
                "delete account",
                "project months into the future",
                "exit",
            ]
            .iter()
            .enumerate()
            {
                println!("{i}) {operation}");
            }

            let mut string = "".to_owned();
            if let Some(Ok(line)) = stdin.lock().lines().next() {
                string = line;
            }

            match string.as_str() {
                "0" => self.create_account(&mut stdin),
                "1" => self.list_accounts(),
                "2" => self.select_account(&mut stdin),
                "3" => self.delete_account(&mut stdin),
                "4" => self.project_months(&mut stdin),
                "5" => break 'menu,
                _ => println!("expected 0-4"),
            }
        }
    }

    pub fn create_account(&mut self, stdin: &mut Stdin) {
        println!("name:");
        if let Some(Ok(line)) = stdin.lock().lines().next() {
            self.new_account(line.trim().to_owned());
        }
    }

    pub fn list_accounts(&self) {
        let mut account_name_len = 0;
        let mut account_balance_len = 0;
        for account in self.accounts.iter() {
            let name_len = account.name.len();
            if name_len > account_name_len {
                account_name_len = name_len;
            }
            let balance = account.ledger.sum();
            let balance_len = balance.to_string().len();
            if balance_len > account_balance_len {
                account_balance_len = balance_len;
            }
        }
        let account_str = "Account";
        account_name_len = max(account_str.len(), account_name_len);
        let balance_str = "Balance";
        account_balance_len = max(balance_str.len(), account_balance_len);

        println!(
            "  # {:^account_name_len$} {:^account_balance_len$}",
            account_str,
            balance_str,
            account_name_len = account_name_len,
            account_balance_len = account_balance_len
        );
        println!(
            "{}-{}----",
            "-".repeat(account_name_len),
            "-".repeat(account_balance_len)
        );
        let mut total = dec!(0.00);
        for (i, account) in self.accounts.iter().enumerate() {
            let sum = account.ledger.sum();
            total += sum;
            println!(
                "{i:>3} {:<account_name_len$} {sum:>account_balance_len$}",
                account.name,
                account_name_len = account_name_len,
                account_balance_len = account_balance_len
            );
        }
        println!("\ntotal: {total}\n");
    }

    pub fn select_account_inner(&self, stdin: &mut Stdin) -> usize {
        loop {
            if let Some(Ok(line)) = stdin.lock().lines().next() {
                if let Ok(index) = line.parse::<usize>() {
                    if index >= self.accounts.len() {
                        println!("expected an integer equal to one of the accounts")
                    } else {
                        return index;
                    }
                } else {
                    println!("expected an integer");
                }
            }
        }
    }

    pub fn select_account(&mut self, stdin: &mut Stdin) {
        println!("account number:");
        let index = self.select_account_inner(stdin);
        let account = &mut self.accounts[index];

        'menu: loop {
            for (i, operation) in [
                "create transaction",
                "list transactions",
                // update transaction
                "delete transaction",
                "return to main menu",
            ]
            .iter()
            .enumerate()
            {
                println!("{i}) {operation}");
            }

            let mut string = "".to_owned();
            if let Some(Ok(line)) = stdin.lock().lines().next() {
                string = line;
            }
            match string.as_str() {
                "0" => account.ledger.create_transaction(stdin),
                "1" => account.ledger.list_transactions(),
                "2" => account.ledger.delete_transaction(stdin),
                "3" => break 'menu,
                _ => println!("expected 0-3"),
            }
        }
    }

    pub fn delete_account(&mut self, stdin: &mut Stdin) {
        println!("account number:");
        let index = self.select_account_inner(stdin);
        self.accounts.remove(index);
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

    pub fn new() -> Self {
        let accounts = Vec::new();
        let checked_up_to = Utc::now();
        Accounts {
            accounts,
            checked_up_to,
        }
    }

    pub fn new_account(&mut self, name: String) {
        self.accounts.push(Account::new(name))
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
