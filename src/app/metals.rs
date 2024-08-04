use std::{env, fmt::Display, fs};

use anyhow::Context;
use chrono::{serde::ts_seconds, DateTime, Utc};
use reqwest::blocking::Client;
use reqwest::Url;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::money::Metal;

const _TESTING_RESPONSE: &str = r#"{
    "timestamp":1719978277,
    "metal":"XAU",
    "currency":"USD",
    "exchange":"FOREXCOM",
    "symbol":"FOREXCOM:XAUUSD",
    "prev_close_price":2329.645,
    "open_price":2329.645,
    "low_price":2326.925,
    "high_price":2332.235,
    "open_time":1719964800,
    "price":2330.825,
    "ch":1.18,
    "chp":0.05,
    "ask":2331.14,
    "bid":2330.52,
    "price_gram_24k":74.9378,
    "price_gram_22k":68.693,
    "price_gram_21k":65.5705,
    "price_gram_20k":62.4481,
    "price_gram_18k":56.2033,
    "price_gram_16k":49.9585,
    "price_gram_14k":43.7137,
    "price_gram_10k":31.2241
}"#;

const LOCATION_ACCESS_TOKEN: &str = "./goldapi.io.txt";

pub fn get_price_metal(client: &Client, metal: &Metal) -> anyhow::Result<Price> {
    let pwd = env::current_dir()?;
    let access_token = fs::read_to_string(LOCATION_ACCESS_TOKEN).context(format!(
        "pwd: {pwd:?} location: {LOCATION_ACCESS_TOKEN:?} doesn't exist"
    ))?;
    let access_token = access_token.trim();

    let url = Url::parse(&format!(
        "https://www.goldapi.io/api/{}/{}",
        metal.symbol,
        metal.currency.symbol()
    ))?;
    let response = client
        .get(url)
        .header("x-access-token", access_token)
        .send()?;
    let string = response.text()?;
    // let string = _TESTING_RESPONSE;
    let metals: Price = serde_json::from_str(&string)?;
    Ok(metals)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Price {
    #[serde(with = "ts_seconds")]
    pub timestamp: DateTime<Utc>,
    pub metal: String,
    pub currency: String,
    pub exchange: String,
    pub symbol: String,
    pub prev_close_price: Decimal,
    pub open_price: Decimal,
    pub low_price: Decimal,
    pub high_price: Decimal,
    #[serde(with = "ts_seconds")]
    pub open_time: DateTime<Utc>,
    pub price: Decimal,
    pub ch: Decimal,
    pub chp: Decimal,
    pub ask: Decimal,
    pub bid: Decimal,
    pub price_gram_24k: Decimal,
    pub price_gram_22k: Decimal,
    pub price_gram_21k: Decimal,
    pub price_gram_20k: Decimal,
    pub price_gram_18k: Decimal,
    pub price_gram_16k: Decimal,
    pub price_gram_14k: Decimal,
    pub price_gram_10k: Decimal,
}

impl Display for Price {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        writeln!(f, "timestamp: {}", self.timestamp)?;
        writeln!(f, "metal: {}", self.metal)?;
        writeln!(f, "currency: {}", self.currency)?;
        writeln!(f, "exchange: {}", self.exchange)?;
        writeln!(f, "symbol: {}", self.symbol)?;
        writeln!(f, "prev_close_price: {}", self.prev_close_price)?;
        writeln!(f, "open_price: {}", self.open_price)?;
        writeln!(f, "low_price: {}", self.low_price)?;
        writeln!(f, "high_price: {}", self.high_price)?;
        writeln!(f, "open_time: {}", self.open_time)?;
        writeln!(f, "price: {}", self.price)?;
        writeln!(f, "ch: {}", self.ch)?;
        writeln!(f, "chp: {}", self.chp)?;
        writeln!(f, "ask: {}", self.ask)?;
        writeln!(f, "bid: {}", self.bid)?;
        writeln!(f, "price_per_gram_24k: {}", self.price_gram_24k)?;
        writeln!(f, "price_per_gram_22k: {}", self.price_gram_22k)?;
        writeln!(f, "price_per_gram_21k: {}", self.price_gram_21k)?;
        writeln!(f, "price_per_gram_20k: {}", self.price_gram_20k)?;
        writeln!(f, "price_per_gram_18k: {}", self.price_gram_18k)?;
        writeln!(f, "price_per_gram_16k: {}", self.price_gram_16k)?;
        writeln!(f, "price_per_gram_14k: {}", self.price_gram_14k)?;
        writeln!(f, "price_per_gram_10k: {}", self.price_gram_10k)
    }
}
