use std::path::PathBuf;

use chrono::NaiveDateTime;
use regex::Regex;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::Deserialize;

use super::{
    account::{transaction::Transaction, transactions::Transactions},
    money::Fiat,
};

#[derive(Debug, Deserialize)]
struct BoaRecord {
    #[serde(rename = "Status")]
    _status: String,
    #[serde(rename = "Date")]
    date: String,
    #[serde(rename = "Original Description")]
    original_description: String,
    #[serde(rename = "Split Type")]
    _split_type: String,
    #[serde(rename = "Category")]
    _category: String,
    #[serde(rename = "Currency")]
    _currency: String,
    #[serde(rename = "Amount")]
    amount: String,
    #[serde(rename = "User Description")]
    _user_description: String,
    #[serde(rename = "Memo")]
    _memo: String,
    #[serde(rename = "Classification")]
    _classification: String,
    #[serde(rename = "Account Name")]
    account_name: String,
    #[serde(rename = "Simple Description")]
    _simple_description: String,
}

pub fn import_boa(file_path: PathBuf) -> anyhow::Result<Transactions<Fiat>> {
    let mut records = Vec::new();

    for boa_record in csv::Reader::from_path(file_path)?.deserialize() {
        let mut boa_record: BoaRecord = boa_record?;
        // We don't get the time of day, so can't tell what day it really is in UTC.
        boa_record.date.push_str(" 00:00:00");

        let name_formal = Regex::new(r" - .+ - .+")?;
        let name = Regex::replace(&name_formal, &boa_record.account_name, "").into_owned();
        let white_space = Regex::new(r"\s+")?;
        let description =
            Regex::replace_all(&white_space, &boa_record.original_description, " ").into_owned();
        let comment = format!("{name}: {description}");

        let record = Transaction {
            amount: boa_record.amount.replace(',', "").parse::<Decimal>()?,
            balance: dec!(0),
            comment,
            date: NaiveDateTime::parse_from_str(&boa_record.date, "%m/%d/%Y %H:%M:%S")?.and_utc(),
        };
        records.push(record);
    }

    let mut txs = Transactions::new(Fiat::Usd);
    txs.txs = records;
    Ok(txs)
}
