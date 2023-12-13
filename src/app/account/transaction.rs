use ::serde::{Deserialize, Serialize};
use chrono::{serde::ts_seconds, DateTime, Utc};
use rust_decimal::Decimal;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Transaction {
    pub amount: Decimal,
    pub comment: String,
    #[serde(with = "ts_seconds")]
    pub date: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TransactionMonthly {
    pub amount: Decimal,
    pub comment: String,
}

impl From<Transaction> for TransactionMonthly {
    fn from(transaction: Transaction) -> Self {
        TransactionMonthly {
            amount: transaction.amount,
            comment: transaction.comment,
        }
    }
}

#[derive(Clone, Debug)]
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
