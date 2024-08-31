use anyhow::Context;
use chrono::{offset::Utc, DateTime, Datelike, TimeZone};
use ron::ser::PrettyConfig;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::ops::{Index, IndexMut};
use std::path::PathBuf;

use crate::app::account::{transaction::Transaction, Account};

use super::account::transactions::Transactions;
use super::crypto::Crypto;
use super::metal::Metal;
use super::money::{Currency, Fiat};
use super::stocks::StockPlus;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Accounts {
    checked_up_to: DateTime<Utc>,
    #[serde(rename = "accounts")]
    pub inner: Vec<Account>,
    pub groups: Vec<Group>,
    pub crypto: Vec<Crypto>,
    pub fiats: Vec<Fiat>,
    pub metals: Vec<Metal>,
    pub stocks_plus: Vec<StockPlus>,
}

impl Accounts {
    pub fn sort(&mut self) {
        self.inner.sort_by_key(|account| account.name.clone());
    }

    // Fixme: what to do when transactions are not in USD?
    pub fn all_accounts_txs_1st(&self) -> Transactions<Fiat> {
        let mut transactions = Transactions::new(Fiat::Usd);
        for account in &self.inner {
            if account.txs_1st.currency == Fiat::Usd {
                for tx in &account.txs_1st.txs {
                    transactions.txs.push(tx.clone());
                }
            }
        }
        transactions.sort();

        let mut balance = dec!(0);
        for tx in &mut transactions.txs {
            balance += tx.amount;
            tx.balance = balance;
        }
        transactions
    }

    pub fn check_monthly(&mut self) {
        let past = self.checked_up_to;
        let now = Utc::now();
        let day_1 = TimeZone::with_ymd_and_hms(&Utc, now.year(), now.month(), 1, 0, 0, 0).unwrap();

        if day_1 >= past && day_1 < now {
            for account in &mut self.inner {
                let mut balance = account.balance_1st();
                for tx in &account.txs_monthly {
                    balance += tx.amount;
                    account.txs_1st.txs.push(Transaction {
                        amount: tx.amount,
                        balance,
                        comment: tx.comment.clone(),
                        date: day_1,
                    });
                }
                account.txs_1st.sort();
            }
        }
        self.checked_up_to = now;
    }

    #[must_use]
    pub async fn get_all_prices(&mut self) -> Vec<anyhow::Error> {
        let mut tasks = Vec::new();
        let mut indexes = Vec::new();
        for (index, account) in &mut self.inner.iter().enumerate() {
            if account.txs_2nd.is_some() {
                indexes.push(index);
                tasks.push(account.submit_price_as_transaction());
            }
        }

        let results = futures::future::join_all(tasks).await;
        let mut errors = Vec::new();
        for (index, result) in indexes.into_iter().zip(results) {
            let account = &mut self.inner[index];
            match result {
                Ok(tx) => {
                    account.txs_1st.txs.push(tx);
                    account.txs_1st.sort();
                }
                Err(error) => {
                    errors.push(error);
                }
            }
        }
        errors
    }

    pub fn get_currencies(&self) -> Vec<Currency> {
        let mut currencies = Vec::new();
        for crypto in &self.crypto {
            currencies.push(Currency::Crypto(crypto.clone()));
        }
        for fiat in &self.fiats {
            currencies.push(Currency::Fiat(fiat.clone()));
        }
        for metal in &self.metals {
            currencies.push(Currency::Metal(metal.clone()));
        }
        for stock_plus in &self.stocks_plus {
            currencies.push(Currency::StockPlus(stock_plus.clone()));
        }
        currencies
    }

    pub fn new() -> Self {
        Self {
            checked_up_to: DateTime::<Utc>::default(),
            inner: Vec::new(),
            groups: Vec::new(),
            crypto: Vec::new(),
            fiats: Vec::new(),
            metals: Vec::new(),
            stocks_plus: Vec::new(),
        }
    }

    pub fn project_months(&self, months: Option<u16>) -> Decimal {
        months.map_or_else(
            || self.balance_usd(),
            |months| self.balance_usd() + self.total_for_months_usd(months),
        )
    }

    pub fn balance_usd(&self) -> Decimal {
        let mut balance = dec!(0);
        for account in &self.inner {
            if account.txs_1st.currency == Fiat::Usd {
                balance += account.balance_1st();
            }
        }
        balance
    }

    pub fn total_for_months_usd(&self, project_months: u16) -> Decimal {
        let mut total = dec!(0);
        for account in &self.inner {
            if account.txs_1st.currency == Fiat::Usd {
                let sum = account.sum_monthly();
                let times: Decimal = project_months.into();
                total += sum * times;
            }
        }
        total
    }

    pub fn total_for_last_week_usd(&self) -> Decimal {
        let mut total = dec!(0);
        for account in &self.inner {
            if account.txs_1st.currency == Fiat::Usd {
                let sum = account.sum_last_week();
                total += sum;
            }
        }
        total
    }

    pub fn total_for_last_month_usd(&self) -> Decimal {
        let mut total = dec!(0);
        for account in &self.inner {
            if account.txs_1st.currency == Fiat::Usd {
                let sum = account.sum_last_month();
                total += sum;
            }
        }
        total
    }

    pub fn total_for_last_year_usd(&self) -> Decimal {
        let mut total = dec!(0);
        for account in &self.inner {
            if account.txs_1st.currency == Fiat::Usd {
                let sum = account.sum_last_year();
                total += sum;
            }
        }
        total
    }

    pub fn save_first(&self, file_path: &PathBuf) -> anyhow::Result<()> {
        let pretty_config = PrettyConfig::new();
        let j = ron::ser::to_string_pretty(self, pretty_config)?;
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(file_path)?;
        file.write_all(j.as_bytes())?;
        Ok(())
    }

    pub fn save(&self, file_path: Option<&PathBuf>) -> anyhow::Result<()> {
        let pretty_config = PrettyConfig::new();
        let j = ron::ser::to_string_pretty(self, pretty_config)?;
        let file_path = file_path.context("Cannot save because file path is empty!")?;
        let mut file = File::create(file_path)?;
        file.write_all(j.as_bytes())?;
        Ok(())
    }

    pub fn load(file_path: &PathBuf) -> anyhow::Result<Self> {
        let mut buf = String::new();
        let mut file = File::open(file_path)?;
        file.read_to_string(&mut buf)?;
        let accounts = ron::from_str(&buf)?;
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Group {
    pub name: String,
    pub members: Vec<usize>,
}

impl Group {
    fn remove_inner(&mut self, index: usize) -> Option<usize> {
        for (remove, i) in &mut self.members.iter().enumerate() {
            if index == *i {
                return Some(remove);
            }
        }

        None
    }

    pub fn remove(&mut self, index: usize) -> Option<usize> {
        for i in &mut self.members.iter_mut() {
            if *i > index {
                *i -= 1;
            }
        }

        if let Some(index) = self.remove_inner(index) {
            return Some(self.members.remove(index));
        }

        None
    }
}
