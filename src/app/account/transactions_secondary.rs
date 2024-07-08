use serde::{Deserialize, Serialize};

use crate::app::money::Currency;

use super::transaction::Transaction;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Transactions2nd {
    pub currency: Currency,
    #[serde(rename = "transactions")]
    pub txs: Vec<Transaction>,
}

impl Transactions2nd {
    pub fn new(currency: Currency) -> Self {
        Transactions2nd {
            currency,
            txs: Vec::new(),
        }
    }
}
