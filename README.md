# financial-accounts

[![github]](https://github.com/dcampbell24/financial-accounts)&ensp;[![crates-io]](https://crates.io/crates/financial-accounts)&ensp;[![docs-rs]](https://docs.rs/financial-accounts)

[github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
[crates-io]: https://img.shields.io/badge/crates.io-fc8d62?style=for-the-badge&labelColor=555555&logo=rust
[docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs

[![Build Status](https://app.travis-ci.com/dcampbell24/financial-accounts.svg?token=vLYc8ao87tXucK2UNrhi&branch=main)](https://app.travis-ci.com/dcampbell24/financial-accounts)

An application for tracking your personal finances.

To get the price of metals I use a service from goldapi.io and it requires a
token (free for 100 requests per month). The token is stored in a file named
"goldapi.io.txt" stored in the same directory as the program
is run from or your home directory if you installed the application.

To get the price of stocks I use a service from polygon.io and it requires a
token (free for 5 API Calls / Minute). The token is stored in a file named
"polygon.io.txt" and is preceded by the string "Bearer ". It is stored in the
same directory as the program is run from or your home directory if you
installed the application.

You can import Bank of America transactions via Import BoA.

Via Tx 2nd you can get the price of metals, mutual funds, stocks, Bitcoin,
Ethereum, and Gnosis. Tx 2nd becomes active when you select one of these
currencies. Tx is the currency Tx 2nd is traded in. When you select "Get Price"
Tx is populated with the quantity of Tx 2nd you hold times the current price
(sometimes a day old price).

Via Monthly Tx it supports making predictions into the future for what will
happen to your finances. It also automatically submits these transactions at
the beginning of each month.

On Tx and 2nd Tx you can limit transactions displayed by month.

Because there are many metals and stocks to choose from and many possible
fiat currencies, you select which ones you want in your config file. An
example of a config is shown in demo-ledger.ron. All the allowed fiat
currencies are shown.

I may allow for cypto currencies in the same way I do stocks and metals, but I
haven't figured out the best way to support it yet.

## Zillow

Zillow doesn't like bots scraping it's site, so in order to scrape the site
(and get the price of a house) you have to go to the page of the address you
want _in Firefox_ and do a captcha. Then the application will grab your cookies
from a _default_ profile and use them. Next the application creates a file
named "zillow-cookies.json" in the directory you ran the application from and
reads your cookies from that file in the future.
