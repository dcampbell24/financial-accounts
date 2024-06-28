
use std::error::Error;

use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct BoaRecord {
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


pub fn import_boa() -> Result<(), Box<dyn Error>> {

    let mut rdr = csv::Reader::from_path("/home/david/Documents/boa/2024-06-17.csv")?;
    for result in rdr.deserialize() {

        let record: BoaRecordImport = result?;
        println!("{:?}", record);

        if record.amount.is_empty() {
            continue;
        }

        let record =  BoaRecord {
            date: record.date,
            description: record.description,
            amount: record.amount.replace(',', "").parse::<Decimal>()?,
            running_balance: record.running_balance.replace(',', "").parse::<Decimal>()?,
        };
        println!("{:?}", record);
    }
    Ok(())
}

