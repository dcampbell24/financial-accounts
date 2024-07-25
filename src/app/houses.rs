use std::fmt;

use html5ever::tendril::TendrilSink;
use markup5ever_rcdom::{NodeData, RcDom};
use reqwest::blocking::Client;

use html5ever::tree_builder::{TreeBuilderOpts, TreeSink};
use html5ever::{parse_document, ParseOpts};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
pub struct Address {
    street_address: String,
    // apt_suite_other: Option<String>,
    city: String,
    state: String,
    zip_code: String,
    zp_id: String,
}

impl Address {
    fn zillow_search_string(&self) -> String {
        let street_address = self.street_address.replace(' ', "-");
        format!(
            "{}-{},-{}-{}_rb/{}_zpid/",
            street_address, self.city, self.state, self.zip_code, self.zp_id
        )
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            /* name, */ "{}, {}, {} {}, United States",
            self.street_address, self.city, self.state, self.zip_code
        )
    }
}

pub fn get_house_price(client: &Client, address: &Address) -> anyhow::Result<Decimal> {
    let resp = client
        .get(&format!(
            "https://www.zillow.com/homes/{}",
            address.zillow_search_string()
        ))
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
    let mut price = "".to_string();

    'end: while children.len() > 0 {
        for child_1st in children {
            match &child_1st.data {
                NodeData::Document
                | NodeData::Doctype { .. }
                | NodeData::Comment { .. }
                | NodeData::Element { .. }
                | NodeData::ProcessingInstruction { .. } => {}

                NodeData::Text { contents } => {
                    let mut contents: String = contents.clone().into_inner().into();
                    if contents.starts_with('$') {
                        contents = contents.replace('$', "");
                        contents = contents.replace(',', "");
                        price = contents;
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

    if price.is_empty() {
        Err(anyhow::Error::msg(
            "zillow thinks you're a bot. Try again much later!",
        ))
    } else {
        let price: Decimal = price.parse()?;
        Ok(price)
    }
}
