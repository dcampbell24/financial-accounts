use chrono::serde::ts_seconds;
use chrono::{offset::Utc, DateTime};
use iced::widget::{button, column, row, text, text_input, Column};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use thousands::Separable;

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
        let mut col_1 = column![text("Amount ")];
        let mut col_2 = column![text("Date ")];
        let mut col_3 = column![text("Repeats Monthly ")];
        let mut col_4 = column![text("Comment ")];

        let mut total = dec!(0.00);
        for tx in self.data.iter() {
            total += tx.amount;
            col_1 = col_1.push(text(tx.amount.separate_with_underscores()));
            col_2 = col_2.push(text(tx.date.format("%Y-%m-%d %Z ")));
            col_3 = col_3.push(text(tx.repeats_monthly));
            col_4 = col_4.push(text(tx.comment.clone()));
        }

        let rows = row![col_1, col_2, col_3, col_4];

        let row = row![
            text_input("Amount", &self.tx.amount).on_input(|amount| Message::ChangeTx(amount)),
            text_input("Date", &self.tx.date).on_input(|date| Message::ChangeDate(date)),
            button("Repeats Monthly").on_press(Message::RepeatsMonthly),
            text(self.tx.repeats_monthly.to_string()),
            text_input("Comment", &self.tx.comment).on_input(|comment| Message::ChangeComment(comment)),
            button("Add").on_press(Message::SubmitTx),
        ];

        column![rows, text(format!("\ntotal: {total}\n")), row]
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