use std::fmt;

use chrono::{TimeZone, Utc};
use html5ever::tendril::TendrilSink;
use markup5ever_rcdom::{NodeData, RcDom};
use reqwest::Client;

use html5ever::tree_builder::{TreeBuilderOpts, TreeSink};
use html5ever::{parse_document, ParseOpts};
use reqwest::Url;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::account::transactions::Price;
use super::zillow_cookies;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Serialize)]
pub struct House {
    street_address: String,
    // apt_suite_other: Option<String>,
    city: String,
    state: String,
    zip_code: String,
    zp_id: String,
}

impl House {
    fn zillow_search_string(&self) -> String {
        let street_address = self.street_address.replace(' ', "-");
        format!(
            "{}-{},-{}-{}_rb/{}_zpid/",
            street_address, self.city, self.state, self.zip_code, self.zp_id
        )
    }
}

impl fmt::Display for House {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            /* name, */ "{}, {}, {} {}, United States",
            self.street_address, self.city, self.state, self.zip_code
        )
    }
}

impl Price for House {
    async fn get_price(&self, _client: &Client) -> anyhow::Result<Decimal> {
        let cookie_store = {
            if let Ok(file) =
                std::fs::File::open("zillow-cookies.json").map(std::io::BufReader::new)
            {
                reqwest_cookie_store::CookieStore::load_json(file).unwrap()
            } else {
                let cookies = zillow_cookies::get_cookies()?;
                let mut cookie_store = reqwest_cookie_store::CookieStore::new(None);
                let request_url = Url::parse("https://www.zillow.com")?;

                for cookie in cookies {
                    if Utc::timestamp_opt(&Utc, cookie.expiry, 0).unwrap() > Utc::now() {
                        cookie_store.insert(
                            cookie_store::Cookie::parse(cookie.to_string(), &request_url)?,
                            &request_url,
                        )?;
                    }
                }
                cookie_store
            }
        };

        let cookie_store = reqwest_cookie_store::CookieStoreMutex::new(cookie_store);
        let cookie_store = std::sync::Arc::new(cookie_store);

        let client = Client::builder()
            .user_agent("Mozilla/5.0 (compatible; financial-accounts/0.2-dev; +https://github.com/dcampbell24/financial-accounts)")
            .cookie_provider(std::sync::Arc::clone(&cookie_store))
            .build()?;

        let resp = client
            .get(format!(
                "https://www.zillow.com/homes/{}",
                self.zillow_search_string()
            ))
            .send()
            .await?;

        let text = resp.text().await?;

        let mut writer =
            std::fs::File::create("zillow-cookies.json").map(std::io::BufWriter::new)?;

        {
            let store = cookie_store.lock().unwrap();
            store.save_json(&mut writer).unwrap();
        }

        let opts = ParseOpts {
            tree_builder: TreeBuilderOpts {
                drop_doctype: true,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut dom = parse_document(RcDom::default(), opts)
            .from_utf8()
            .read_from(&mut text.as_bytes())?;

        let document = dom.get_document();

        let mut children = document.children.clone().into_inner();
        let mut children_2nd = Vec::new();
        let mut price = String::new();

        'end: while !children.is_empty() {
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
                "zillow thinks you're a bot. Re-do the captcha!",
            ))
        } else {
            let price: Decimal = price.parse()?;
            Ok(price)
        }
    }
}
