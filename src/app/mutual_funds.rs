use std::error::Error;

use html5ever::tendril::TendrilSink;
use html5ever::ParseOpts;
use markup5ever_rcdom::{NodeData, RcDom};
use reqwest::blocking::Client;

use html5ever::parse_document;
use html5ever::tree_builder::{TreeBuilderOpts, TreeSink};

fn get_mutual_fund_price() -> Result<(), Box<dyn Error>> {
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:127.0) Gecko/20100101 Firefox/127.0")
        .build()?;

    let resp = client
        .get("https://finance.yahoo.com/quote/CHTRX/")
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

    'end: while children.len() > 0 {
        for child_1st in children {
            match &child_1st.data {
                NodeData::Document
                | NodeData::Doctype { .. }
                | NodeData::Text { .. }
                | NodeData::Comment { .. }
                | NodeData::ProcessingInstruction { .. } => {}
                NodeData::Element { name, attrs, .. } => {
                    let name: String = name.local.as_ascii().iter().map(|c| c.as_str()).collect();
                    let attrs = attrs.clone().into_inner();
                    if name == "fin-streamer" && attrs[0].value == "CHTRX".into() {
                        println!("{}: {}", attrs[0].value, attrs[1].value);
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

    Ok(())
}
