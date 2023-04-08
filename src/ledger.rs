use chrono::{Duration, Local};
use chrono::serde::ts_seconds;
use chrono::{offset::Utc, DateTime, NaiveDate, NaiveDateTime, NaiveTime};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use thousands::Separable;

use std::cmp::max;
use std::io::prelude::*;
use std::io::Stdin;
use std::str::FromStr;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Ledger {
    pub data: Vec<Transaction>,
}

impl Default for Ledger {
    fn default() -> Self {
        Self::new()
    }
}

impl Ledger {
    pub fn new() -> Ledger {
        Ledger { data: Vec::new() }
    }

    pub fn create_transaction(&mut self, stdin: &mut Stdin) {
        println!("amount:");
        let mut amount = dec!(0.00);
        if let Some(Ok(line)) = stdin.lock().lines().next() {
            if let Ok(num) = Decimal::from_str(&line) {
                amount = num;
            }
        }

        println!("comment:");
        let mut comment = "".to_owned();
        if let Some(Ok(line)) = stdin.lock().lines().next() {
            comment = line;
        }

        println!("date:");
        let mut date_option = None;
        if let Some(Ok(line)) = stdin.lock().lines().next() {
            match NaiveDate::parse_from_str(&line, "%Y-%m-%d") {
                Ok(date) => {
                    let date_time =
                        NaiveDateTime::new(date, NaiveTime::from_hms_opt(0, 0, 0).unwrap());
                    date_option = Some(DateTime::<Utc>::from_local(date_time, Utc));
                }
                Err(_) => println!("using the current DateTime"),
            }
        }

        println!("repeats monthly:");
        let mut repeats_monthly = false;
        if let Some(Ok(line)) = stdin.lock().lines().next() {
            if line.trim() == "yes" {
                repeats_monthly = true;
            }
        }

        self.transaction(amount, comment, date_option, repeats_monthly);
    }

    pub fn list_transactions(&self) {
        let mut amount_len = 0;
        let mut comment_len = 0;
        for tx in self.data.iter() {
            let tx_amount_len = tx.amount.separate_with_underscores().len();
            if tx_amount_len > amount_len {
                amount_len = tx_amount_len
            }
            let tx_comment_len = tx.comment.len();
            if tx_comment_len > comment_len {
                comment_len = tx_comment_len;
            }
        }
        let amount_str = "Amount";
        amount_len = max(amount_str.len(), amount_len);
        let comment_str = "Comment";
        comment_len = max(comment_str.len(), comment_len);

        println!(
            "  # {:^amount_len$} {:^comment_len$} {:^14}",
            amount_str,
            comment_str,
            "Date",
            amount_len = amount_len,
            comment_len = comment_len,
        );
        println!(
            "{}-{}-{}",
            "-".repeat(amount_len),
            "-".repeat(comment_len),
            "-".repeat(18)
        );
        let mut total = dec!(0.00);
        for (i, transaction) in self.data.iter().enumerate() {
            total += transaction.amount;
            println!(
                "{i:>3} {:>amount_len$} {:<comment_len$} {:<10}",
                transaction.amount.separate_with_underscores(),
                transaction.comment,
                transaction.date.format("%Y-%m-%d %Z"),
                amount_len = amount_len,
                comment_len = comment_len,
            );
        }
        println!(
            "\ntotal: {total}\n");
    }

    pub fn select_transaction(&self, stdin: &mut Stdin) -> usize {
        loop {
            if let Some(Ok(line)) = stdin.lock().lines().next() {
                if let Ok(index) = line.parse::<usize>() {
                    if index >= self.data.len() {
                        println!("expected an integer equal to one of the accounts")
                    } else {
                        return index;
                    }
                } else {
                    println!("expected an integer");
                }
            }
        }
    }

    pub fn delete_transaction(&mut self, stdin: &mut Stdin) {
        println!("transaction number:");
        let index = self.select_transaction(stdin);
        self.data.remove(index);
    }

    pub fn transaction(
        &mut self,
        amount: Decimal,
        comment: String,
        date_option: Option<DateTime<Utc>>,
        repeats_monthly: bool,
    ) {
        let date = if let Some(date) = date_option {
            date
        } else {
            Utc::now()
        };

        self.data.push(Transaction {
            amount,
            comment,
            date,
            repeats_monthly,
        })
    }

    pub fn sum(&self) -> Decimal {
        self.data.iter().map(|d| d.amount).sum()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Transaction {
    pub amount: Decimal,
    pub comment: String,
    #[serde(with = "ts_seconds")]
    pub date: DateTime<Utc>,
    pub repeats_monthly: bool,
}
