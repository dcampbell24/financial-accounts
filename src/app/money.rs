use std::{fmt, ops::Add};

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use thousands::Separable;

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
pub struct Money {
    amount: Decimal,
    unit: Unit,
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.amount.separate_with_commas(), self.unit)
    }
}

impl Add for Money {
    type Output = Option<Self>;

    fn add(mut self, other: Self) -> Option<Self> {
        if self.unit == other.unit {
            self.amount += other.amount;
            Some(self)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
pub enum Unit {
    Eth,
    Gno,
    GoldOz,
    Usd,
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Unit::Eth => write!(f, "ETH"),
            Unit::Gno => write!(f, "GNO"),
            Unit::GoldOz => write!(f, "gold Oz"),
            Unit::Usd => write!(f, "USD"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
pub struct UnitsSummed {
    eth: Money,
    gno: Money,
    gold_oz: Money,
    usd: Money,
}

impl fmt::Display for UnitsSummed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {} {}", self.eth, self.gno, self.gold_oz, self.usd)
    }
}
