use std::{error::Error, fmt::Display, str::FromStr};

use chrono::{DateTime, Utc};
use reqwest::blocking::Client;
use reqwest::Url;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::money::Currency;

const URL_KRAKEN_OHLC: &str = "https://api.kraken.com/0/public/OHLC";

pub fn get_ohlc_untyped(client: &Client, name: &str) -> Result<String, Box<dyn Error>> {
    let url = Url::parse_with_params(
        URL_KRAKEN_OHLC,
        &[
            ("pair", name),
            // A day.
            ("interval", "1440"),
            ("since", &Utc::now().timestamp().to_string()),
        ],
    )?;

    let response = client.get(url).send()?;
    let string = response.text()?;
    Ok(string)
}

pub fn get_ohlc_bitcoin(client: &Client) -> Result<Ohlc, Box<dyn Error>> {
    let name = "XBTUSD".to_string();
    let string = get_ohlc_untyped(client, &name)?;
    let response: Response<BitCoinOhlcVec> = serde_json::from_str(&string)?;
    response.to_ohlc(name)
}

pub fn get_ohlc_eth(client: &Client) -> Result<Ohlc, Box<dyn Error>> {
    let name = "ETHUSD".to_string();
    let string = get_ohlc_untyped(client, &name)?;
    let response: Response<EthOhlcVec> = serde_json::from_str(&string)?;
    response.to_ohlc(name)
}

pub fn get_ohlc_gno(client: &Client) -> Result<Ohlc, Box<dyn Error>> {
    let name = "GNOUSD".to_string();
    let string = get_ohlc_untyped(client, &name)?;
    let response: Response<GnoOhlcVec> = serde_json::from_str(&string)?;
    response.to_ohlc(name)
}

trait OhlcResponse {
    fn errors(&self) -> OhlcErrors;
    fn result(&self) -> OhlcVec;

    fn to_ohlc(&self, name: String) -> Result<Ohlc, Box<dyn Error>> {
        let errors = self.errors();
        let result = self.result();

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
            Ok(ohlc)
        } else {
            Err(Box::new(errors))
        }
    }
}

impl OhlcResponse for Response<BitCoinOhlcVec> {
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

impl OhlcResponse for Response<EthOhlcVec> {
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

impl OhlcResponse for Response<GnoOhlcVec> {
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

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Response<T> {
    error: Vec<String>,
    result: T,
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
struct EthOhlcVec {
    #[serde(rename = "XETHZUSD")]
    ohlc: Vec<Vec<IntOrString>>,
    last: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct GnoOhlcVec {
    #[serde(rename = "GNOUSD")]
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
    pub name: String,
    pub currency: Currency,
    pub date_time: DateTime<Utc>,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub vwap: Decimal,
    pub volume: Decimal,
    pub count: i64,
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

// impl anyhow::Error for OhlcErrors {}
