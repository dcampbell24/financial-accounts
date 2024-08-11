use std::fmt;

use html5ever::{
    parse_document,
    tendril::TendrilSink,
    tree_builder::{TreeBuilderOpts, TreeSink},
    ParseOpts,
};
use markup5ever_rcdom::{NodeData, RcDom};
use reqwest::blocking::Client;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::account::transactions::Price;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Serialize)]
pub struct StockPlus {
    // currency: USD
    pub description: String,
    pub symbol: String,
}

impl Price for StockPlus {
    fn get_price(&self, client: &Client) -> anyhow::Result<Decimal> {
        let resp = client
            .get(format!("https://finance.yahoo.com/quote/{}/", self.symbol))
            .send()?;

        let text = resp.text()?;

        let opts = ParseOpts {
            tree_builder: TreeBuilderOpts {
                drop_doctype: true,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut dom = parse_document(RcDom::default(), opts)
            .from_utf8()
            .read_from(&mut text.as_bytes())
            .unwrap();

        let document = dom.get_document();

        let mut children = document.children.clone().into_inner();
        let mut children_2nd = Vec::new();
        let mut price = "".into();

        'end: while !children.is_empty() {
            for child_1st in children {
                match &child_1st.data {
                    NodeData::Document
                    | NodeData::Doctype { .. }
                    | NodeData::Text { .. }
                    | NodeData::Comment { .. }
                    | NodeData::ProcessingInstruction { .. } => {}
                    NodeData::Element { name, attrs, .. } => {
                        let name = name.local.to_string();
                        let attrs = attrs.clone().into_inner();
                        if name == "fin-streamer" && attrs[0].value == self.symbol.clone().into() {
                            price = attrs[1].value.clone();
                            break 'end;
                        }
                    }
                }
                for child_2nd in child_1st.children.clone().into_inner() {
                    children_2nd.push(child_2nd);
                }
            }
            children = children_2nd;
            children_2nd = Vec::new();
        }

        let price: Decimal = price.parse()?;
        Ok(price)
    }
}

impl fmt::Display for StockPlus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description)
    }
}
