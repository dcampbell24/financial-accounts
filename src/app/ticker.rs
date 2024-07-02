use std::{error::Error, str::FromStr};

use chrono::Utc;
use reqwest::blocking::Client;
use reqwest::Url;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::money::Currency;

const URL_KRAKEN_OHLC: &str = "https://api.kraken.com/0/public/OHLC";

pub struct Ticker {
    http_client: Client,
}

impl Ticker {
    pub fn init() -> Self {
        Ticker {
            http_client: Client::new(),
        }
    }

    pub fn get_bitcoin_ohlc(&self) -> Result<(), Box<dyn Error>> {
        let url = Url::parse_with_params(
            URL_KRAKEN_OHLC,
            &[
                ("pair", "XBTUSD"),
                // A day.
                ("interval", "1440"),
                ("since", &Utc::now().timestamp().to_string()),
            ],
        )?;

        let response = self.http_client.get(url).send()?;
        let text = response.text()?;
        let body: BitCoinResponse = serde_json::from_str(&text)?;

        if body.error.is_empty() {
            let ohlc = Ohlc {
                name: "bitcoin".to_string(),
                currency: Currency::Usd,
                date_time: body.result.bitcoin_usd[0][0].take_u64(),
                open: Decimal::from_str(&body.result.bitcoin_usd[0][1].clone().take_string())?,
                high: Decimal::from_str(&body.result.bitcoin_usd[0][2].clone().take_string())?,
                low: Decimal::from_str(&body.result.bitcoin_usd[0][3].clone().take_string())?,
                close: Decimal::from_str(&body.result.bitcoin_usd[0][4].clone().take_string())?,
                vwap: Decimal::from_str(&body.result.bitcoin_usd[0][5].clone().take_string())?,
                volume: Decimal::from_str(&body.result.bitcoin_usd[0][6].clone().take_string())?,
                count: body.result.bitcoin_usd[0][7].take_u64(),
            };
            println!("{ohlc:#?}");
            Ok(())
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct BitCoinResponse {
    error: Vec<String>,
    result: BitCoinOhlcVec,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct BitCoinOhlcVec {
    #[serde(rename = "XXBTZUSD")]
    bitcoin_usd: Vec<Vec<IntOrString>>,
    last: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum IntOrString {
    U64(u64),
    Str(String),
}

impl IntOrString {
    fn take_u64(&self) -> u64 {
        match self {
            IntOrString::U64(i) => *i,
            IntOrString::Str(_) => panic!("You can only take u64s!"),
        }
    }

    fn take_string(self) -> String {
        match self {
            IntOrString::U64(_) => panic!("You can only take Strings!"),
            IntOrString::Str(s) => s,
        }
    }
}

impl Default for IntOrString {
    fn default() -> Self {
        IntOrString::U64(0)
    }
}

#[derive(Clone, Debug)]
struct Ohlc {
    name: String,
    currency: Currency,
    date_time: u64,
    open: Decimal,
    high: Decimal,
    low: Decimal,
    close: Decimal,
    vwap: Decimal,
    volume: Decimal,
    count: u64,
}
// {"error":[],"result":{"GNOUSD":[[1719878400,"286.27","286.88","284.97","284.97","285.78","4.74983692",10]],"last":1719792000}}
// {"error":[],"result":{"XETHZUSD":[[1719878400,"3438.32","3450.99","3432.20","3444.99","3442.24","357.97391572",651]],"last":1719792000}}
