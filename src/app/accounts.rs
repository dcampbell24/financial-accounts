use anyhow::Context;
use ron::ser::PrettyConfig;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use std::collections::HashSet;
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::ops::{Index, IndexMut};
use std::path::PathBuf;

use crate::app::account::Account;

use super::File;
use super::account::transactions::Transactions;
use super::crypto::Crypto;
use super::metal::Metal;
use super::money::{Currency, Fiat};
use super::stocks::StockPlus;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Accounts {
    #[serde(rename = "accounts")]
    pub inner: Vec<Account>,
    pub groups: Vec<Group>,
    pub crypto: Vec<Crypto>,
    pub fiats: Vec<Fiat>,
    pub metals: Vec<Metal>,
    pub stocks_plus: Vec<StockPlus>,
}

impl Accounts {
    pub fn currencies(&self) -> HashSet<Fiat> {
        let mut currencies = HashSet::new();
        for account in &self.inner {
            currencies.insert(account.txs_1st.currency.clone());
        }
        currencies
    }

    pub fn sort(&mut self) {
        self.inner.sort_by_key(|account| account.name.clone());
    }

    pub fn all_accounts_txs_1st(&self, currency: Fiat) -> Transactions<Fiat> {
        let mut transactions = Transactions::new(currency);
        for account in &self.inner {
            if account.txs_1st.currency == transactions.currency {
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

    #[must_use]
    pub async fn get_all_prices(&mut self) -> Vec<anyhow::Error> {
        let mut tasks = Vec::new();
        let mut indexes = Vec::new();
        for (index, account) in self.inner.iter().enumerate() {
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
            inner: Vec::new(),
            groups: Vec::new(),
            crypto: Vec::new(),
            fiats: Vec::new(),
            metals: Vec::new(),
            stocks_plus: Vec::new(),
        }
    }

    pub fn balance(&self, currency: &Fiat) -> Decimal {
        let mut balance = dec!(0);
        for account in &self.inner {
            if account.txs_1st.currency == *currency {
                balance += account.balance_1st();
            }
        }
        balance
    }

    pub fn total_for_last_week(&self, currency: &Fiat) -> (Decimal, Decimal) {
        let mut previous_total = dec!(0);
        let mut total = dec!(0);
        for account in &self.inner {
            if account.txs_1st.currency == *currency {
                let (previous_sum, sum) = account.sum_last_week();
                previous_total += previous_sum;
                total += sum;
            }
        }
        (previous_total, total)
    }

    pub fn total_for_last_month(&self, currency: &Fiat) -> (Decimal, Decimal) {
        let mut previous_total = dec!(0);
        let mut total = dec!(0);
        for account in &self.inner {
            if account.txs_1st.currency == *currency {
                let (previous_sum, sum) = account.sum_last_month();
                previous_total += previous_sum;
                total += sum;
            }
        }
        (previous_total, total)
    }

    pub fn total_for_last_year(&self, currency: &Fiat) -> (Decimal, Decimal) {
        let mut previous_total = dec!(0);
        let mut total = dec!(0);
        for account in &self.inner {
            if account.txs_1st.currency == *currency {
                let (previous_sum, sum) = account.sum_last_year();
                previous_total += previous_sum;
                total += sum;
            }
        }
        (previous_total, total)
    }

    pub fn to_string(&self) -> anyhow::Result<String> {
        let pretty_config = PrettyConfig::new();
        let string = ron::ser::to_string_pretty(self, pretty_config)?;
        Ok(string)
    }

    pub fn save_dialogue(
        &self,
        old_file: Option<File>,
        file_path: PathBuf,
    ) -> anyhow::Result<File> {
        if let Some(old_file) = old_file {
            old_file.inner.unlock()?;
        }

        if fs::exists(&file_path)? {
            let file = fs::File::open(&file_path)?;
            file.try_lock()?;
            file.unlock()?;
        }

        let mut file = fs::File::create(&file_path)?;
        file.try_lock()?;
        file.write_all(self.to_string()?.as_bytes())?;

        Ok(File {
            path: file_path,
            inner: file,
        })
    }

    pub fn save_first(&self, file_path: PathBuf) -> anyhow::Result<File> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&file_path)?;

        file.try_lock()?;
        file.write_all(self.to_string()?.as_bytes())?;

        Ok(File {
            path: file_path,
            inner: file,
        })
    }

    pub fn save(&self, old_file: Option<File>) -> anyhow::Result<File> {
        let old_file = old_file.context("Cannot save because file is None!")?;
        let file_path = old_file.path;

        let mut file = fs::File::create(&file_path)?;
        old_file.inner.unlock()?;
        file.try_lock()?;
        file.write_all(self.to_string()?.as_bytes())?;

        Ok(File {
            path: file_path,
            inner: file,
        })
    }

    pub fn load(old_file: Option<File>, file_path: PathBuf) -> anyhow::Result<(Self, File)> {
        if let Some(old_file) = old_file {
            old_file.inner.unlock()?;
        }
        let mut file = fs::File::open(&file_path)?;

        file.try_lock()?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;
        let accounts = ron::from_str(&buf)?;

        Ok((
            accounts,
            File {
                path: file_path,
                inner: file,
            },
        ))
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
