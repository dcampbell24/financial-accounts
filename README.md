# financial-accounts

[![github]](https://github.com/dcampbell24/financial-accounts)&ensp;[![crates-io]](https://crates.io/crates/financial-accounts)&ensp;[![docs-rs]](https://docs.rs/financial-accounts)

[github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
[crates-io]: https://img.shields.io/badge/crates.io-fc8d62?style=for-the-badge&labelColor=555555&logo=rust
[docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs

An application for tracking your personal finances.

To get the price of metals I use a service from goldapi.io and it requires a
token (free for 100 requests per month). The token is stored in a file named
"goldapi.io.txt" stored in the same directory as the program
is run from or your home directory if you installed the application.

You can import Bank of America transactions via Import BoA.

You can import Investor 360 via Import Investor 360.

Via Tx 2nd you can get the price of metals, stocks plus, and crypto. Tx 2nd
becomes active when you select one of these currencies. Tx is the currency
Tx 2nd is traded in. When you select "Get Price" Tx is populated with the
quantity of Tx 2nd you hold times the current price.

Via Monthly Tx it supports making predictions into the future for what will
happen to your finances. It also automatically submits these transactions at
the beginning of each month.

On Tx and 2nd Tx you can limit transactions displayed by month.

Because there are many crypto, fiat, metals and stocks to choose from, you
select which ones you want in your config file. An example of a config is
shown in demo-ledger.ron, located under `/usr/share/doc/financial-accounts/`
if this was installed as a Debian package. All the allowed fiat currencies are
shown.
