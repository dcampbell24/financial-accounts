use std::fmt;

use serde::{Deserialize, Serialize};

use super::houses::Address;

#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
pub enum Currency {
    Btc,
    Eth,
    Gno,
    Fiat(Fiat),
    House(Address),
    Metal(Metal),
    MutualFund(MutualFund),
    Stock(Stock),
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Btc => write!(f, "BTC"),
            Self::Eth => write!(f, "ETH"),
            Self::Gno => write!(f, "GNO"),
            Self::Fiat(fiat) => write!(f, "{fiat}"),
            Self::House(address) => write!(f, "House: {address}"),
            Self::Metal(metal) => write!(f, "Metal: {metal}"),
            Self::MutualFund(mutual_fund) => write!(f, "Mutual Fund: {mutual_fund}"),
            Self::Stock(stock) => write!(f, "Stock: {stock}"),
        }
    }
}

// Supported by https://www.goldapi.io/ .
#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Serialize)]
pub enum Fiat {
    Usd,
    Aud,
    Gbp,
    Eur,
    Cad,
    Chf,
    Jpy,
    Krw,
    Inr,
    Cny,
    Zar,
    Thb,
    Sgd,
    Hkd,
    // Btc, // Bitcoin - clearly not a fiat currency.
    Czk,
    Pln,
    Myr,
    Rub,
    Aed,
    Kwd,
    Egp,
    Omr,
    Sar,
    Mxn,
    Jod,
}

impl Fiat {
    pub fn symbol(&self) -> String {
        match self {
            Self::Usd => "USD".to_string(),
            Self::Aud => "AUD".to_string(),
            Self::Gbp => "GBP".to_string(),
            Self::Eur => "EUR".to_string(),
            Self::Cad => "CAD".to_string(),
            Self::Chf => "CHF".to_string(),
            Self::Jpy => "JPY".to_string(),
            Self::Krw => "KRW".to_string(),
            Self::Inr => "INR".to_string(),
            Self::Cny => "CNY".to_string(),
            Self::Zar => "ZAR".to_string(),
            Self::Thb => "THB".to_string(),
            Self::Sgd => "SGD".to_string(),
            Self::Hkd => "HKD".to_string(),
            Self::Czk => "CZK".to_string(),
            Self::Pln => "PLN".to_string(),
            Self::Myr => "MYR".to_string(),
            Self::Rub => "RUB".to_string(),
            Self::Aed => "AED".to_string(),
            Self::Kwd => "KWD".to_string(),
            Self::Egp => "EGP".to_string(),
            Self::Omr => "OMR".to_string(),
            Self::Sar => "SAR".to_string(),
            Self::Mxn => "MXN".to_string(),
            Self::Jod => "JOD".to_string(),
        }
    }
}

impl fmt::Display for Fiat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usd => write!(f, "United States Dollar"),
            Self::Aud => write!(f, "Australian Dollar"),
            Self::Gbp => write!(f, "British Pound"),
            Self::Eur => write!(f, "European Euro"),
            Self::Cad => write!(f, "Canadian Dollar"),
            Self::Chf => write!(f, "Swiss Franc"),
            Self::Jpy => write!(f, "Japanese Yen"),
            Self::Krw => write!(f, "South Korean Won"),
            Self::Inr => write!(f, "Indian Rupee"),
            Self::Cny => write!(f, "Chinese/Yuan Renminbi"),
            Self::Zar => write!(f, "South African Rand"),
            Self::Thb => write!(f, "Thai Baht"),
            Self::Sgd => write!(f, "Singapore Dollar"),
            Self::Hkd => write!(f, "Hong Kong Dollar"),
            Self::Czk => write!(f, "Czech Krona"),
            Self::Pln => write!(f, "Polish Złoty"),
            Self::Myr => write!(f, "Malaysian Ringgit"),
            Self::Rub => write!(f, "Russian Ruble"),
            Self::Aed => write!(f, "U.A.E. Dirham"),
            Self::Kwd => write!(f, "Kuwaiti Dinar"),
            Self::Egp => write!(f, "Egyptian Pound"),
            Self::Omr => write!(f, "Omani Rial"),
            Self::Sar => write!(f, "Saudi Rial"),
            Self::Mxn => write!(f, "Mexican Peso"),
            Self::Jod => write!(f, "Jordanian Dinar"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Serialize)]
pub struct Metal {
    pub currency: Fiat,
    pub description: String,
    pub symbol: String,
}

impl fmt::Display for Metal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description)
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Serialize)]
pub struct MutualFund {
    // currency: USD
    pub description: String,
    pub symbol: String,
}

impl fmt::Display for MutualFund {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description)
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Serialize)]
pub struct Stock {
    // currency: USD
    pub description: String,
    pub symbol: String,
}

impl fmt::Display for Stock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description)
    }
}
