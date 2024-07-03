use chrono::{offset::Utc, DateTime, Datelike, TimeZone};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::ops::{Index, IndexMut};
use std::path::PathBuf;

use crate::app::account::{transaction::Transaction, Account};
use crate::app::money::Currency;
use crate::app::ticker::Ticker;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Accounts {
    checked_up_to: DateTime<Utc>,
    pub total_crypto: Decimal,
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
            total_crypto: dec!(0),
            inner: Vec::new(),
        }
    }

    pub fn project_months(&self, months: Option<u16>) -> Decimal {
        match months {
            Some(months) => self.total(Currency::Usd) + self.total_for_months_usd(months),
            None => self.total(Currency::Usd),
        }
    }

    pub fn total(&self, currency: Currency) -> Decimal {
        let mut total = dec!(0);
        for account in self.inner.iter() {
            if account.currency == currency {
                let sum = account.sum();
                total += sum;
            }
        }
        total
    }

    pub fn total_cypto(&self) -> Result<Decimal, Box<dyn Error>> {
        let ticker = Ticker::init();
        // let bitcoin = ticker._get_ohlc_bitcoin()?;
        let eth = ticker.get_ohlc_eth()?;
        let gno = ticker.get_ohlc_gno()?;

        let mut total = dec!(0);
        for account in &self.inner {
            if account.currency == Currency::Eth {
                let sum: Decimal = account.data.iter().map(|record| record.amount).sum();
                total += sum * eth.close;
            } else if account.currency == Currency::Gno {
                let sum: Decimal = account.data.iter().map(|record| record.amount).sum();
                total += sum * gno.close;
            }
        }
        Ok(total)
    }

    pub fn total_for_months_usd(&self, project_months: u16) -> Decimal {
        let mut total = dec!(0);
        for account in self.inner.iter() {
            if account.currency == Currency::Usd {
                let sum = account.sum_monthly();
                let times: Decimal = project_months.into();
                total += sum * times
            }
        }
        total
    }

    pub fn total_for_current_month_usd(&self) -> Decimal {
        let mut total = dec!(0);
        for account in self.inner.iter() {
            if account.currency == Currency::Usd {
                let sum = account.sum_current_month();
                total += sum
            }
        }
        total
    }

    pub fn total_for_last_month_usd(&self) -> Decimal {
        let mut total = dec!(0);
        for account in self.inner.iter() {
            if account.currency == Currency::Usd {
                let sum = account.sum_last_month();
                total += sum
            }
        }
        total
    }

    pub fn total_for_current_year_usd(&self) -> Decimal {
        let mut total = dec!(0);
        for account in self.inner.iter() {
            if account.currency == Currency::Usd {
                let sum = account.sum_current_year();
                total += sum
            }
        }
        total
    }

    pub fn total_for_last_year_usd(&self) -> Decimal {
        let mut total = dec!(0);
        for account in self.inner.iter() {
            if account.currency == Currency::Usd {
                let sum = account.sum_last_year();
                total += sum
            }
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

    pub fn save(&self, file_path: &PathBuf) -> Result<(), Box<dyn Error>> {
        let j = serde_json::to_string_pretty(self)?;
        let mut file = File::create(file_path)?;
        file.write_all(j.as_bytes())?;
        Ok(())
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
