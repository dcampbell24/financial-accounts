//! A financial account.

use chrono::{DateTime, LocalResult, NaiveDate, TimeZone, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::ledger::{Ledger, Transaction};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Account {
    pub name: String,
    pub ledger: Ledger,
    pub error_str: String,
}

impl Account {
    pub fn new(name: String) -> Self {
        Account {
            name,
            ledger: Ledger::new(),
            error_str: String::new(),
        }
    }

    pub fn submit_filter_date(&self) -> Result<DateTime<Utc>, String> {
        let mut _year = 0;
        let mut _month = 0;

        if self.ledger.filter_date_year.is_empty() && self.ledger.filter_date_month.is_empty() {
            return Ok(DateTime::<Utc>::default());
        }
        match self.ledger.filter_date_year.parse::<i32>() {
            Ok(year_input) => _year = year_input,
            Err(err) => {
                let mut msg = "Parse Year error: ".to_string();
                msg.push_str(&err.to_string());
                return Err(msg);
            }
        }
        match self.ledger.filter_date_month.parse::<u32>() {
            Ok(month_input) => _month = month_input,
            Err(err) => {
                let mut msg = "Parse Month error: ".to_string();
                msg.push_str(&err.to_string());
                return Err(msg);
            }
        }
        match TimeZone::with_ymd_and_hms(&Utc, _year, _month, 1, 0, 0, 0) {
            LocalResult::None | LocalResult::Ambiguous(_, _) => {
                Err("Filter Date error: invalid string passed".to_string())
            }
            LocalResult::Single(date) => Ok(date),
        }
    }

    pub fn submit_tx(&self) -> Result<Transaction, String> {
        let amount_str = self.ledger.tx.amount.clone();
        let amount = match Decimal::from_str_exact(&amount_str) {
            Ok(tx) => tx,
            Err(err) => {
                let mut msg = "Parse Amount error: ".to_string();
                msg.push_str(&err.to_string());
                return Err(msg);
            }
        };
        let mut date = Utc::now();
        if !self.ledger.tx.date.is_empty() {
            match NaiveDate::parse_from_str(&self.ledger.tx.date, "%Y-%m-%d") {
                Ok(naive_date) => {
                    date = naive_date.and_hms_opt(0, 0, 0).unwrap().and_utc();
                }
                Err(err) => {
                    let mut msg = "Parse Date error: ".to_string();
                    msg.push_str(&err.to_string());
                    return Err(msg);
                }
            }
        }
        let comment = self.ledger.tx.comment.clone();
        Ok(Transaction {
            amount,
            comment,
            date,
        })
    }
}
