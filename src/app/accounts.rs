use chrono::{offset::Utc, DateTime, Datelike, TimeZone};
use ron::ser::PrettyConfig;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::ops::{Index, IndexMut};
use std::path::PathBuf;

use crate::app::account::{transaction::Transaction, Account};

use super::account::transactions::Transactions;
use super::houses::House;
use super::metal::Metal;
use super::money::{Currency, Fiat};
use super::mutual_fund::MutualFund;
use super::stocks::Stock;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Accounts {
    checked_up_to: DateTime<Utc>,
    #[serde(rename = "accounts")]
    pub inner: Vec<Account>,
    pub fiats: Vec<Fiat>,
    pub houses: Vec<House>,
    pub metals: Vec<Metal>,
    pub mutual_funds: Vec<MutualFund>,
    pub stocks: Vec<Stock>,
}

impl Accounts {
    pub fn all_accounts_txs_1st(&self) -> Transactions<Fiat> {
        let mut txs = Vec::new();
        for account in &self.inner {
            if account.txs_1st.currency == Fiat::Usd {
                for tx in &account.txs_1st.txs {
                    txs.push(tx.clone());
                }
            }
        }

        txs.sort_by_key(|tx| tx.date);
        let mut balance = dec!(0);
        for tx in &mut txs {
            balance += tx.amount;
            tx.balance = balance;
        }

        // Fixme: what to do when transactions are not in USD?
        let mut transactions = Transactions::new(Fiat::Usd);
        transactions.txs = txs;
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
                account.txs_1st.txs.sort_by_key(|tx| tx.date);
            }
        }
        self.checked_up_to = now;
    }

    pub fn get_currencies(&self) -> Vec<Currency> {
        let mut currencies = vec![Currency::Btc, Currency::Eth, Currency::Gno];
        for fiat in &self.fiats {
            currencies.push(Currency::Fiat(fiat.clone()));
        }
        for address in &self.houses {
            currencies.push(Currency::House(address.clone()));
        }
        for metal in &self.metals {
            currencies.push(Currency::Metal(metal.clone()));
        }
        for mutual_fund in &self.mutual_funds {
            currencies.push(Currency::MutualFund(mutual_fund.clone()));
        }
        for stock in &self.stocks {
            currencies.push(Currency::Stock(stock.clone()));
        }
        currencies
    }

    pub fn new() -> Self {
        Self {
            checked_up_to: DateTime::<Utc>::default(),
            inner: Vec::new(),
            fiats: Vec::new(),
            houses: Vec::new(),
            metals: Vec::new(),
            mutual_funds: Vec::new(),
            stocks: Vec::new(),
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

    pub fn total_for_current_month_usd(&self) -> Decimal {
        let mut total = dec!(0);
        for account in &self.inner {
            if account.txs_1st.currency == Fiat::Usd {
                let sum = account.sum_current_month();
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

    pub fn total_for_current_year_usd(&self) -> Decimal {
        let mut total = dec!(0);
        for account in &self.inner {
            if account.txs_1st.currency == Fiat::Usd {
                let sum = account.sum_current_year();
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

    pub fn save_first(&self, file_path: &PathBuf) -> Result<(), Box<dyn Error>> {
        let pretty_config = PrettyConfig::new();
        let j = ron::ser::to_string_pretty(self, pretty_config)?;
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(file_path)?;
        file.write_all(j.as_bytes())?;
        Ok(())
    }

    pub fn save(&self, file_path: &PathBuf) -> Result<(), Box<dyn Error>> {
        let pretty_config = PrettyConfig::new();
        let j = ron::ser::to_string_pretty(self, pretty_config)?;
        let mut file = File::create(file_path)?;
        file.write_all(j.as_bytes())?;
        Ok(())
    }

    pub fn load(file_path: &PathBuf) -> Result<Self, Box<dyn Error>> {
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
