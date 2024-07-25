use std::fmt;

use serde::{Deserialize, Serialize};

use super::houses::Address;

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
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
            Currency::Btc => write!(f, "BTC"),
            Currency::Eth => write!(f, "ETH"),
            Currency::Gno => write!(f, "GNO"),
            Currency::Fiat(fiat) => write!(f, "{fiat}"),
            Currency::House(address) => write!(f, "House: {address}"),
            Currency::Metal(metal) => write!(f, "Metal: {metal}"),
            Currency::MutualFund(mutual_fund) => write!(f, "Mutual Fund: {mutual_fund}"),
            Currency::Stock(stock) => write!(f, "Stock: {stock}"),
        }
    }
}

// Supported by https://www.goldapi.io/ .
#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
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
            Fiat::Usd => "USD".to_string(),
            Fiat::Aud => "AUD".to_string(),
            Fiat::Gbp => "GBP".to_string(),
            Fiat::Eur => "EUR".to_string(),
            Fiat::Cad => "CAD".to_string(),
            Fiat::Chf => "CHF".to_string(),
            Fiat::Jpy => "JPY".to_string(),
            Fiat::Krw => "KRW".to_string(),
            Fiat::Inr => "INR".to_string(),
            Fiat::Cny => "CNY".to_string(),
            Fiat::Zar => "ZAR".to_string(),
            Fiat::Thb => "THB".to_string(),
            Fiat::Sgd => "SGD".to_string(),
            Fiat::Hkd => "HKD".to_string(),
            Fiat::Czk => "CZK".to_string(),
            Fiat::Pln => "PLN".to_string(),
            Fiat::Myr => "MYR".to_string(),
            Fiat::Rub => "RUB".to_string(),
            Fiat::Aed => "AED".to_string(),
            Fiat::Kwd => "KWD".to_string(),
            Fiat::Egp => "EGP".to_string(),
            Fiat::Omr => "OMR".to_string(),
            Fiat::Sar => "SAR".to_string(),
            Fiat::Mxn => "MXN".to_string(),
            Fiat::Jod => "JOD".to_string(),
        }
    }
}

impl fmt::Display for Fiat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Fiat::Usd => write!(f, "United States Dollar"),
            Fiat::Aud => write!(f, "Australian Dollar"),
            Fiat::Gbp => write!(f, "British Pound"),
            Fiat::Eur => write!(f, "European Euro"),
            Fiat::Cad => write!(f, "Canadian Dollar"),
            Fiat::Chf => write!(f, "Swiss Franc"),
            Fiat::Jpy => write!(f, "Japanese Yen"),
            Fiat::Krw => write!(f, "South Korean Won"),
            Fiat::Inr => write!(f, "Indian Rupee"),
            Fiat::Cny => write!(f, "Chinese/Yuan Renminbi"),
            Fiat::Zar => write!(f, "South African Rand"),
            Fiat::Thb => write!(f, "Thai Baht"),
            Fiat::Sgd => write!(f, "Singapore Dollar"),
            Fiat::Hkd => write!(f, "Hong Kong Dollar"),
            Fiat::Czk => write!(f, "Czech Krona"),
            Fiat::Pln => write!(f, "Polish Złoty"),
            Fiat::Myr => write!(f, "Malaysian Ringgit"),
            Fiat::Rub => write!(f, "Russian Ruble"),
            Fiat::Aed => write!(f, "U.A.E. Dirham"),
            Fiat::Kwd => write!(f, "Kuwaiti Dinar"),
            Fiat::Egp => write!(f, "Egyptian Pound"),
            Fiat::Omr => write!(f, "Omani Rial"),
            Fiat::Sar => write!(f, "Saudi Rial"),
            Fiat::Mxn => write!(f, "Mexican Peso"),
            Fiat::Jod => write!(f, "Jordanian Dinar"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
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

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
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

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
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
