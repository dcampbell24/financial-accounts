use chrono::serde::ts_seconds;
use chrono::{offset::Utc, DateTime};
use iced::widget::{button, column, row, text, text_input, Column};
use iced::{Alignment, Element, Sandbox};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use thousands::Separable;

use std::cmp::max;

use crate::accounts::Message;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Ledger {
    pub tx: TransactionToSubmit,
    pub data: Vec<Transaction>,
}

impl Default for Ledger {
    fn default() -> Self {
        Self::new()
    }
}

impl Ledger {
    pub fn new() -> Ledger {
        Ledger {
            tx: TransactionToSubmit::new(),
            data: Vec::new(),
        }
    }

    pub fn list_transactions(&self) -> Column<Message> {
        let tx_input = row![
            text_input("Amount", &self.tx.amount).on_input(|amount| Message::ChangeTx(amount)),
            text_input("Comment", &self.tx.comment)
                .on_input(|comment| Message::ChangeComment(comment)),
            button("Add").on_press(Message::SubmitTx),
        ];

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

        let mut str = String::new();
        str.push_str(&format!(
            "  # {:^amount_len$} {:^comment_len$} {:^14}\n",
            amount_str,
            comment_str,
            "Date",
            amount_len = amount_len,
            comment_len = comment_len,
        ));
        str.push_str(&format!(
            "{}-{}-{}\n",
            "-".repeat(amount_len),
            "-".repeat(comment_len),
            "-".repeat(18)
        ));
        let mut total = dec!(0.00);
        for (i, transaction) in self.data.iter().enumerate() {
            total += transaction.amount;
            str.push_str(&format!(
                "{i:>3} {:>amount_len$} {:<comment_len$} {:<10}\n",
                transaction.amount.separate_with_underscores(),
                transaction.comment,
                transaction.date.format("%Y-%m-%d %Z"),
                amount_len = amount_len,
                comment_len = comment_len,
            ));
        }
        str.push_str(&format!("\ntotal: {total}\n"));

        column![text(str), tx_input]
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

impl Transaction {
    pub fn new() -> Self {
        Self {
            amount: dec!(0.00),
            comment: String::new(),
            date: Utc::now(),
            repeats_monthly: false,
        }
    }
}

impl Default for Transaction {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TransactionToSubmit {
    pub amount: String,
    pub comment: String,
    pub date: String,
    pub repeats_monthly: bool,
}

impl TransactionToSubmit {
    pub fn new() -> Self {
        Self {
            amount: String::new(),
            comment: String::new(),
            date: String::new(),
            repeats_monthly: false,
        }
    }
}