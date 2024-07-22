use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
pub enum Currency {
    Btc,
    Eth,
    Gno,
    GoldOz,
    Stock(Stock),
    Usd,
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Currency::Btc => write!(f, "BTC"),
            Currency::Eth => write!(f, "ETH"),
            Currency::Gno => write!(f, "GNO"),
            Currency::GoldOz => write!(f, "Gold Troy Oz"),
            Currency::Stock(stock) => write!(f, "Stock: {stock}"),
            Currency::Usd => write!(f, "USD"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
pub struct Stock {
    pub description: String,
    pub symbol: String,
}

impl fmt::Display for Stock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description)
    }
}
