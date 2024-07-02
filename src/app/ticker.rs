use std::{error::Error, fmt::Display, str::FromStr};

use chrono::{DateTime, Utc};
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

    pub fn get_ohlc_untyped(&self, name: &str) -> Result<String, Box<dyn Error>> {
        let url = Url::parse_with_params(
            URL_KRAKEN_OHLC,
            &[
                ("pair", name),
                // A day.
                ("interval", "1440"),
                ("since", &Utc::now().timestamp().to_string()),
            ],
        )?;

        let response = self.http_client.get(url).send()?;
        let string = response.text()?;
        Ok(string)
    }

    pub fn get_bitcoin_ohlc(&self) -> Result<Ohlc, Box<dyn Error>> {
        let name = "XBTUSD".to_string();
        let string = self.get_ohlc_untyped(&name)?;
        let response: BitCoinResponse = serde_json::from_str(&string)?;
        to_ohlc(name, Box::new(response))
    }
}

fn to_ohlc(name: String, response: Box<dyn OhlcResponse>) -> Result<Ohlc, Box<dyn Error>> {
    let errors = response.errors();
    let result = response.result();

    if errors.errors.is_empty() {
        let ohlc = Ohlc {
            name: name.to_string(),
            currency: Currency::Usd,
            date_time: DateTime::from_timestamp(result.ohlc[0][0].take_i64(), 0).unwrap(),
            open: Decimal::from_str(&result.ohlc[0][1].clone().take_string())?,
            high: Decimal::from_str(&result.ohlc[0][2].clone().take_string())?,
            low: Decimal::from_str(&result.ohlc[0][3].clone().take_string())?,
            close: Decimal::from_str(&result.ohlc[0][4].clone().take_string())?,
            vwap: Decimal::from_str(&result.ohlc[0][5].clone().take_string())?,
            volume: Decimal::from_str(&result.ohlc[0][6].clone().take_string())?,
            count: result.ohlc[0][7].take_i64(),
        };
        println!("{ohlc}");
        Ok(ohlc)
    } else {
        Err(Box::new(errors))
    }
}

trait OhlcResponse {
    fn errors(&self) -> OhlcErrors;
    fn result(&self) -> OhlcVec;
}

impl OhlcResponse for BitCoinResponse {
    fn errors(&self) -> OhlcErrors {
        OhlcErrors {
            errors: self.error.clone(),
        }
    }

    fn result(&self) -> OhlcVec {
        OhlcVec {
            ohlc: self.result.ohlc.clone(),
            last: self.result.last,
        }
    }
}

// fixme: make generic
#[derive(Clone, Debug, Serialize, Deserialize)]
struct BitCoinResponse {
    error: Vec<String>,
    result: BitCoinOhlcVec,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct OhlcVec {
    ohlc: Vec<Vec<IntOrString>>,
    last: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct BitCoinOhlcVec {
    #[serde(rename = "XXBTZUSD")]
    ohlc: Vec<Vec<IntOrString>>,
    last: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum IntOrString {
    I64(i64),
    Str(String),
}

impl IntOrString {
    fn take_i64(&self) -> i64 {
        match self {
            IntOrString::I64(i) => *i,
            IntOrString::Str(_) => panic!("You can only take i64s!"),
        }
    }

    fn take_string(self) -> String {
        match self {
            IntOrString::I64(_) => panic!("You can only take Strings!"),
            IntOrString::Str(s) => s,
        }
    }
}

impl Default for IntOrString {
    fn default() -> Self {
        IntOrString::I64(0)
    }
}

#[derive(Clone, Debug)]
pub struct Ohlc {
    name: String,
    currency: Currency,
    date_time: DateTime<Utc>,
    open: Decimal,
    high: Decimal,
    low: Decimal,
    close: Decimal,
    vwap: Decimal,
    volume: Decimal,
    count: i64,
}

impl Display for Ohlc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        writeln!(f, "name: {}", self.name)?;
        writeln!(f, "currency: {}", self.currency)?;
        writeln!(f, "date_time: {}", self.date_time)?;
        writeln!(f, "open: {}", self.open)?;
        writeln!(f, "high: {}", self.high)?;
        writeln!(f, "low: {}", self.low)?;
        writeln!(f, "close: {}", self.close)?;
        writeln!(f, "vwap: {}", self.vwap)?;
        writeln!(f, "volume: {}", self.volume)?;
        writeln!(f, "count: {}", self.count)
    }
}

#[derive(Debug)]
struct OhlcErrors {
    errors: Vec<String>,
}

impl Display for OhlcErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        writeln!(f, "{:#?}", self.errors)
    }
}

impl Error for OhlcErrors {}

// {"error":[],"result":{"GNOUSD":[[1719878400,"286.27","286.88","284.97","284.97","285.78","4.74983692",10]],"last":1719792000}}
// {"error":[],"result":{"XETHZUSD":[[1719878400,"3438.32","3450.99","3432.20","3444.99","3442.24","357.97391572",651]],"last":1719792000}}
