use chrono::{DateTime, Months, Utc};
use reqwest::blocking::Client;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use crate::app::{crypto, metals, money::Currency, stocks};

use super::transaction::Transaction;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Transactions {
    pub currency: Currency,
    pub txs: Vec<Transaction>,
}

impl Transactions {
    pub fn new(currency: Currency) -> Self {
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

    pub fn get_ohlc(&self) -> anyhow::Result<Transaction> {
        let http_client = Client::new();

        match &self.currency {
            Currency::Btc => {
                let btc = crypto::get_ohlc_bitcoin(&http_client)?;
                let count = self.count();
                Ok(Transaction {
                    amount: dec!(0),
                    balance: count * btc.close,
                    date: Utc::now(),
                    comment: format!("{count} {} at {} USD", self.currency, btc.close),
                })
            }
            Currency::Eth => {
                let eth = crypto::get_ohlc_eth(&http_client)?;
                let count = self.count();
                Ok(Transaction {
                    amount: dec!(0),
                    balance: count * eth.close,
                    date: Utc::now(),
                    comment: format!("{count} {} at {} USD", self.currency, eth.close),
                })
            }
            Currency::Gno => {
                let gno = crypto::get_ohlc_gno(&http_client)?;
                let count = self.count();
                Ok(Transaction {
                    amount: dec!(0),
                    balance: count * gno.close,
                    date: Utc::now(),
                    comment: format!("{count} {} at {} USD", self.currency, gno.close),
                })
            }
            Currency::Stock(stock) => {
                let stock_price = stocks::get_stock_price(&http_client, &stock)?;
                let count = self.count();
                Ok(Transaction {
                    amount: dec!(0),
                    balance: count * stock_price.close,
                    date: Utc::now(),
                    comment: format!("{count} {} at {} USD", self.currency, stock_price.close),
                })
            }
            Currency::GoldOz => {
                let gold = metals::get_price_gold(&http_client)?;
                let count = self.count();
                Ok(Transaction {
                    amount: dec!(0),
                    balance: count * gold.price,
                    date: Utc::now(),
                    comment: format!("{count} {} at {} USD", self.currency, gold.price),
                })
            }
            Currency::Usd => panic!("You can't hold USD as a secondary currency!"),
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
