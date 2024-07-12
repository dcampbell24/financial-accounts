use std::error::Error;

use chrono::Utc;
use reqwest::blocking::Client;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use crate::app::{crypto, metals, money::Currency};

use super::transaction::Transaction;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Txs2nd {
    pub currency: Currency,
    pub txs: Vec<Transaction>,
}

impl Txs2nd {
    pub fn new(currency: Currency) -> Self {
        Txs2nd {
            currency,
            txs: Vec::new(),
        }
    }

    pub fn get_ohlc(&self) -> Result<Transaction, Box<dyn Error>> {
        let http_client = Client::new();

        match self.currency {
            Currency::Btc => {
                let btc = crypto::get_ohlc_bitcoin(&http_client)?;
                let count: Decimal = self.txs.iter().map(|tx| tx.amount).sum();
                Ok(Transaction {
                    amount: dec!(0),
                    balance: count * btc.close,
                    date: Utc::now(),
                    comment: format!("OHLC: {count} {} at {} USD", self.currency, btc.close),
                })
            }
            Currency::Eth => {
                let eth = crypto::get_ohlc_eth(&http_client)?;
                let count: Decimal = self.txs.iter().map(|tx| tx.amount).sum();
                Ok(Transaction {
                    amount: dec!(0),
                    balance: count * eth.close,
                    date: Utc::now(),
                    comment: format!("OHLC: {count} {} at {} USD", self.currency, eth.close),
                })
            }
            Currency::Gno => {
                let gno = crypto::get_ohlc_gno(&http_client)?;
                let count: Decimal = self.txs.iter().map(|tx| tx.amount).sum();
                Ok(Transaction {
                    amount: dec!(0),
                    balance: count * gno.close,
                    date: Utc::now(),
                    comment: format!("OHLC: {count} {} at {} USD", self.currency, gno.close),
                })
            }
            Currency::GoldOz => {
                let gold = metals::get_price_gold(&http_client)?;
                let count: Decimal = self.txs.iter().map(|tx| tx.amount).sum();
                Ok(Transaction {
                    amount: dec!(0),
                    balance: count * gold.price,
                    date: Utc::now(),
                    comment: format!("OHLC: {count} {} at {} USD", self.currency, gold.price),
                })
            }
            Currency::Usd => panic!("You can't hold USD as a secondary currency!"),
        }
    }
}
