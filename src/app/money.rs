use std::fmt;

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
pub enum Currency {
    Eth,
    Gno,
    GoldOz,
    Usd,
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Currency::Eth => write!(f, "ETH"),
            Currency::Gno => write!(f, "GNO"),
            Currency::GoldOz => write!(f, "gold Oz"),
            Currency::Usd => write!(f, "USD"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
pub struct CurrencySummed {
    eth: Decimal,
    gno: Decimal,
    gold_oz: Decimal,
    usd: Decimal,
}

impl CurrencySummed {
    fn new() -> Self {
        CurrencySummed {
            eth: dec!(0), gno: dec!(0), gold_oz: dec!(0), usd: dec!(0)
        }
    }

    fn add(&mut self, amount: Decimal, currency: Currency) {
        match currency {
            Currency::Eth => self.eth += amount,
            Currency::Gno => self.gno += amount,
            Currency::GoldOz => self.gold_oz += amount,
            Currency::Usd => self.usd += amount,
        }
    }
}

impl fmt::Display for CurrencySummed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ETH {} GNO {} gold Oz {} USD", self.eth, self.gno, self.gold_oz, self.usd)
    }
}
