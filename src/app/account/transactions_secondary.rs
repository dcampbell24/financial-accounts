use std::error::Error;

use reqwest::blocking::Client;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::app::{metals, money::Currency, ticker::Ticker};

use super::transaction::Transaction;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Transactions2nd {
    pub currency: Currency,
    #[serde(rename = "transactions")]
    pub txs: Vec<Transaction>,
}

impl Transactions2nd {
    pub fn new(currency: Currency) -> Self {
        Transactions2nd {
            currency,
            txs: Vec::new(),
        }
    }

    pub fn get_ohlc(&self) -> Result<(Decimal, String), Box<dyn Error>> {
        let http_client = Client::new();
        let ticker = Ticker::init();

        match self.currency {
            Currency::Eth => {
                let eth = ticker.get_ohlc_eth()?;
                let sum: Decimal = self.txs.iter().map(|tx| tx.amount).sum();
                Ok((
                    sum * eth.close,
                    format!("OHLC: {sum} {} at {} USD", self.currency, eth.close),
                ))
            }
            Currency::Gno => {
                let gno = ticker.get_ohlc_gno()?;
                let sum: Decimal = self.txs.iter().map(|tx| tx.amount).sum();
                Ok((
                    sum * gno.close,
                    format!("OHLC: {sum} {} at {} USD", self.currency, gno.close),
                ))
            }
            Currency::GoldOz => {
                let gold = metals::get_price_gold(&http_client)?;
                let sum: Decimal = self.txs.iter().map(|tx| tx.amount).sum();
                Ok((
                    sum * gold.price,
                    format!("OHLC: {sum} {} at {} USD", self.currency, gold.price),
                ))
            }
            Currency::Usd => panic!("You can't hold USD as a secondary currency!"),
        }
    }
}
