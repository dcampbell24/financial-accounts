use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
pub enum Currency {
    Btc,
    Eth,
    Gno,
    Metal(Metal),
    Stock(Stock),
    Usd,
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Currency::Btc => write!(f, "BTC"),
            Currency::Eth => write!(f, "ETH"),
            Currency::Gno => write!(f, "GNO"),
            Currency::Metal(metal) => write!(f, "Metal: {metal}"),
            Currency::Stock(stock) => write!(f, "Stock: {stock}"),
            Currency::Usd => write!(f, "USD"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
pub struct Metal {
    pub currency: String,
    pub description: String,
    pub symbol: String,
}

impl fmt::Display for Metal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description)
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
pub struct Stock {
    // currency: USD
    pub description: String,
    pub symbol: String,
}

impl fmt::Display for Stock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description)
    }
}
