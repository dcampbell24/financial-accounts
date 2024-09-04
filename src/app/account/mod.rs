pub mod transaction;
pub mod transactions;

use std::{error::Error, fmt::Display, path::PathBuf, string::ToString};

use chrono::{DateTime, NaiveDate, ParseError, TimeDelta, TimeZone, Utc};
use iced::{
    widget::{button, column, row, text, text_input, Button, Row, Scrollable, TextInput},
    Length,
};
use plotters_iced::ChartWidget;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use transactions::{PriceAsTransaction, Transactions};

use crate::app::{self, account::transaction::Transaction, EDGE_PADDING, PADDING};

use super::{
    button_cell,
    chart::Chart,
    import_boa::import_boa,
    money::{Currency, Fiat},
    number_cell,
    screen::Screen,
    set_amount, some_or_empty, text_cell, Duration, ROW_SPACING,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Account {
    #[serde(skip)]
    pub check_box: bool,
    pub name: String,
    #[serde(skip)]
    pub tx: transaction::ToSubmit,
    #[serde(skip)]
    pub tx_monthly: transaction::MonthlyToSubmit,
    #[serde(rename = "transactions")]
    pub txs_1st: Transactions<Fiat>,
    #[serde(rename = "transactions_secondary")]
    pub txs_2nd: Option<Transactions<Currency>>,
    #[serde(rename = "transactions_monthly")]
    pub txs_monthly: Vec<transaction::Monthly>,
    #[serde(skip)]
    pub filter_date: Option<DateTime<Utc>>,
    #[serde(skip)]
    pub filter_date_year: Option<i32>,
    #[serde(skip)]
    pub filter_date_month: Option<u32>,
    #[serde(skip)]
    pub error: Option<anyhow::Error>,
}

impl Account {
    pub fn new(name: String, currency: Currency) -> Self {
        let (txs_1st, txs_2nd) = match &currency {
            Currency::StockPlus(_) => (
                Transactions::new(Fiat::Usd),
                Some(Transactions::new(currency)),
            ),
            Currency::Crypto(crypto) => (
                Transactions::new(crypto.currency.clone()),
                Some(Transactions::new(currency)),
            ),
            Currency::Metal(metal) => (
                Transactions::new(metal.currency.clone()),
                Some(Transactions::new(currency)),
            ),
            Currency::Fiat(currency) => (Transactions::new(currency.clone()), None),
        };

        Self {
            check_box: false,
            name,
            tx: transaction::ToSubmit::new(),
            tx_monthly: transaction::MonthlyToSubmit::new(),
            txs_1st,
            txs_2nd,
            txs_monthly: Vec::new(),
            filter_date: None,
            filter_date_year: None,
            filter_date_month: None,
            error: None,
        }
    }

    pub fn balance_1st(&self) -> Decimal {
        self.txs_1st.balance()
    }

    pub fn balance_2nd(&self) -> Option<Decimal> {
        self.txs_2nd
            .as_ref()
            .map(transactions::Transactions::balance)
    }

    fn clear_date(&mut self) {
        self.filter_date_year = None;
        self.filter_date_month = None;
        self.filter_date = None;
    }

    pub fn import_boa(&mut self, file_path: PathBuf) -> anyhow::Result<()> {
        let mut boa = import_boa(file_path)?;
        boa.remove_duplicates(&self.txs_1st);
        boa.sort();

        if let Some(tx_1st) = self.txs_1st.txs.last() {
            if let Some(tx_add) = boa.txs.first() {
                if tx_1st.date > tx_add.date {
                    return Err(anyhow::Error::msg("The starting date of the first transaction you want to add is not greater than the end date of the old transactions."));
                }
            } else {
                return Ok(());
            }
        }

        let mut balance = self.txs_1st.balance();
        for mut tx in boa.txs {
            balance += tx.amount;
            tx.balance = balance;
            self.txs_1st.txs.push(tx);
        }
        self.tx = transaction::ToSubmit::new();
        Ok(())
    }

    pub fn list_transactions<'a, T: 'a + Clone + Display>(
        &'a self,
        mut txs_struct: Transactions<T>,
        total: Decimal,
        balance: Decimal,
    ) -> Scrollable<app::Message> {
        txs_struct.filter_month(self.filter_date);

        let chart = Chart {
            txs: txs_struct.clone(),
            duration: Duration::All,
        };
        let chart: ChartWidget<'a, _, _, _, _> =
            ChartWidget::new(chart).height(Length::Fixed(400.0));

        let mut col_1 = column![text_cell(" Amount ")].align_items(iced::Alignment::End);
        let mut col_2 = column![text_cell(" Date ")];
        let mut col_3 = column![text_cell(" Balance ")].align_items(iced::Alignment::End);
        let mut col_4 = column![text_cell(" Comment ")];
        let mut col_5 = column![text_cell("")];

        for (i, tx) in txs_struct.txs.iter().enumerate() {
            col_1 = col_1.push(number_cell(tx.amount));
            col_2 = col_2.push(text_cell(tx.date.format("%Y-%m-%d")));
            col_3 = col_3.push(number_cell(tx.balance));
            col_4 = col_4.push(text_cell(&tx.comment));
            col_5 = col_5.push(button_cell(
                button("Delete").on_press(app::Message::Delete(i)),
            ));
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

        let year = text_input("Year", &some_or_empty(&self.filter_date_year))
            .on_input(|string| app::Message::Account(Message::ChangeFilterDateYear(string)));
        let month = text_input("Month", &some_or_empty(&self.filter_date_month))
            .on_input(|string| app::Message::Account(Message::ChangeFilterDateMonth(string)));
        let mut filter_button = button("Filter");
        if self.submit_filter_date().is_some() {
            filter_button =
                filter_button.on_press(app::Message::Account(Message::SubmitFilterDate));
        }
        let clear_button = button("Clear").on_press(app::Message::Account(Message::ClearDate));
        let filter_date = row![
            year,
            month,
            filter_button,
            clear_button,
            text(" ".repeat(EDGE_PADDING)),
        ];
        let error = self
            .error
            .as_ref()
            .map_or_else(|| row![], |error| row![text_cell(error)]);

        let col = column![
            text_cell(txs_struct.currency.to_string()),
            chart,
            rows,
            row![text_cell("total: "), number_cell(total)],
            row![text_cell("balance: "), number_cell(balance)],
            input.padding(PADDING).spacing(ROW_SPACING),
            filter_date.padding(PADDING).spacing(ROW_SPACING),
            error,
            back_exit_view(),
        ];

        Scrollable::new(col)
    }

    pub fn list_monthly(&self) -> Scrollable<app::Message> {
        let mut col_1 = column![text_cell(" Amount ")].align_items(iced::Alignment::End);
        let mut col_2 = column![text_cell(" Comment ")];
        let mut col_3 = column![text_cell("")];

        let mut total = dec!(0);
        for (i, tx) in self.txs_monthly.iter().enumerate() {
            total += tx.amount;
            col_1 = col_1.push(number_cell(tx.amount));
            col_2 = col_2.push(text_cell(&tx.comment));
            col_3 = col_3.push(button_cell(
                button("Delete").on_press(app::Message::Delete(i)),
            ));
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
            row![text_cell("total: "), number_cell(total)],
            input.padding(PADDING).spacing(ROW_SPACING),
            back_exit_view(),
        ];

        Scrollable::new(col)
    }

    fn parse_date(&self) -> Result<DateTime<Utc>, ParseDateError> {
        if self.tx.date.is_empty() {
            Ok(Utc::now())
        } else {
            match NaiveDate::parse_from_str(&self.tx.date, "%Y-%m-%d") {
                Ok(naive_date) => Ok(naive_date.and_hms_opt(0, 0, 0).unwrap().and_utc()),
                Err(error) => Err(ParseDateError { error }),
            }
        }
    }

    fn submit_filter_date(&self) -> Option<DateTime<Utc>> {
        let year = self.filter_date_year?;
        let month = self.filter_date_month?;

        Some(TimeZone::with_ymd_and_hms(&Utc, year, month, 1, 0, 0, 0).unwrap())
    }

    fn submit_balance_1st(&mut self) -> anyhow::Result<Transaction> {
        let balance = self.tx.balance.unwrap();
        let date = self.parse_date()?;

        let tx = Transaction {
            amount: dec!(0),
            balance,
            comment: self.tx.submit_commit(),
            date,
        };
        Ok(self.txs_1st.balance_to_amount(tx))
    }

    fn submit_balance_2nd(&mut self) -> anyhow::Result<Transaction> {
        let balance = self.tx.balance.unwrap();
        let date = self.parse_date()?;

        let tx = Transaction {
            amount: dec!(0),
            balance,
            comment: self.tx.submit_commit(),
            date,
        };
        Ok(self.txs_2nd.as_mut().unwrap().balance_to_amount(tx))
    }

    pub async fn submit_price_as_transaction(&self) -> anyhow::Result<Transaction> {
        let mut tx = self
            .txs_2nd
            .as_ref()
            .unwrap()
            .get_price_as_transaction()
            .await?;
        tx.amount = tx.balance - self.balance_1st();
        Ok(tx)
    }

    fn submit_tx_1st(&self) -> anyhow::Result<Transaction> {
        let amount = self.tx.amount.unwrap();
        let date = self.parse_date()?;
        self.txs_1st.date_most_recent(&date)?;

        Ok(Transaction {
            amount,
            balance: self.balance_1st() + amount,
            comment: self.tx.submit_commit(),
            date,
        })
    }

    fn submit_tx_2nd(&self) -> anyhow::Result<Transaction> {
        let amount = self.tx.amount.unwrap();
        let date = self.parse_date()?;
        self.txs_2nd.as_ref().unwrap().date_most_recent(&date)?;

        Ok(Transaction {
            amount,
            balance: self.balance_2nd().unwrap() + amount,
            comment: self.tx.submit_commit(),
            date,
        })
    }

    fn submit_tx_monthly(&mut self) {
        let tx = transaction::Monthly {
            amount: self.tx.amount.unwrap(),
            comment: self.tx.submit_commit(),
        };
        self.txs_monthly.push(tx);
    }

    pub fn total_1st(&self) -> Decimal {
        self.txs_1st.total()
    }

    pub fn total_2nd(&self) -> Decimal {
        self.txs_2nd.as_ref().unwrap().total()
    }

    pub fn sum_monthly(&self) -> Decimal {
        self.txs_monthly.iter().map(|d| d.amount).sum()
    }

    pub fn sum_last_week(&self) -> Decimal {
        let last_week = Utc::now() - TimeDelta::weeks(1);
        let mut amount = dec!(0);

        for tx in &self.txs_1st.txs {
            if tx.date >= last_week {
                amount += tx.amount;
            }
        }
        amount
    }

    pub fn sum_last_month(&self) -> Decimal {
        let last_month = Utc::now() - TimeDelta::days(30);
        let mut amount = dec!(0);

        for tx in &self.txs_1st.txs {
            if tx.date >= last_month {
                amount += tx.amount;
            }
        }
        amount
    }

    pub fn sum_last_year(&self) -> Decimal {
        let last_year = Utc::now() - TimeDelta::days(365);
        let mut amount = dec!(0);

        for tx in &self.txs_1st.txs {
            if tx.date >= last_year {
                amount += tx.amount;
            }
        }
        amount
    }

    fn display_error(&mut self, result: anyhow::Result<Transaction>) -> Option<Transaction> {
        match result {
            Ok(tx) => Some(tx),
            Err(error) => {
                self.error = Some(error);
                None
            }
        }
    }

    pub fn update(&mut self, screen: &Screen, message: Message) -> bool {
        let list_monthly = list_monthly(screen);
        self.error = None;

        match message {
            Message::ChangeBalance(balance) => {
                set_amount(&mut self.tx.balance, &balance);
            }
            Message::ChangeComment(comment) => {
                if list_monthly {
                    self.tx_monthly.comment = comment;
                } else {
                    self.tx.comment = comment;
                }
            }
            Message::ChangeDate(date) => self.tx.date = date,
            Message::ChangeFilterDateMonth(date) => {
                if date.is_empty() {
                    self.filter_date_month = None;
                }
                if let Ok(date) = date.parse() {
                    if (1..13).contains(&date) {
                        self.filter_date_month = Some(date);
                    }
                }
            }
            Message::ChangeFilterDateYear(date) => {
                if date.is_empty() {
                    self.filter_date_year = None;
                }
                if let Ok(date) = date.parse() {
                    if (0..3_000).contains(&date) {
                        self.filter_date_year = Some(date);
                    }
                }
            }
            Message::ChangeTx(tx) => {
                if list_monthly {
                    set_amount(&mut self.tx_monthly.amount, &tx);
                } else {
                    set_amount(&mut self.tx.amount, &tx);
                }
            }
            Message::ClearDate => self.clear_date(),
            Message::SubmitBalance => match screen {
                Screen::Account(_) => {
                    let result = self.submit_balance_1st();
                    if let Some(tx) = self.display_error(result) {
                        self.txs_1st.txs.push(tx);
                        self.txs_1st.sort();
                        self.tx = transaction::ToSubmit::new();
                        return true;
                    }
                }
                Screen::AccountSecondary(_) => {
                    let result = self.submit_balance_2nd();
                    if let Some(tx) = self.display_error(result) {
                        self.txs_2nd.as_mut().unwrap().txs.push(tx);
                        self.txs_2nd.as_mut().unwrap().sort();
                        self.tx = transaction::ToSubmit::new();
                        return true;
                    }
                }
                Screen::Accounts | Screen::Configuration | Screen::Monthly(_) => {
                    panic!("You can't submit a balance here!");
                }
            },
            Message::SubmitFilterDate => {
                self.filter_date = self.submit_filter_date();
            }
            Message::SubmitTx => match screen {
                Screen::Account(_) => {
                    if let Some(tx) = self.display_error(self.submit_tx_1st()) {
                        self.txs_1st.txs.push(tx);
                        self.txs_1st.sort();
                        self.tx = transaction::ToSubmit::new();
                        return true;
                    }
                }
                Screen::AccountSecondary(_) => {
                    if let Some(tx) = self.display_error(self.submit_tx_2nd()) {
                        self.txs_2nd.as_mut().unwrap().txs.push(tx);
                        self.txs_2nd.as_mut().unwrap().sort();
                        self.tx = transaction::ToSubmit::new();
                        return true;
                    }
                }
                Screen::Monthly(_) => self.submit_tx_monthly(),
                Screen::Accounts | Screen::Configuration => {
                    panic!("You can't submit a transaction here!")
                }
            },
        }
        false
    }
}

fn amount_view(amount: &Option<Decimal>) -> TextInput<app::Message> {
    text_input("Amount", &some_or_empty(amount))
        .on_input(|string| app::Message::Account(Message::ChangeTx(string)))
}

fn balance_view(balance: &Option<Decimal>) -> TextInput<app::Message> {
    text_input("Balance", &some_or_empty(balance))
        .on_input(|string| app::Message::Account(Message::ChangeBalance(string)))
}

fn date_view(date: &str) -> TextInput<app::Message> {
    text_input("Date YYYY-MM-DD (empty for today)", date)
        .on_input(|string| app::Message::Account(Message::ChangeDate(string)))
}

fn comment_view(comment: &str) -> TextInput<app::Message> {
    text_input("Comment", comment)
        .on_input(|string| app::Message::Account(Message::ChangeComment(string)))
        .on_paste(|string| app::Message::Account(Message::ChangeComment(string)))
}

fn add_view<'a>(amount: &Option<Decimal>, balance: &Option<Decimal>) -> Button<'a, app::Message> {
    let mut add = button("Add");
    match (amount, balance) {
        (Some(_amount), None) => add = add.on_press(app::Message::Account(Message::SubmitTx)),
        (None, Some(_balance)) => {
            add = add.on_press(app::Message::Account(Message::SubmitBalance));
        }
        (None, None) | (Some(_), Some(_)) => {}
    }
    add
}

fn back_exit_view<'a>() -> Row<'a, app::Message> {
    row![
        button("Back").on_press(app::Message::Back),
        button("Exit").on_press(app::Message::Exit),
    ]
    .spacing(ROW_SPACING)
}

const fn list_monthly(screen: &Screen) -> bool {
    match screen {
        Screen::Accounts
        | Screen::Account(_)
        | Screen::AccountSecondary(_)
        | Screen::Configuration => false,
        Screen::Monthly(_) => true,
    }
}

#[derive(Clone, Debug)]
pub struct ParseDateError {
    error: ParseError,
}

impl Display for ParseDateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        writeln!(f, "Parse Date error: {}", self.error)
    }
}

impl Error for ParseDateError {}

#[derive(Clone, Debug)]
pub enum Message {
    ChangeBalance(String),
    ChangeComment(String),
    ChangeDate(String),
    ChangeFilterDateMonth(String),
    ChangeFilterDateYear(String),
    ChangeTx(String),
    ClearDate,
    SubmitBalance,
    SubmitFilterDate,
    SubmitTx,
}
