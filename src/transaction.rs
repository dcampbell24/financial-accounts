//! A financial transaction.

use ::serde::{Deserialize, Serialize};
use chrono::{serde::ts_seconds, DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Transaction {
    pub amount: Decimal,
    pub comment: String,
    #[serde(with = "ts_seconds")]
    pub date: DateTime<Utc>,
}

impl Transaction {
    pub fn new() -> Self {
        Self {
            amount: dec!(0),
            comment: String::new(),
            date: Utc::now(),
        }
    }
}

impl Default for Transaction {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TransactionMonthly {
    pub amount: Decimal,
    pub comment: String,
}

impl TransactionMonthly {
    pub fn new() -> Self {
        Self {
            amount: dec!(0),
            comment: String::new(),
        }
    }
}

impl Default for TransactionMonthly {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Transaction> for TransactionMonthly {
    fn from(transaction: Transaction) -> Self {
        TransactionMonthly {
            amount: transaction.amount,
            comment: transaction.comment,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TransactionToSubmit {
    pub amount: Option<Decimal>,
    pub comment: String,
    pub date: String,
}

impl TransactionToSubmit {
    pub fn new() -> Self {
        Self {
            amount: None,
            comment: String::new(),
            date: String::new(),
        }
    }
}

impl Default for TransactionToSubmit {
    fn default() -> Self {
        Self::new()
    }
}
