use chrono::{DateTime, Months, Utc};
use dyn_clone::DynClone;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use crate::app::money::Currency;

use super::{transaction::Transaction, transactions_secondary::Txs2nd};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Txs {
    pub txs: Vec<Transaction>,
}

impl Txs {
    pub fn new() -> Txs {
        Txs { txs: Vec::new() }
    }
}

impl Transactions for Txs {
    fn transactions(&self) -> &Vec<Transaction> {
        &self.txs
    }

    fn filter_month(&mut self, filter_date: Option<DateTime<Utc>>) {
        if let Some(txs) = filter_month(&self.txs, filter_date) {
            self.txs = txs;
        }
    }
}

impl Transactions for Txs2nd {
    fn transactions(&self) -> &Vec<Transaction> {
        &self.txs
    }

    fn currency(&self) -> Currency {
        self.currency
    }

    fn filter_month(&mut self, filter_date: Option<DateTime<Utc>>) {
        if let Some(txs) = filter_month(&self.txs, filter_date) {
            self.txs = txs;
        }
    }
}

pub trait Transactions: DynClone {
    fn transactions(&self) -> &Vec<Transaction>;

    fn balance(&self) -> Decimal {
        match self.transactions().last() {
            Some(tx) => tx.balance,
            None => dec!(0),
        }
    }

    fn filter_month(&mut self, filter_date: Option<DateTime<Utc>>);

    fn currency(&self) -> Currency {
        Currency::Usd
    }

    fn max_balance(&self) -> Option<Decimal> {
        self.transactions().iter().map(|tx| tx.balance).max()
    }

    fn min_balance(&self) -> Option<Decimal> {
        self.transactions().iter().map(|tx| tx.balance).min()
    }

    fn max_date(&self) -> Option<DateTime<Utc>> {
        self.transactions().iter().map(|tx| tx.date).max()
    }

    fn min_date(&self) -> Option<DateTime<Utc>> {
        self.transactions().iter().map(|tx| tx.date).min()
    }

    fn total(&self) -> Decimal {
        self.transactions().iter().map(|d| d.amount).sum()
    }
}

fn filter_month(
    txs: &[Transaction],
    filter_date: Option<DateTime<Utc>>,
) -> Option<Vec<Transaction>> {
    match filter_date {
        Some(date) => {
            let mut filtered_tx = Vec::new();
            for tx in txs.iter() {
                if tx.date >= date && tx.date < date.checked_add_months(Months::new(1)).unwrap() {
                    filtered_tx.push(tx.clone());
                }
            }
            Some(filtered_tx)
        }
        None => None,
    }
}
