use html5ever::tendril::TendrilSink;
use html5ever::ParseOpts;
use markup5ever_rcdom::{NodeData, RcDom};
use reqwest::blocking::Client;

use html5ever::parse_document;
use html5ever::tree_builder::{TreeBuilderOpts, TreeSink};
use rust_decimal::Decimal;

pub fn get_mutual_fund_price(client: &Client, symbol: &str) -> anyhow::Result<Decimal> {
    let resp = client
        .get(format!("https://finance.yahoo.com/quote/{symbol}/"))
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
                    if name == "fin-streamer" && attrs[0].value == symbol.into() {
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
