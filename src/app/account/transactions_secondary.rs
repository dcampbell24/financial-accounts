use serde::{Deserialize, Serialize};

use crate::app::money::Currency;

use super::transaction::Transaction;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Transactions2nd {
    pub currency: Currency,
    pub txs: Vec<Transaction>,
}
