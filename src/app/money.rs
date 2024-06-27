use std::fmt;

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
            Currency::GoldOz => write!(f, "Gold Oz"),
            Currency::Usd => write!(f, "USD"),
        }
    }
}
