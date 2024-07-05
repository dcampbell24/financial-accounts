pub mod transaction;

use std::mem::take;

use chrono::{DateTime, Datelike, Months, NaiveDate, TimeZone, Utc};
use iced::{
    widget::{button, column, row, text, text_input, Scrollable, TextInput},
    Length,
};
use plotters_iced::ChartWidget;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use crate::app::{
    account::transaction::{Transaction, TransactionMonthly, TransactionToSubmit},
    Message, EDGE_PADDING, PADDING, TEXT_SIZE,
};

use self::transaction::TransactionMonthlyToSubmit;

use super::{button_cell, chart::MyChart, money::Currency, number_cell, text_cell};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Account {
    pub name: String,
    pub currency: Currency,
    #[serde(skip)]
    pub tx: TransactionToSubmit,
    #[serde(skip)]
    pub tx_monthly: TransactionMonthlyToSubmit,
    #[serde(rename = "transactions")]
    pub data: Vec<Transaction>,
    #[serde(rename = "monthly_transactions")]
    pub monthly: Vec<TransactionMonthly>,
    #[serde(skip)]
    pub filter_date: Option<DateTime<Utc>>,
    #[serde(skip)]
    pub filter_date_year: Option<i32>,
    #[serde(skip)]
    pub filter_date_month: Option<u32>,
    #[serde(skip)]
    pub error_str: String,
}

impl Account {
    pub fn new(name: String, currency: Currency) -> Self {
        Account {
            name,
            tx: TransactionToSubmit::new(),
            tx_monthly: TransactionMonthlyToSubmit::new(),
            data: Vec::new(),
            monthly: Vec::new(),
            filter_date: None,
            filter_date_year: None,
            filter_date_month: None,
            error_str: String::new(),
            currency,
        }
    }

    fn amount_view(&self) -> TextInput<Message> {
        let amount = match &self.tx.amount {
            Some(amount) => text_input("Amount", &amount.to_string()),
            None => text_input("Amount", ""),
        };
        amount.on_input(Message::ChangeTx)
    }

    pub fn balance(&self) -> Decimal {
        match self.data.last() {
            Some(tx) => tx.balance,
            None => dec!(0),
        }
    }

    pub fn list_transactions(&self) -> Scrollable<Message> {
        let my_chart = MyChart {
            account: self.clone(),
        };
        let chart = ChartWidget::new(my_chart).height(Length::Fixed(400.0));

        let mut col_1 = column![text_cell(" Amount ")].align_items(iced::Alignment::End);
        let mut col_2 = column![text_cell(" Date ")];
        let mut col_3 = column![text_cell(" Balance ")].align_items(iced::Alignment::End);
        let mut col_4 = column![text_cell(" Comment ")];
        let mut col_5 = column![text_cell("")];

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
            col_1 = col_1.push(number_cell(tx.amount));
            col_2 = col_2.push(text_cell(tx.date.format("%Y-%m-%d %Z ")));
            col_3 = col_3.push(number_cell(tx.balance));
            col_4 = col_4.push(text_cell(&tx.comment));
            col_5 = col_5.push(button_cell(button("Delete").on_press(Message::Delete(i))));
        }

        let rows = row![col_1, col_2, col_3, col_4];

        let mut add = button("Add");
        if self.tx.amount.is_some() {
            add = add.on_press(Message::SubmitTx);
        }
        let input = row![
            self.amount_view(),
            text(" "),
            text_input("Date YYYY-MM-DD (empty for today)", &self.tx.date)
                .on_input(Message::ChangeDate),
            text(" "),
            text_input("Comment", &self.tx.comment).on_input(Message::ChangeComment),
            text(" "),
            add,
            text(" ".repeat(EDGE_PADDING)),
        ];

        let mut year = match &self.filter_date_year {
            Some(year) => text_input("Year", &year.to_string()),
            None => text_input("Year", ""),
        };
        year = year.on_input(Message::ChangeFilterDateYear);
        let mut month = match &self.filter_date_month {
            Some(month) => text_input("Month", &month.to_string()),
            None => text_input("Month", ""),
        };
        month = month.on_input(Message::ChangeFilterDateMonth);

