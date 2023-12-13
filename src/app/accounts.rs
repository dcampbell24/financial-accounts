pub mod account;

use chrono::{offset::Utc, DateTime, Datelike, TimeZone};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::ops::{Index, IndexMut};
use std::path::PathBuf;
use std::{u64, usize};

use crate::app::accounts::account::Account;
use crate::app::accounts::account::transaction::Transaction;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Accounts {
    checked_up_to: DateTime<Utc>,
    #[serde(rename = "accounts")]
    pub inner: Vec<Account>,
}

impl Accounts {
    pub fn check_monthly(&mut self) {
        let past = self.checked_up_to;
        let now = Utc::now();
        let day_1 = TimeZone::with_ymd_and_hms(&Utc, now.year(), now.month(), 1, 0, 0, 0).unwrap();

        if day_1 >= past && day_1 < now {
            for account in self.inner.iter_mut() {
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

    pub fn new() -> Self {
        Self {
            checked_up_to: DateTime::<Utc>::default(),
            inner: Vec::new(),
        }
    }

    pub fn project_months(&self, months: Option<u16>) -> Decimal {
        match months {
            Some(months) => self.total() + self.total_for_months(months),
            None => self.total(),
        }
    }

    pub fn total(&self) -> Decimal {
        let mut total = dec!(0);
        for account in self.inner.iter() {
            let sum = account.sum();
            total += sum;
        }
        total
    }

    pub fn total_for_months(&self, project_months: u16) -> Decimal {
        let mut total = dec!(0);
        for account in self.inner.iter() {
            let sum = account.sum_monthly();
            let times: Decimal = project_months.into();
            total += sum * times
        }
        total
    }

    pub fn total_for_current_month(&self) -> Decimal {
        let mut total = dec!(0);
        for account in self.inner.iter() {
            let sum = account.sum_current_month();
            total += sum
        }
        total
    }

    pub fn total_for_last_month(&self) -> Decimal {
        let mut total = dec!(0);
        for account in self.inner.iter() {
            let sum = account.sum_last_month();
            total += sum
        }
        total
    }

    pub fn total_for_current_year(&self) -> Decimal {
        let mut total = dec!(0);
        for account in self.inner.iter() {
            let sum = account.sum_current_year();
            total += sum
        }
        total
    }

    pub fn total_for_last_year(&self) -> Decimal {
        let mut total = dec!(0);
        for account in self.inner.iter() {
            let sum = account.sum_last_year();
            total += sum
        }
        total
    }

    pub fn save_first(&self, file_path: &PathBuf) -> Result<(), Box<dyn Error>> {
        let j = serde_json::to_string_pretty(self)?;
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(file_path)?;
        file.write_all(j.as_bytes())?;
        Ok(())
    }

    pub fn save(&self, file_path: &PathBuf) {
        let j = serde_json::to_string_pretty(self).unwrap();
        let mut file = File::create(file_path).unwrap();
        file.write_all(j.as_bytes()).unwrap()
    }

    pub fn load(file_path: &PathBuf) -> Result<Self, Box<dyn Error>> {
        let mut buf = String::new();
        let mut file = File::open(file_path)?;
        file.read_to_string(&mut buf)?;
        let accounts = serde_json::from_str(&buf)?;
        Ok(accounts)
    }
}

impl Index<usize> for Accounts {
    type Output = Account;

    fn index(&self, i: usize) -> &Self::Output {
        &self.inner[i]
    }
}

impl IndexMut<usize> for Accounts {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        &mut self.inner[i]
    }
}
