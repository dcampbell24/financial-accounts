use std::fmt;

use reqwest::Client;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use yahoo_finance_api::YahooConnector;

use super::account::transactions::Price;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Serialize)]
pub struct StockPlus {
    // currency: USD
    pub description: String,
    pub symbol: String,
}

impl Price for StockPlus {
    async fn get_price(&self, _client: &Client) -> anyhow::Result<Decimal> {
        let provider = YahooConnector::new()?;
        let response = provider.get_latest_quotes(&self.symbol, "1d").await?;
        let quote = response.last_quote()?;

        Ok(quote.close)
    }
}

impl fmt::Display for StockPlus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} in United States Dollar", self.description)
    }
}
