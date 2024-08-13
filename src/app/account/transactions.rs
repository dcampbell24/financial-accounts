use std::fmt::Display;

use chrono::{DateTime, Months, TimeDelta, Utc};
use reqwest::Client;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use crate::app::{
    crypto,
    money::{Currency, Fiat},
};

use super::transaction::Transaction;

pub trait Price {
    async fn get_price(&self, client: &Client) -> anyhow::Result<Decimal>;
}

impl Price for Transactions<Currency> {
    async fn get_price(&self, client: &Client) -> anyhow::Result<Decimal> {
        match &self.currency {
            Currency::Btc => crypto::BtcOhlc::get_price(client).await,
            Currency::Eth => crypto::EthOhlc::get_price(client).await,
            Currency::Gno => crypto::GnoOhlc::get_price(client).await,
            Currency::Fiat(_) => panic!("You can't hold a fiat currency as a secondary currency!"),
            Currency::Metal(metal) => metal.get_price(client).await,
            Currency::StockPlus(stock_plus) => stock_plus.get_price(client).await,
        }
    }
}

pub trait PriceAsTransaction: Price {
    async fn get_price_as_transaction(&self) -> anyhow::Result<Transaction>;
}

impl PriceAsTransaction for Transactions<Currency> {
    async fn get_price_as_transaction(&self) -> anyhow::Result<Transaction> {
        let client = Client::builder()
        .user_agent("Mozilla/5.0 (compatible; financial-accounts/0.2-dev; +https://github.com/dcampbell24/financial-accounts)")
        .build()?;

        let price = self.get_price(&client).await?;
        let count = self.count();

        if let Currency::Metal(metal) = &self.currency {
            Ok(Transaction {
                amount: dec!(0),
                balance: count * price,
                date: Utc::now(),
                comment: format!("{count} {} at {price} {}", &self.currency, metal.currency),
            })
        } else {
            Ok(Transaction {
                amount: dec!(0),
                balance: count * price,
                date: Utc::now(),
                comment: format!("{count} {} at {price} USD", &self.currency),
            })
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Transactions<T: Clone + Display> {
    pub currency: T,
    pub txs: Vec<Transaction>,
}

impl<T: Clone + Display> Transactions<T> {
    pub const fn new(currency: T) -> Self {
        Self {
            currency,
            txs: Vec::new(),
        }
    }

    pub fn balance(&self) -> Decimal {
        self.txs.last().map_or_else(|| dec!(0), |tx| tx.balance)
    }

    fn count(&self) -> Decimal {
        self.txs.iter().map(|tx| tx.amount).sum()
    }

    pub fn filter_month(&mut self, filter_date: Option<DateTime<Utc>>) {
        if let Some(date) = filter_date {
            let mut filtered_tx = Vec::new();
            for tx in &self.txs {
                if tx.date >= date && tx.date < date.checked_add_months(Months::new(1)).unwrap() {
                    filtered_tx.push(tx.clone());
                }
            }
            self.txs = filtered_tx;
        }
    }

    pub fn last_week(&self) -> Transactions<T> {
        let last_week = Utc::now() - TimeDelta::weeks(1);
        let mut txs = Vec::new();

        for tx in &self.txs {
            if tx.date >= last_week {
                txs.push(tx.clone());
            }
        }

        Transactions {
            txs,
            currency: self.currency.clone(),
        }
    }

    pub fn last_month(&self) -> Transactions<T> {
        let last_week = Utc::now() - TimeDelta::days(30);
        let mut txs = Vec::new();

        for tx in &self.txs {
            if tx.date >= last_week {
                txs.push(tx.clone());
            }
        }

        Transactions {
            txs,
            currency: self.currency.clone(),
        }
    }

    pub fn last_year(&self) -> Transactions<T> {
        let last_week = Utc::now() - TimeDelta::days(365);
        let mut txs = Vec::new();

        for tx in &self.txs {
            if tx.date >= last_week {
                txs.push(tx.clone());
            }
        }

        Transactions {
            txs,
            currency: self.currency.clone(),
        }
    }

    pub fn max_balance(&self) -> Option<Decimal> {
        self.txs.iter().map(|tx| tx.balance).max()
    }

    pub fn min_balance(&self) -> Option<Decimal> {
        self.txs.iter().map(|tx| tx.balance).min()
    }

    pub fn max_date(&self) -> Option<DateTime<Utc>> {
        self.txs.iter().map(|tx| tx.date).max()
    }

    pub fn min_date(&self) -> Option<DateTime<Utc>> {
        self.txs.iter().map(|tx| tx.date).min()
    }

    pub fn sort(&mut self) {
        self.txs.sort_by_key(|tx| tx.date);
    }

    pub fn total(&self) -> Decimal {
        self.txs.iter().map(|d| d.amount).sum()
    }
}

impl Transactions<Currency> {
    pub const fn has_txs_2nd(&self) -> bool {
        match self.currency {
            Currency::Btc
            | Currency::Eth
            | Currency::Gno
            | Currency::Metal(_)
            | Currency::StockPlus(_) => true,
            Currency::Fiat(_) => false,
        }
    }
}

impl Transactions<Fiat> {
    pub fn remove_duplicates(&mut self, txs: &Transactions<Fiat>) {
        let mut txs_new = Vec::new();
        'outer: for tx in &self.txs {
            for tx_2nd in &txs.txs {
                if tx.date == tx_2nd.date
                    && tx.amount == tx_2nd.amount
                    && tx.comment == tx_2nd.comment
                {
                    continue 'outer;
                }
            }
            txs_new.push(tx.clone());
        }
        self.txs = txs_new;
    }
}
