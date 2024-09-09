use chrono::{serde::ts_seconds, DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Transaction {
    pub amount: Decimal,
    pub balance: Decimal,
    pub comment: String,
    #[serde(with = "ts_seconds")]
    pub date: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct ToSubmit {
    pub amount: Option<Decimal>,
    pub balance: Option<Decimal>,
    pub comment: String,
    pub date: String,
}

impl ToSubmit {
    pub const fn new() -> Self {
        Self {
            amount: None,
            balance: None,
            comment: String::new(),
            date: String::new(),
        }
    }

    pub fn submit_commit(&self) -> String {
        self.comment.trim().to_string()
    }
}

impl Default for ToSubmit {
    fn default() -> Self {
        Self::new()
    }
}
