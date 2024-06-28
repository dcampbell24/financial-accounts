use std::{error::Error, fs};

use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct BoaRecord {
    date: String,
    description: String,
    amount: Decimal,
    running_balance: Decimal,
}

#[derive(Debug, Deserialize)]
struct BoaRecordImport {
    #[serde(rename = "Date")]
    date: String,
    #[serde(rename = "Description")]
    description: String,
    #[serde(rename = "Amount")]
    amount: String,
    #[serde(rename = "Running Bal.")]
    running_balance: String,
}

pub fn import_boa() -> Result<Vec<BoaRecord>, Box<dyn Error>> {
    let contents: String = fs::read_to_string("/home/david/Documents/boa/2024-06-17.csv")?
        .lines()
        .skip(6)
        .map(|s| {
            let mut s = s.to_string();
            s.push('\n');
            s
        })
        .collect();

    let mut rdr = csv::Reader::from_reader(contents.as_bytes());
    let mut records = Vec::new();
    for result in rdr.deserialize() {
        let record: BoaRecordImport = result?;

        if record.amount.is_empty() {
            continue;
        }

        let record = BoaRecord {
            date: record.date,
            description: record.description,
            amount: record.amount.replace(',', "").parse::<Decimal>()?,
            running_balance: record.running_balance.replace(',', "").parse::<Decimal>()?,
        };
        records.push(record);
    }
    Ok(records)
}
