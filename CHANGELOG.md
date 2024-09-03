# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Changed

- Make shortcuts to the URLs you open for imports.
- Support pasting into text boxes.
- Lock the configuration file while in use.
- Breaking Change: add groups.
- Breaking Change: get rid of the old crypto.
- Make crypto configurable.
- Add configuration screen.
- Add buttons: Load, Save, Check Monthly.
- Use rfd.
- Breaking change: Remove support for houses.
- Breaking change: get rid of Stock in favor of StockPlus.
- Breaking change: rename mutual_funds to stocks_plus.
- Add import_investor_360().
- Update boa_import to the new way BoA imports work.
- Turn on clippy::{all, nursery, pedantic, cargo} and fix most lints.
- Add houses and make them configurable via the configuration file.
- Add mutual funds and make them configurable via the configuration file.
- Add fiat currencies and make them configurable via the configuration file.
- Make metals and stocks configurable via the configuration file.
- Change the configuration file format to RON.
- [Add stocks](https://github.com/dcampbell24/financial-accounts/commit/e54732e3819d0ca843567259dabb04b194a7f1bc).
- [Move the account messages to MessageAccount](https://github.com/dcampbell24/financial-accounts/commit/56f6705caaea2fa07bb0331116c47adaa69880f4).
- [Display ParseDateError](https://github.com/dcampbell24/financial-accounts/commit/3627d92a30ea5ac1d86298b04e254e61513f4d4d).
- [Trim when you add a string not modify it](https://github.com/dcampbell24/financial-accounts/commit/cbc5b5ba4bfad7f497b097c17bed567936f08d91).
- [Sort the accounts when you change a name](https://github.com/dcampbell24/financial-accounts/commit/351a52a8111137d8a2c99749b451a78cb91a7611).
- [Get rid of Txs2nd](https://github.com/dcampbell24/financial-accounts/commit/ae7a0bfe86fec03acc177f2912afe9c872359b8c).
- [Switch from markdown to djot for the man page](https://github.com/dcampbell24/financial-accounts/commit/66929e72e6c5bd0bbc0c2f447295fb02e5bf4a3b).
- [Chart filtered transactions](https://github.com/dcampbell24/financial-accounts/commit/dad92faaa4b339aa0be7bf202d34d9768911fb06).
- [Change ticker module to crypto module](https://github.com/dcampbell24/financial-accounts/commit/b555d6a38dfea71f4ea7a66d93232b5e1f8263db).

## [0.1.3] - 2024-07-12

### Added

- First release!

[0.1.3]: https://crates.io/crates/financial-accounts/0.1.3
