use std::{error::Error, fmt::Display, str::FromStr};

use chrono::{DateTime, Utc};
use reqwest::Client;
use reqwest::Url;
use rust_decimal::Decimal;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

use super::account::transactions::Price;
use super::money::Fiat;

const URL_KRAKEN_OHLC: &str = "https://api.kraken.com/0/public/OHLC";

#[derive(Clone, Debug)]
pub struct Ohlc {
    pub name: String,
    pub date_time: DateTime<Utc>,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume_weighted_average_price: Decimal,
    pub volume: Decimal,
    pub count: i64,
}

impl Ohlc {
    pub fn from_value(value: &Value) -> anyhow::Result<Self> {
        let mut errors = Vec::new();
        for error in value["error"].as_array().unwrap() {
            errors.push(error.as_str().unwrap().to_string());
        }

        if !errors.is_empty() {
            return Err(CryptoErrors { errors })?;
        }

        let mut result = &value["result"];
        let name = result.as_object().unwrap().keys().next().unwrap().clone();
        result = &result[&name][0];

        Ok(Self {
            date_time: DateTime::from_timestamp(result[0].as_i64().unwrap(), 0).unwrap(),
            open: Decimal::from_str(result[1].as_str().unwrap()).unwrap(),
            high: Decimal::from_str(result[2].as_str().unwrap()).unwrap(),
            low: Decimal::from_str(result[3].as_str().unwrap()).unwrap(),
            close: Decimal::from_str(result[4].as_str().unwrap()).unwrap(),
            volume_weighted_average_price: Decimal::from_str(result[5].as_str().unwrap()).unwrap(),
            volume: Decimal::from_str(result[6].as_str().unwrap()).unwrap(),
            count: result[7].as_i64().unwrap(),

            name,
        })
    }
}

impl Display for Ohlc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        writeln!(f, "name: {}", self.name)?;
        writeln!(f, "date_time: {}", self.date_time)?;
        writeln!(f, "open: {}", self.open)?;
        writeln!(f, "high: {}", self.high)?;
        writeln!(f, "low: {}", self.low)?;
        writeln!(f, "close: {}", self.close)?;
        writeln!(
            f,
            "volume_weighted_average_price: {}",
            self.volume_weighted_average_price
        )?;
        writeln!(f, "volume: {}", self.volume)?;
        writeln!(f, "count: {}", self.count)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Crypto {
    pub currency: Fiat,
    pub description: String,
    pub symbol: String,
}

impl Price for Crypto {
    async fn get_price(&self, client: &Client) -> anyhow::Result<Decimal> {
        let pair = format!("{}{}", self.symbol, self.currency.symbol());
        let url = Url::parse_with_params(
            URL_KRAKEN_OHLC,
            &[
                ("pair", pair.as_str()),
                // A day.
                ("interval", "1440"),
                ("since", &Utc::now().timestamp().to_string()),
            ],
        )?;

        let response = client.get(url).send().await?;
        let string = response.text().await?;
        let value: Value = serde_json::from_str(&string)?;
        let crypto = Ohlc::from_value(&value)?;
        Ok(crypto.close)
    }
}

impl Display for Crypto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        writeln!(f, "{} in {}", self.description, self.currency)
    }
}

#[derive(Debug)]
struct CryptoErrors {
    errors: Vec<String>,
}

impl Display for CryptoErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        writeln!(f, "{:#?}", self.errors)
    }
}

impl Error for CryptoErrors {}
