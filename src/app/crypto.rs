use std::{error::Error, fmt::Display, str::FromStr};

use chrono::{DateTime, Utc};
use reqwest::Client;
use reqwest::Url;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::money::{Currency, Fiat};

const URL_KRAKEN_OHLC: &str = "https://api.kraken.com/0/public/OHLC";

pub async fn get_ohlc_untyped(client: &Client, name: &str) -> anyhow::Result<String> {
    let url = Url::parse_with_params(
        URL_KRAKEN_OHLC,
        &[
            ("pair", name),
            // A day.
            ("interval", "1440"),
            ("since", &Utc::now().timestamp().to_string()),
        ],
    )?;

    let response = client.get(url).send().await?;
    let string = response.text().await?;
    Ok(string)
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

macro_rules! impl_get_ohlc {
    ($ty:ident, $request:expr, $response:expr) => {
        #[derive(Clone, Debug, Serialize, Deserialize)]
        pub struct $ty {
            #[serde(rename = $response)]
            ohlc: Vec<Vec<IntOrString>>,
            last: u64,
        }

        impl $ty {
            pub async fn get_price(client: &Client) -> anyhow::Result<Decimal> {
                let name = $request.to_string();
                let string = get_ohlc_untyped(client, &name).await?;
                let response: Response<$ty> = serde_json::from_str(&string)?;
                let ohlc = response.to_ohlc(name)?;
                Ok(ohlc.close)
            }
        }

        impl OhlcResponse for Response<$ty> {
            fn result(&self) -> OhlcVec {
                OhlcVec {
                    ohlc: self.result.ohlc.clone(),
                    last: self.result.last,
                }
            }
        }
    };
}

impl_get_ohlc!(BtcOhlc, "XBTUSD", "XXBTZUSD");
impl_get_ohlc!(EthOhlc, "ETHUSD", "XETHZUSD");
impl_get_ohlc!(GnoOhlc, "GNOUSD", "GNOUSD");

trait OhlcErrorsTrait {
    fn errors(&self) -> OhlcErrors;
}

impl<T> OhlcErrorsTrait for Response<T> {
    fn errors(&self) -> OhlcErrors {
        OhlcErrors {
            errors: self.error.clone(),
        }
    }
}

trait OhlcResponse: OhlcErrorsTrait {
    fn result(&self) -> OhlcVec;

    fn to_ohlc(&self, name: String) -> anyhow::Result<Ohlc> {
        let errors = self.errors();
        let result = self.result();

        if errors.errors.is_empty() {
            let ohlc = Ohlc {
                name,
                currency: Currency::Fiat(Fiat::Usd),
                date_time: DateTime::from_timestamp(result.ohlc[0][0].take_i64(), 0).unwrap(),
                open: Decimal::from_str(&result.ohlc[0][1].clone().take_string())?,
                high: Decimal::from_str(&result.ohlc[0][2].clone().take_string())?,
                low: Decimal::from_str(&result.ohlc[0][3].clone().take_string())?,
                close: Decimal::from_str(&result.ohlc[0][4].clone().take_string())?,
                volume_weighted_average_price: Decimal::from_str(
                    &result.ohlc[0][5].clone().take_string(),
                )?,
                volume: Decimal::from_str(&result.ohlc[0][6].clone().take_string())?,
                count: result.ohlc[0][7].take_i64(),
            };
            Ok(ohlc)
        } else {
            Err(errors)?
        }
    }
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
            Self::I64(i) => *i,
            Self::Str(_) => panic!("You can only take i64s!"),
        }
    }

    fn take_string(self) -> String {
        match self {
            Self::I64(_) => panic!("You can only take Strings!"),
            Self::Str(s) => s,
        }
    }
}

impl Default for IntOrString {
    fn default() -> Self {
        Self::I64(0)
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
    pub volume_weighted_average_price: Decimal,
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
        writeln!(
            f,
            "volume_weighted_average_price: {}",
            self.volume_weighted_average_price
        )?;
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