        let filter_date = row![
            year,
            text(" "),
            month,
            text(" "),
            button("Filter").on_press(Message::SubmitFilterDate),
            text(" ".repeat(EDGE_PADDING)),
        ];

        let col = column![
            text_cell(format!("{} {}", &self.name, &self.currency)),
            chart,
            rows,
            text_cell("total: "),
            number_cell(total),
            input.padding(PADDING),
            filter_date.padding(PADDING),
            row![
                button("Back").on_press(Message::Back),
                button("Exit").on_press(Message::Exit),
            ],
            text(&self.error_str).size(TEXT_SIZE),
        ];

        Scrollable::new(col)
    }

    pub fn list_monthly(&self) -> Scrollable<Message> {
        let mut col_1 = column![text_cell(" Amount ")].align_items(iced::Alignment::End);
        let mut col_2 = column![text_cell(" Comment ")];
        let mut col_3 = column![text_cell("")];

        let mut total = dec!(0);
        for (i, tx) in self.monthly.iter().enumerate() {
            total += tx.amount;
            col_1 = col_1.push(number_cell(tx.amount));
            col_2 = col_2.push(text_cell(&tx.comment));
            col_3 = col_3.push(button_cell(button("Delete").on_press(Message::Delete(i))));
        }

        let rows = row![col_1, col_2, col_3];

        let mut amount = match &self.tx_monthly.amount {
            Some(amount) => text_input("Amount", &amount.to_string()),
            None => text_input("Amount", ""),
        };
        amount = amount.on_input(Message::ChangeTx);
        let mut add = button("Add");
        if self.tx_monthly.amount.is_some() {
            add = add.on_press(Message::SubmitTx);
        }
        let input = row![
            amount,
            text(" "),
            text_input("Comment", &self.tx_monthly.comment).on_input(Message::ChangeComment),
            text(" "),
            add,
            text(" ".repeat(EDGE_PADDING)),
        ];

        let col = column![
            text_cell(format!("{} {}", &self.name, &self.currency)),
            rows,
            text_cell("total: "),
            number_cell(total),
            input.padding(PADDING),
            row![
                button("Back").on_press(Message::Back),
                button("Exit").on_press(Message::Exit),
            ],
        ];

        Scrollable::new(col)
    }

    pub fn max_balance(&self) -> Option<Decimal> {
        self.data.iter().map(|tx| tx.balance).max()
    }

    pub fn min_balance(&self) -> Option<Decimal> {
        self.data.iter().map(|tx| tx.balance).min()
    }

    pub fn max_date(&self) -> Option<DateTime<Utc>> {
        self.data.iter().map(|tx| tx.date).max()
    }

    pub fn min_date(&self) -> Option<DateTime<Utc>> {
        self.data.iter().map(|tx| tx.date).min()
    }

    pub fn submit_filter_date(&self) -> Option<DateTime<Utc>> {
        let year = match self.filter_date_year {
            Some(year) => year,
            None => return None,
        };
        let month = match self.filter_date_month {
            Some(month) => month,
            None => return None,
        };
        Some(TimeZone::with_ymd_and_hms(&Utc, year, month, 1, 0, 0, 0).unwrap())
    }

    pub fn submit_tx(&self) -> Result<Transaction, String> {
        let amount = self.tx.amount.unwrap();

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
            balance: self.balance() + amount,
            comment: self.tx.comment.clone(),
            date,
        })
    }

    pub fn submit_tx_monthly(&mut self) {
        let tx = take(&mut self.tx_monthly);
        let tx = TransactionMonthly {
            amount: tx.amount.unwrap(),
            comment: tx.comment,
        };
        self.monthly.push(tx);
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
        let mut year = now.year();
        let mut month = now.month() - 1;
        if month == 0 {
            year -= 1;
            month = 12;
        }
        let month_start = Utc.with_ymd_and_hms(year, month, 1, 0, 0, 0).unwrap();
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
