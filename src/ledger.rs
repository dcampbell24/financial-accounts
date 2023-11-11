use chrono::serde::ts_seconds;
use chrono::{offset::Utc, DateTime};
use chrono::{Datelike, Months, TimeZone};
use iced::widget::{button, column, row, text, text_input, Column};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use thousands::Separable;

use crate::accounts::Message;
use crate::TEXT_SIZE;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Ledger {
    pub tx: TransactionToSubmit,
    pub data: Vec<Transaction>,
    pub monthly: Vec<Transaction>,
    pub filter_date: DateTime<Utc>,
    pub filter_date_year: String,
    pub filter_date_month: String,
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
            monthly: Vec::new(),
            filter_date: DateTime::<Utc>::default(),
            filter_date_year: String::new(),
            filter_date_month: String::new(),
        }
    }

    pub fn list_transactions(&self) -> Column<Message> {
        let mut col_1 = column![text("Amount ").size(TEXT_SIZE)];
        let mut col_2 = column![text("Date ").size(TEXT_SIZE)];
        let mut col_3 = column![text("Comment ").size(TEXT_SIZE)];
        let mut col_4 = column![text("").size(TEXT_SIZE)];

        let mut total = dec!(0);

        let mut filtered_tx = Vec::new();
        for tx in self.data.iter() {
            if tx.date > self.filter_date
                && tx.date < self.filter_date.checked_add_months(Months::new(1)).unwrap()
            {
                filtered_tx.push(tx.clone())
            }
        }

        let txs;
        if self.filter_date == DateTime::<Utc>::default() {
            txs = self.data.iter();
        } else {
            txs = filtered_tx.iter();
        }

        for (i, tx) in txs.enumerate() {
            total += tx.amount;
            col_1 = col_1.push(text(tx.amount.separate_with_commas()).size(TEXT_SIZE));
            col_2 = col_2.push(text(tx.date.format("%Y-%m-%d %Z ")).size(TEXT_SIZE));
            col_3 = col_3.push(text(tx.comment.clone()).size(TEXT_SIZE));
            col_4 = col_4.push(button("Delete").on_press(Message::Delete(i)));
        }

        let rows = row![col_1, col_2, col_3, col_4];

        let row = row![
            text_input("Amount", &self.tx.amount).on_input(|amount| Message::ChangeTx(amount)),
            text_input("Date", &self.tx.date).on_input(|date| Message::ChangeDate(date)),
            text_input("Comment", &self.tx.comment)
                .on_input(|comment| Message::ChangeComment(comment)),
            button("Add").on_press(Message::SubmitTx),
        ];

        let filter_date = row![
            text_input("Year", &self.filter_date_year)
                .on_input(|date| Message::ChangeFilterDateYear(date)),
            text_input("Month", &self.filter_date_month)
                .on_input(|date| Message::ChangeFilterDateMonth(date)),
            button("Filter").on_press(Message::SubmitFilterDate),
            text(&self.filter_date).size(TEXT_SIZE),
        ];

        column![
            rows,
            text(format!("\ntotal: {}\n", total.separate_with_commas())).size(TEXT_SIZE),
            row,
            filter_date,
            button("Back").on_press(Message::Back),
        ]
    }

    pub fn list_monthly(&self) -> Column<Message> {
        let mut col_1 = column![text("Amount ").size(TEXT_SIZE)];
        let mut col_2 = column![text("Comment ").size(TEXT_SIZE)];
        let mut col_3 = column![text("").size(TEXT_SIZE)];

        let mut total = dec!(0);
        for (i, tx) in self.monthly.iter().enumerate() {
            total += tx.amount;
            col_1 = col_1.push(text(tx.amount.separate_with_commas()).size(TEXT_SIZE));
            col_2 = col_2.push(text(tx.comment.clone()).size(TEXT_SIZE));
            col_3 = col_3.push(button("Delete").on_press(Message::Delete(i)));
        }

        let rows = row![col_1, col_2, col_3];

        let row = row![
            text_input("Amount", &self.tx.amount).on_input(|amount| Message::ChangeTx(amount)),
            text_input("Comment", &self.tx.comment)
                .on_input(|comment| Message::ChangeComment(comment)),
            button("Add").on_press(Message::SubmitTx),
        ];

        column![
            rows,
            text(format!("\ntotal: {}\n", total.separate_with_commas())).size(TEXT_SIZE),
            row,
            button("Back").on_press(Message::Back),
        ]
    }

    pub fn sum(&self) -> Decimal {
        self.data.iter().map(|d| d.amount).sum()
    }

    pub fn sum_monthly(&self) -> Decimal {
        self.monthly.iter().map(|d| d.amount).sum()
    }

    pub fn sum_current_month(&self) -> Decimal {
        let now = Utc::now();
        let date = Utc
            .with_ymd_and_hms(now.year(), now.month(), 1, 0, 0, 0)
            .unwrap();
        let mut amount = dec!(0);
        for tx in self.data.iter() {
            if tx.date >= date {
                amount += tx.amount;
            }
        }
        amount
    }

    pub fn sum_last_month(&self) -> Decimal {
        let now = Utc::now();
        let month_start = Utc
            .with_ymd_and_hms(now.year(), now.month() - 1, 1, 0, 0, 0)
            .unwrap();
        let month_end = Utc
            .with_ymd_and_hms(now.year(), now.month(), 1, 0, 0, 0)
            .unwrap();
        let mut amount = dec!(0);
        for tx in self.data.iter() {
            if tx.date >= month_start && tx.date < month_end {
                amount += tx.amount;
            }
        }
        amount
    }

    pub fn sum_current_year(&self) -> Decimal {
        let now = Utc::now();
        let date = Utc.with_ymd_and_hms(now.year(), 1, 1, 0, 0, 0).unwrap();
        let mut amount = dec!(0);
        for tx in self.data.iter() {
            if tx.date >= date {
                amount += tx.amount;
            }
        }
        amount
    }

    pub fn sum_last_year(&self) -> Decimal {
        let now = Utc::now();
        let year_start = Utc.with_ymd_and_hms(now.year() - 1, 1, 1, 0, 0, 0).unwrap();
        let year_end = Utc.with_ymd_and_hms(now.year(), 1, 1, 0, 0, 0).unwrap();
        let mut amount = dec!(0);
        for tx in self.data.iter() {
            if tx.date >= year_start && tx.date < year_end {
                amount += tx.amount;
            }
        }
        amount
    }
}

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
pub struct TransactionToSubmit {
    pub amount: String,
    pub comment: String,
    pub date: String,
}

impl TransactionToSubmit {
    pub fn new() -> Self {
        Self {
            amount: String::new(),
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
