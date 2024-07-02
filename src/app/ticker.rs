use chrono::Utc;
use reqwest::Url;
use reqwest::blocking::Client;

const URL_KRAKEN_OHLC: &str = "https://api.kraken.com/0/public/OHLC";

pub struct Ticker {
    http_client: Client,
}

impl Ticker {
    pub fn init() -> Self {
        Ticker {
            http_client: Client::new(),
        }
    }

    pub fn get_ohlc(&self, pair: &str) {
        let url = Url::parse_with_params(
            URL_KRAKEN_OHLC,
            &[
                ("pair", pair),
                // A day.
                ("interval", "1440"),
                ("since", &Utc::now().timestamp().to_string()),
            ],
        )
        .unwrap();

        let response = self.http_client.get(url).send().unwrap();
        println!("{response:#?}")
    }
}
