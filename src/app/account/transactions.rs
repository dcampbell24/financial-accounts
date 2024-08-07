use std::fmt::Display;

use chrono::{DateTime, Months, Utc};
use reqwest::blocking::Client;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use crate::app::{
    crypto,
    money::{Currency, Fiat},
};

use super::transaction::Transaction;

pub trait Price {
    fn get_price(&self, client: &Client) -> anyhow::Result<Decimal>;
}

impl Price for Transactions<Currency> {
    fn get_price(&self, client: &Client) -> anyhow::Result<Decimal> {
        match &self.currency {
            Currency::Btc => crypto::BtcOhlc::get_price(client),
            Currency::Eth => crypto::EthOhlc::get_price(client),
            Currency::Gno => crypto::GnoOhlc::get_price(client),
            Currency::Fiat(_) => panic!("You can't hold a fiat currency as a secondary currency!"),
            Currency::Metal(metal) => metal.get_price(client),
            Currency::House(house) => house.get_price(client),
            Currency::MutualFund(fund) => fund.get_price(client),
            Currency::Stock(stock) => stock.get_price(client),
        }
    }
}

pub trait PriceAsTransaction: Price {
    fn get_price_as_transaction(&self, client: &Client) -> anyhow::Result<Transaction>;
}

impl PriceAsTransaction for Transactions<Currency> {
    fn get_price_as_transaction(&self, client: &Client) -> anyhow::Result<Transaction> {
        let price = self.get_price(client)?;
        let count = self.count();
        Ok(Transaction {
            amount: dec!(0),
            balance: count * price,
            date: Utc::now(),
            comment: format!("{count} {} at {price} USD", &self.currency),
        })
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

    pub fn total(&self) -> Decimal {
        self.txs.iter().map(|d| d.amount).sum()
    }
}

impl Transactions<Currency> {
    pub fn get_ohlc(&self) -> anyhow::Result<Transaction> {
        let http_client = Client::builder()
        .user_agent("Mozilla/5.0 (compatible; financial-accounts/0.2-dev; +https://github.com/dcampbell24/financial-accounts)")
        .build()?;

        match &self.currency {
            Currency::Btc => {
                let btc = crypto::BtcOhlc::get_price(&http_client)?;
                let count = self.count();
                Ok(Transaction {
                    amount: dec!(0),
                    balance: count * btc,
                    date: Utc::now(),
                    comment: format!("{count} {} at {} USD", &self.currency, btc),
                })
            }
            Currency::Eth => {
                let eth = crypto::EthOhlc::get_price(&http_client)?;
                let count = self.count();
                Ok(Transaction {
                    amount: dec!(0),
                    balance: count * eth,
                    date: Utc::now(),
                    comment: format!("{count} {} at {} USD", &self.currency, eth),
                })
            }
            Currency::Gno => {
                let gno = crypto::GnoOhlc::get_price(&http_client)?;
                let count = self.count();
                Ok(Transaction {
                    amount: dec!(0),
                    balance: count * gno,
                    date: Utc::now(),
                    comment: format!("{count} {} at {} USD", &self.currency, gno),
                })
            }
            Currency::Fiat(_) => panic!("You can't hold a fiat currency as a secondary currency!"),
            Currency::Metal(metal) => {
                let price = metal.get_price(&http_client)?;
                let count = self.count();
                Ok(Transaction {
                    amount: dec!(0),
                    balance: count * price,
                    date: Utc::now(),
                    comment: format!("{count} {} at {} USD", &self.currency, price),
                })
            }
            Currency::House(house) => {
                let house_price = house.get_price(&http_client)?;
                Ok(Transaction {
                    amount: dec!(0),
                    balance: house_price,
                    date: Utc::now(),
                    comment: house.to_string(),
                })
            }
            Currency::MutualFund(fund) => {
                let fund_price = fund.get_price(&http_client)?;
                let count = self.count();
                Ok(Transaction {
                    amount: dec!(0),
                    balance: count * fund_price,
                    date: Utc::now(),
                    comment: format!("{count} {} at {} USD", &self.currency, fund_price),
                })
            }
            Currency::Stock(stock) => {
                let stock_price = stock.get_price(&http_client)?;
                let count = self.count();
                Ok(Transaction {
                    amount: dec!(0),
                    balance: count * stock_price,
                    date: Utc::now(),
                    comment: format!("{count} {} at {} USD", &self.currency, stock_price),
                })
            }
        }
    }

    pub const fn has_txs_2nd(&self) -> bool {
        match self.currency {
            Currency::Btc
            | Currency::Eth
            | Currency::Gno
            | Currency::Metal(_)
            | Currency::MutualFund(_)
            | Currency::Stock(_) => true,
            Currency::Fiat(_) | Currency::House(_) => false,
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
