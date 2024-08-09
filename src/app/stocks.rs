use std::{
    env,
    fmt::{self, Display},
    fs,
};

use anyhow::Context;
use chrono::{serde::ts_milliseconds, DateTime, Utc};
use reqwest::blocking::Client;
use reqwest::Url;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::account::transactions::Price;

const LOCATION_ACCESS_TOKEN: &str = "./polygon.io.txt";

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Serialize)]
pub struct Stock {
    // currency: USD
    pub description: String,
    pub symbol: String,
}

impl Price for Stock {
    fn get_price(&self, client: &Client) -> anyhow::Result<Decimal> {
        let pwd = env::current_dir()?;
        let access_token = fs::read_to_string(LOCATION_ACCESS_TOKEN).context(format!(
            "pwd: {pwd:?} location: {LOCATION_ACCESS_TOKEN:?} doesn't exist"
        ))?;
        let access_token = access_token.trim();

        let url = format!(
            "https://api.polygon.io/v2/aggs/ticker/{}/prev",
            &self.symbol
        );
        let url = Url::parse(&url)?;

        let response = client
            .get(url)
            .header("Authorization", access_token)
            .send()?;
        // Fixme: What if what you requested wasn't found?
        let string = response.text()?;
        let previous_days_stock_price: StockResult =
            serde_json::from_str(&string).context("You made too many requests too quickly!")?;
        Ok(previous_days_stock_price.results[0].close)
    }
}

impl fmt::Display for Stock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StockResult {
    pub ticker: String,
    #[serde(rename = "queryCount")]
    pub query_count: u64,
    #[serde(rename = "resultsCount")]
    pub results_count: u64,
    pub adjusted: bool,
    pub results: Vec<StockPrice>,
    pub status: String,
    pub request_id: String,
    pub count: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StockPrice {
    #[serde(rename = "T")]
    pub symbol: String,
    #[serde(rename = "v")]
    pub volume: Decimal,
    #[serde(rename = "vw")]
    pub volume_weighted: Decimal,
    #[serde(rename = "o")]
    pub open: Decimal,
    #[serde(rename = "c")]
    pub close: Decimal,
    #[serde(rename = "h")]
    pub high: Decimal,
    #[serde(rename = "l")]
    pub low: Decimal,
    #[serde(rename = "t", with = "ts_milliseconds")]
    pub timestamp: DateTime<Utc>,
    #[serde(rename = "n")]
    pub number: u64,
}

impl Display for StockPrice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        writeln!(f, "symbol: {}", self.symbol)?;
        writeln!(f, "volume: {}", self.volume)?;
        writeln!(f, "volume_weighted: {}", self.volume_weighted)?;
        writeln!(f, "open: {}", self.open)?;
        writeln!(f, "close: {}", self.close)?;
        writeln!(f, "high: {}", self.high)?;
        writeln!(f, "low: {}", self.low)?;
        writeln!(f, "timestamp: {}", self.timestamp)?;
        writeln!(f, "number: {}", self.number)
    }
}
