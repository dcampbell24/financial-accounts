pub mod transaction;
pub mod transactions_secondary;

use std::{error::Error, mem::take};

use chrono::{DateTime, Datelike, Months, NaiveDate, TimeZone, Utc};
use iced::{
    widget::{button, column, row, text, text_input, Button, Row, Scrollable, TextInput},
    Length,
};
use plotters_iced::ChartWidget;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use transactions_secondary::Transactions2nd;

use crate::app::{
    account::transaction::{Transaction, TransactionMonthly, TransactionToSubmit},
    Message, EDGE_PADDING, PADDING,
};

use self::transaction::TransactionMonthlyToSubmit;

use super::{button_cell, chart::MyChart, money::Currency, number_cell, text_cell, ROW_SPACING};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Account {
    pub name: String,
    #[serde(skip)]
    pub tx: TransactionToSubmit,
    #[serde(skip)]
    pub tx_monthly: TransactionMonthlyToSubmit,
    #[serde(rename = "transactions")]
    pub txs_1st: Vec<Transaction>,
    #[serde(rename = "transactions_secondary")]
    pub txs_2nd: Option<Transactions2nd>,
    #[serde(rename = "transactions_monthly")]
    pub txs_monthly: Vec<TransactionMonthly>,
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
        let txs_2nd = match currency {
            Currency::Eth | Currency::Gno | Currency::GoldOz => {
                Some(Transactions2nd::new(currency))
            }
            Currency::Usd => None,
        };

        Account {
            name,
            tx: TransactionToSubmit::new(),
            tx_monthly: TransactionMonthlyToSubmit::new(),
            txs_1st: Vec::new(),
            txs_2nd,
            txs_monthly: Vec::new(),
            filter_date: None,
            filter_date_year: None,
            filter_date_month: None,
            error_str: String::new(),
        }
    }

    pub fn balance_1st(&self) -> Decimal {
        match self.txs_1st.last() {
            Some(tx) => tx.balance,
            None => dec!(0),
        }
    }

    pub fn balance_2nd(&self) -> Decimal {
        match self.txs_2nd.as_ref().unwrap().txs.last() {
            Some(tx) => tx.balance,
            None => dec!(0),
        }
    }

    pub fn list_transactions(
        &self,
        txs: &Vec<Transaction>,
        currency: Currency,
        total: Decimal,
        balance: Decimal,
    ) -> Scrollable<Message> {
        let my_chart = MyChart {
            account: self.clone(),
        };
        let chart = ChartWidget::new(my_chart).height(Length::Fixed(400.0));

        let mut col_1 = column![text_cell(" Amount ")].align_items(iced::Alignment::End);
        let mut col_2 = column![text_cell(" Date ")];
        let mut col_3 = column![text_cell(" Balance ")].align_items(iced::Alignment::End);
        let mut col_4 = column![text_cell(" Comment ")];
        let mut col_5 = column![text_cell("")];

        let mut txs = txs;

        let mut filtered_tx = Vec::new();
        if let Some(date) = self.filter_date {
            for tx in txs.iter() {
                if tx.date >= date && tx.date < date.checked_add_months(Months::new(1)).unwrap() {
                    filtered_tx.push(tx.clone())
                }
            }
            txs = &filtered_tx;
        }

        for (i, tx) in txs.iter().enumerate() {
            col_1 = col_1.push(number_cell(tx.amount));
            col_2 = col_2.push(text_cell(tx.date.format("%Y-%m-%d %Z ")));
            col_3 = col_3.push(number_cell(tx.balance));
            col_4 = col_4.push(text_cell(&tx.comment));
            col_5 = col_5.push(button_cell(button("Delete").on_press(Message::Delete(i))));
        }
        let rows = row![col_1, col_2, col_3, col_4, col_5];

        let input = row![
            amount_view(&self.tx.amount),
            balance_view(&self.tx.balance),
            date_view(&self.tx.date),
            comment_view(&self.tx.comment),
            add_view(&self.tx.amount, &self.tx.balance),
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
            month,
            button("Filter").on_press(Message::SubmitFilterDate),
            text(" ".repeat(EDGE_PADDING)),
        ];

        let col = column![
            text_cell(format!("{} {}", &self.name, currency)),
            chart,
            rows,
            row![text_cell("total: "), number_cell(total)],
            row![text_cell("balance: "), number_cell(balance)],
            input.padding(PADDING).spacing(ROW_SPACING),
            filter_date.padding(PADDING).spacing(ROW_SPACING),
            back_exit_view(),
        ];

        Scrollable::new(col)
    }

    pub fn list_monthly(&self) -> Scrollable<Message> {
        let mut col_1 = column![text_cell(" Amount ")].align_items(iced::Alignment::End);
        let mut col_2 = column![text_cell(" Comment ")];
        let mut col_3 = column![text_cell("")];

        for (i, tx) in self.txs_monthly.iter().enumerate() {
            col_1 = col_1.push(number_cell(tx.amount));
            col_2 = col_2.push(text_cell(&tx.comment));
            col_3 = col_3.push(button_cell(button("Delete").on_press(Message::Delete(i))));
        }
        let rows = row![col_1, col_2, col_3];

        let input = row![
            amount_view(&self.tx_monthly.amount),
            comment_view(&self.tx_monthly.comment),
            add_view(&self.tx_monthly.amount, &None),
            text(" ".repeat(EDGE_PADDING)),
        ];

        let col = column![
            text_cell(&self.name),
            rows,
            // Fixme: display a total for the monthly transactions.
            // text_cell("total: "),
            // number_cell(self.total_1st()),
            input.padding(PADDING).spacing(ROW_SPACING),
            back_exit_view(),
        ];

        Scrollable::new(col)
    }

    pub fn max_balance(&self) -> Option<Decimal> {
        self.txs_1st.iter().map(|tx| tx.balance).max()
    }

    pub fn min_balance(&self) -> Option<Decimal> {
        self.txs_1st.iter().map(|tx| tx.balance).min()
    }

    pub fn max_date(&self) -> Option<DateTime<Utc>> {
        self.txs_1st.iter().map(|tx| tx.date).max()
    }

    pub fn min_date(&self) -> Option<DateTime<Utc>> {
        self.txs_1st.iter().map(|tx| tx.date).min()
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

    pub fn submit_balance_1st(&self) -> Result<Transaction, String> {
        let balance = self.tx.balance.unwrap();

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
            amount: balance - self.balance_1st(),
            balance,
            comment: self.tx.comment.clone(),
            date,
        })
    }

    pub fn submit_balance_2nd(&self) -> Result<Transaction, String> {
        let balance = self.tx.balance.unwrap();

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
            amount: balance - self.balance_2nd(),
            balance,
            comment: self.tx.comment.clone(),
            date,
        })
    }

    pub fn submit_ohlc(&self) -> Result<Transaction, Box<dyn Error>> {
        let mut tx = self.txs_2nd.as_ref().unwrap().get_ohlc()?;
        tx.amount = tx.balance - self.balance_1st();
        Ok(tx)
    }

    pub fn submit_tx_1st(&self) -> Result<Transaction, String> {
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
            balance: self.balance_1st() + amount,
            comment: self.tx.comment.clone(),
            date,
        })
    }

    pub fn submit_tx_2nd(&self) -> Result<Transaction, String> {
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
            balance: self.balance_2nd() + amount,
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
        self.txs_monthly.push(tx);
    }

    pub fn total_1st(&self) -> Decimal {
        self.txs_1st.iter().map(|d| d.amount).sum()
    }

    pub fn total_2nd(&self) -> Decimal {
        self.txs_2nd
            .as_ref()
            .unwrap()
            .txs
            .iter()
            .map(|d| d.amount)
            .sum()
    }

    pub fn sum_monthly(&self) -> Decimal {
        self.txs_monthly.iter().map(|d| d.amount).sum()
    }

    pub fn sum_current_month(&self) -> Decimal {
        let now = Utc::now();
        let date = Utc
            .with_ymd_and_hms(now.year(), now.month(), 1, 0, 0, 0)
            .unwrap();
        let mut amount = dec!(0);
        for tx in self.txs_1st.iter() {
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
        for tx in self.txs_1st.iter() {
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
        for tx in self.txs_1st.iter() {
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
        for tx in self.txs_1st.iter() {
            if tx.date >= year_start && tx.date < year_end {
                amount += tx.amount;
            }
        }
        amount
    }
}

fn amount_view(amount: &Option<Decimal>) -> TextInput<Message> {
    let amount = match amount {
        Some(amount) => text_input("Amount", &amount.to_string()),
        None => text_input("Amount", ""),
    };
    amount.on_input(Message::ChangeTx)
}

fn balance_view(balance: &Option<Decimal>) -> TextInput<Message> {
    let balance = match balance {
        Some(balance) => text_input("Balance", &balance.to_string()),
        None => text_input("Balance", ""),
    };
    balance.on_input(Message::ChangeBalance)
}

fn date_view(date: &str) -> TextInput<Message> {
    text_input("Date YYYY-MM-DD (empty for today)", date).on_input(Message::ChangeDate)
}

fn comment_view(comment: &str) -> TextInput<Message> {
    text_input("Comment", comment).on_input(Message::ChangeComment)
}

fn add_view<'a>(amount: &Option<Decimal>, balance: &Option<Decimal>) -> Button<'a, Message> {
    let mut add = button("Add");
    match (amount, balance) {
        (Some(_amount), None) => add = add.on_press(Message::SubmitTx),
        (None, Some(_balance)) => add = add.on_press(Message::SubmitBalance),
        (None, None) | (Some(_), Some(_)) => {}
    }
    add
}

fn back_exit_view<'a>() -> Row<'a, Message> {
    row![
        button("Back").on_press(Message::Back),
        button("Exit").on_press(Message::Exit),
    ]
    .spacing(ROW_SPACING)
}
