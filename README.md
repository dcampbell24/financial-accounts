# financial-accounts

[![github]](https://github.com/dcampbell24/financial-accounts)&ensp;[![crates-io]](https://crates.io/crates/financial-accounts)&ensp;[![docs-rs]](https://docs.rs/financial-accounts)

[github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
[crates-io]: https://img.shields.io/badge/crates.io-fc8d62?style=for-the-badge&labelColor=555555&logo=rust
[docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs

An application for tracking your personal finances.

To get the gold price I use a service from goldapi.io and it requires a token
(free for 100 requests per month). The token is stored in a file named
"www.goldapi.io-access-token.txt" stored in the same directory as the program
is run from or your home directory if you installed the application.

You can import Bank of America transactions via Import BoA.

Via Tx 2nd it supports and getting the price of Bitcoin, Ethereum, Gnosis, and
gold troy ounce. Tx 2nd becomes active when you select one of these currencies.
Tx remains USD (imported when "Get Price" is selected).

Via Monthly Tx it supports making predictions into the future for what will
happen to your finances. It also automatically submits these transactions at
the beginning of each month.

On Tx and 2nd Tx you can limit transactions displayed by month.

It would be trivial to add support for more cypto currencies and metals, but I
have only implemented the ones I use. I could also add more bank imports, but
again I have only implemented the one I use.
