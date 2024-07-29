use std::fmt::Display;

use chrono::{DateTime, Months, Utc};
use reqwest::blocking::Client;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use crate::app::{crypto, houses, metals, money::Currency, mutual_funds, stocks};

use super::transaction::Transaction;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Transactions<T: Clone + Display> {
    pub currency: T,
    pub txs: Vec<Transaction>,
}

impl<T: Clone + Display> Transactions<T> {
    pub fn new(currency: T) -> Self {
        Transactions {
            currency,
            txs: Vec::new(),
        }
    }

    pub fn balance(&self) -> Decimal {
        match self.txs.last() {
            Some(tx) => tx.balance,
            None => dec!(0),
        }
    }

    fn count(&self) -> Decimal {
        self.txs.iter().map(|tx| tx.amount).sum()
    }

    pub fn filter_month(&mut self, filter_date: Option<DateTime<Utc>>) {
        if let Some(date) = filter_date {
            let mut filtered_tx = Vec::new();
            for tx in self.txs.iter() {
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
                let btc = crypto::get_ohlc_bitcoin(&http_client)?;
                let count = self.count();
                Ok(Transaction {
                    amount: dec!(0),
                    balance: count * btc.close,
                    date: Utc::now(),
                    comment: format!("{count} {} at {} USD", &self.currency, btc.close),
                })
            }
            Currency::Eth => {
                let eth = crypto::get_ohlc_eth(&http_client)?;
                let count = self.count();
                Ok(Transaction {
                    amount: dec!(0),
                    balance: count * eth.close,
                    date: Utc::now(),
                    comment: format!("{count} {} at {} USD", &self.currency, eth.close),
                })
            }
            Currency::Gno => {
                let gno = crypto::get_ohlc_gno(&http_client)?;
                let count = self.count();
                Ok(Transaction {
                    amount: dec!(0),
                    balance: count * gno.close,
                    date: Utc::now(),
                    comment: format!("{count} {} at {} USD", &self.currency, gno.close),
                })
            }
            Currency::Fiat(_) => panic!("You can't hold a fiat currency as a secondary currency!"),
            Currency::Metal(metal) => {
                let gold = metals::get_price_metal(&http_client, metal)?;
                let count = self.count();
                Ok(Transaction {
                    amount: dec!(0),
                    balance: count * gold.price,
                    date: Utc::now(),
                    comment: format!("{count} {} at {} USD", &self.currency, gold.price),
                })
            }
            Currency::House(address) => {
                let house_price = houses::get_house_price(address)?;
                Ok(Transaction {
                    amount: dec!(0),
                    balance: house_price,
                    date: Utc::now(),
                    comment: address.to_string(),
                })
            }
            Currency::MutualFund(fund) => {
                let fund_price = mutual_funds::get_mutual_fund_price(&http_client, &fund.symbol)?;
                let count = self.count();
                Ok(Transaction {
                    amount: dec!(0),
                    balance: count * fund_price,
                    date: Utc::now(),
                    comment: format!("{count} {} at {} USD", &self.currency, fund_price),
                })
            }
            Currency::Stock(stock) => {
                let stock_price = stocks::get_stock_price(&http_client, stock)?;
                let count = self.count();
                Ok(Transaction {
                    amount: dec!(0),
                    balance: count * stock_price.close,
                    date: Utc::now(),
                    comment: format!("{count} {} at {} USD", &self.currency, stock_price.close),
                })
            }
        }
    }

    pub fn has_txs_2nd(&self) -> bool {
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
