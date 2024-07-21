use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Serialize)]
pub enum Currency {
    Btc,
    Eth,
    Gno,
    GoldOz,
    Stocks(Stock),
    Usd,
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Currency::Btc => write!(f, "BTC"),
            Currency::Eth => write!(f, "ETH"),
            Currency::Gno => write!(f, "GNO"),
            Currency::GoldOz => write!(f, "Gold Troy Oz"),
            Currency::Stocks(stock) => write!(f, "Stock: {stock}"),
            Currency::Usd => write!(f, "USD"),
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Serialize)]
pub enum Stock {
    Cvx,
    Csco,
    Dis,
    Chtrx, // Fixme: not found by polygon.
    Jnj,
    Kmi,
    Txn,
    Wbs,
}

impl Stock {
    pub fn description(&self) -> String {
        match self {
            Stock::Cvx => "CHEVRON CORP".to_string(),
            Stock::Csco => "CISCO SYSTEMS INC COM".to_string(),
            Stock::Dis => "DISNEY (WALT) CO COM STK".to_string(),
            Stock::Chtrx => "INVESCO CHARTER FUND".to_string(),
            Stock::Jnj => "JOHNSON AND JOHNSON COM".to_string(),
            Stock::Kmi => "KINDER MORGAN INC. DEL".to_string(),
            Stock::Txn => "TEXAS INSTRUMENTS".to_string(),
            Stock::Wbs => "WEBSTER FINL CP PV $0.01".to_string(),
        }
    }

    pub fn symbol(&self) -> String {
        match self {
            Stock::Cvx => "CVX".to_string(),
            Stock::Csco => "CSCO".to_string(),
            Stock::Dis => "DIS".to_string(),
            Stock::Chtrx => "CHTRX".to_string(),
            Stock::Jnj => "JNJ".to_string(),
            Stock::Kmi => "KMI".to_string(),
            Stock::Txn => "TXN".to_string(),
            Stock::Wbs => "WBS".to_string(),
        }
    }
}

impl fmt::Display for Stock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}
