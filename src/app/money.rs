use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
pub enum Currency {
    Btc,
    Eth,
    Gno,
    Fiat(Fiat),
    Metal(Metal),
    Stock(Stock),
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Currency::Btc => write!(f, "BTC"),
            Currency::Eth => write!(f, "ETH"),
            Currency::Gno => write!(f, "GNO"),
            Currency::Fiat(fiat) => write!(f, "{fiat}"),
            Currency::Metal(metal) => write!(f, "Metal: {metal}"),
            Currency::Stock(stock) => write!(f, "Stock: {stock}"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
pub enum Fiat {
    Eur,
    Usd,
}

impl Fiat {
    pub fn symbol(&self) -> String {
        match self {
            Fiat::Eur => "EUR".to_string(),
            Fiat::Usd => "USD".to_string(),
        }
    }
}

impl fmt::Display for Fiat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Fiat::Eur => write!(f, "European Euro"),
            Fiat::Usd => write!(f, "United States Dollar"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
pub struct Metal {
    pub currency: Fiat,
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
