use std::mem::take;

use chrono::Utc;
use reqwest::blocking::Client;
use reqwest::Url;
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

    pub fn get_bitcoin_ohlc(&self) {
        let url = Url::parse_with_params(
            URL_KRAKEN_OHLC,
            &[
                ("pair", "XBTUSD"),
                // A day.
                ("interval", "1440"),
                ("since", &Utc::now().timestamp().to_string()),
            ],
        )
        .unwrap();

        let response = self.http_client.get(url).send().unwrap();
        let text = response.text().unwrap();
        let mut body: BitCoinResponse = serde_json::from_str(&text).unwrap();

        if body.error.is_empty() {
            let ohlc = Ohlc {
                name: "bitcoin".to_string(),
                currency: Currency::Usd,
                date_time: body.result.bitcoin_usd[0][0].take_u64(),
                open: take(&mut body.result.bitcoin_usd[0][1]).take_string(),
                high: take(&mut body.result.bitcoin_usd[0][2]).take_string(),
                low: take(&mut body.result.bitcoin_usd[0][3]).take_string(),
                close: take(&mut body.result.bitcoin_usd[0][4]).take_string(),
                vwap: take(&mut body.result.bitcoin_usd[0][5]).take_string(),
                volume: take(&mut body.result.bitcoin_usd[0][6]).take_string(),
                count: body.result.bitcoin_usd[0][7].take_u64(),
            };
            println!("{ohlc:#?}");
        }
    }
}

#[derive(Serialize, Debug, Deserialize)]
struct BitCoinResponse {
    error: Vec<String>,
    result: BitCoinOhlcVec,
}

#[derive(Serialize, Debug, Deserialize)]
struct BitCoinOhlcVec {
    #[serde(rename = "XXBTZUSD")]
    bitcoin_usd: Vec<Vec<IntOrString>>,
    last: u64,
}

#[derive(Serialize, Debug, Deserialize)]
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

#[derive(Debug)]
struct Ohlc {
    name: String,
    currency: Currency,
    date_time: u64,
    open: String,
    high: String,
    low: String,
    close: String,
    vwap: String,
    volume: String,
    count: u64,
}
// {"error":[],"result":{"GNOUSD":[[1719878400,"286.27","286.88","284.97","284.97","285.78","4.74983692",10]],"last":1719792000}}
// {"error":[],"result":{"XETHZUSD":[[1719878400,"3438.32","3450.99","3432.20","3444.99","3442.24","357.97391572",651]],"last":1719792000}}
