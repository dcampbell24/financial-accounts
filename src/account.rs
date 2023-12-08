//! A financial account.

use chrono::{DateTime, Datelike, LocalResult, Months, NaiveDate, TimeZone, Utc};
use iced::widget::{button, column, row, text, text_input, Scrollable};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use thousands::Separable;

use crate::{
    message::Message,
    transaction::{Transaction, TransactionMonthly, TransactionToSubmit},
    PADDING, TEXT_SIZE, EDGE_PADDING,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Account {
    pub name: String,
    #[serde(skip)]
    pub tx: TransactionToSubmit,
    #[serde(rename = "transactions")]
    pub data: Vec<Transaction>,
    #[serde(rename = "monthly_transactions")]
    pub monthly: Vec<TransactionMonthly>,
    #[serde(skip)]
    pub filter_date: Option<DateTime<Utc>>,
    #[serde(skip)]
    pub filter_date_year: String,
    #[serde(skip)]
    pub filter_date_month: String,
    #[serde(skip)]
    pub error_str: String,
}

impl Account {
    pub fn new(name: String) -> Self {
        Account {
            name,
            tx: TransactionToSubmit::new(),
            data: Vec::new(),
            monthly: Vec::new(),
            filter_date: None,
            filter_date_year: String::new(),
            filter_date_month: String::new(),
            error_str: String::new(),
        }
    }

    pub fn list_transactions(&self) -> Scrollable<Message> {
        let mut col_1 = column![text(" Amount ").size(TEXT_SIZE)]
            .padding(PADDING)
            .align_items(iced::Alignment::End);
        let mut col_2 = column![text(" Date ").size(TEXT_SIZE)].padding(PADDING);
        let mut col_3 = column![text(" Comment ").size(TEXT_SIZE)].padding(PADDING);
        let mut col_4 = column![text("").size(TEXT_SIZE)].padding(PADDING);

        let mut total = dec!(0);
        let mut txs = &self.data;

        let mut filtered_tx = Vec::new();
        if let Some(date) = self.filter_date {
            for tx in self.data.iter() {
                if tx.date >= date && tx.date < date.checked_add_months(Months::new(1)).unwrap() {
                    filtered_tx.push(tx.clone())
                }
            }
            txs = &filtered_tx;
        }

        for (i, tx) in txs.iter().enumerate() {
            total += tx.amount;
            col_1 = col_1.push(
                row![text(tx.amount.separate_with_commas()).size(TEXT_SIZE)].padding(PADDING),
            );
            col_2 = col_2
                .push(row![text(tx.date.format("%Y-%m-%d %Z ")).size(TEXT_SIZE)].padding(PADDING));
            col_3 = col_3.push(row![text(&tx.comment).size(TEXT_SIZE)].padding(PADDING));
            col_4 =
                col_4.push(row![button("Delete").on_press(Message::Delete(i))].padding(PADDING));
        }

        let rows = row![col_1, col_2, col_3, col_4];

        let row = row![
            text_input("Amount", &self.tx.amount).on_input(Message::ChangeTx),
            text(" "),
            text_input("Date", &self.tx.date).on_input(Message::ChangeDate),
            text(" "),
            text_input("Comment", &self.tx.comment).on_input(Message::ChangeComment),
            text(" "),
            button("Add").on_press(Message::SubmitTx),
            text(" ".repeat(EDGE_PADDING)),
        ];

        let filter_date = row![
            text_input("Year", &self.filter_date_year).on_input(Message::ChangeFilterDateYear),
            text(" "),
            text_input("Month", &self.filter_date_month).on_input(Message::ChangeFilterDateMonth),
            text(" "),
            button("Filter").on_press(Message::SubmitFilterDate),
            text(" ".repeat(EDGE_PADDING)),
        ];

        let col = column![
            rows,
            text(format!("total: {}", total.separate_with_commas())).size(TEXT_SIZE),
            row.padding(PADDING),
            filter_date.padding(PADDING),
            button("Back").on_press(Message::Back),
            text(&self.error_str).size(TEXT_SIZE),
        ];

        Scrollable::new(col)
    }

    pub fn list_monthly(&self) -> Scrollable<Message> {
        let mut col_1 = column![text(" Amount ").size(TEXT_SIZE)]
            .padding(PADDING)
            .align_items(iced::Alignment::End);
        let mut col_2 = column![text(" Comment ").size(TEXT_SIZE)].padding(PADDING);
        let mut col_3 = column![text("").size(TEXT_SIZE)].padding(PADDING);

        let mut total = dec!(0);
        for (i, tx) in self.monthly.iter().enumerate() {
            total += tx.amount;
            col_1 = col_1.push(
                row![text(tx.amount.separate_with_commas()).size(TEXT_SIZE)].padding(PADDING),
            );
            col_2 = col_2.push(row![text(&tx.comment).size(TEXT_SIZE)].padding(PADDING));
            col_3 =
                col_3.push(row![button("Delete").on_press(Message::Delete(i))].padding(PADDING));
        }

        let rows = row![col_1, col_2, col_3];

        let row = row![
            text_input("Amount", &self.tx.amount).on_input(Message::ChangeTx),
            text(" "),
            text_input("Comment", &self.tx.comment).on_input(Message::ChangeComment),
            text(" "),
            button("Add").on_press(Message::SubmitTx),
            text(" ".repeat(EDGE_PADDING)),
        ];

        let col = column![
            rows,
            text(format!("total: {}", total.separate_with_commas())).size(TEXT_SIZE),
            row.padding(PADDING),
            button("Back").on_press(Message::Back),
            text(&self.error_str).size(TEXT_SIZE),
        ];

        Scrollable::new(col)
    }

    pub fn submit_filter_date(&self) -> Result<Option<DateTime<Utc>>, String> {
        let mut _year = 0;
        let mut _month = 0;

        if self.filter_date_year.is_empty() && self.filter_date_month.is_empty() {
            return Ok(None);
        }
        match self.filter_date_year.parse::<i32>() {
            Ok(year_input) => _year = year_input,
            Err(err) => {
                let mut msg = "Parse Year error: ".to_string();
                msg.push_str(&err.to_string());
                return Err(msg);
            }
        }
        match self.filter_date_month.parse::<u32>() {
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
            LocalResult::Single(date) => Ok(Some(date)),
        }
    }

    pub fn submit_tx(&self) -> Result<Transaction, String> {
        let amount = match Decimal::from_str_exact(&self.tx.amount) {
            Ok(tx) => tx,
            Err(err) => {
                let mut msg = "Parse Amount error: ".to_string();
                msg.push_str(&err.to_string());
                return Err(msg);
            }
        };
        let mut date = Utc::now();
        if !self.tx.date.is_empty() {
            match NaiveDate::parse_from_str(&self.tx.date, "%Y-%m-%d") {
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

        Ok(Transaction {
            amount,
            comment: self.tx.comment.clone(),
            date,
        })
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
