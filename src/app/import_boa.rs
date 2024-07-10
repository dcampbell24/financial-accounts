use std::{collections::VecDeque, error::Error, fs, path::PathBuf};

use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::Deserialize;

use super::account::transaction::Transaction;

#[derive(Debug, Deserialize)]
struct BoaRecord {
    #[serde(rename = "Date")]
    date: String,
    #[serde(rename = "Description")]
    description: String,
    #[serde(rename = "Amount")]
    amount: String,
    #[serde(rename = "Running Bal.")]
    running_balance: String,
}

pub fn import_boa(file_path: PathBuf) -> Result<VecDeque<Transaction>, Box<dyn Error>> {
    let contents: String = fs::read_to_string(file_path)?
        .lines()
        .skip(6)
        .map(|s| {
            let mut s = s.to_string();
            s.push('\n');
            s
        })
        .collect();

    let mut rdr = csv::Reader::from_reader(contents.as_bytes());
    let mut records = VecDeque::new();
    for (i, result) in rdr.deserialize().enumerate() {
        let mut record: BoaRecord = result?;

        if i == 0 {
            assert!(record.amount.is_empty());
            record.amount = "0".to_string()
        }
        record.date.push_str(" 00:00:00");

        // Fixme: not really UTC.
        let record = Transaction {
            amount: record.amount.replace(',', "").parse::<Decimal>()?,
            balance: record.running_balance.replace(',', "").parse::<Decimal>()?,
            comment: record.description,
            date: NaiveDateTime::parse_from_str(&record.date, "%m/%d/%Y %H:%M:%S")?.and_utc(),
        };
        records.push_back(record);
    }
    Ok(records)
}
