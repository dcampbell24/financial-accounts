mod account;
mod accounts;
mod chart;
mod command_line;
mod crypto;
mod import_boa;
mod message;
mod metal;
mod money;
mod screen;
pub mod solarized;
mod stocks;

use std::{cmp::Ordering, fs, mem::take, path::PathBuf, str::FromStr, sync::Arc};

use account::{transaction::Transaction, transactions::Transactions};
use accounts::Group;
use anyhow::Context;
use chart::Chart;
use chrono::Utc;
use crypto::Crypto;
use iced::{
    widget::{
        self, button, column,
        combo_box::{ComboBox, State},
        row,
        text::IntoFragment,
        text_input, Button, Checkbox, Column, ProgressBar, Row, Scrollable,
    },
    Alignment, Element, Length, Task, Theme,
};
use metal::Metal;
use money::{Currency, Fiat};
use plotters_iced::ChartWidget;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::Deserialize;
use stocks::StockPlus;
use thousands::Separable;

use crate::app::{account::Account, accounts::Accounts, message::Message, screen::Screen};

const BOA_URL: &str = "https://secure.bankofamerica.com/myaccounts/brain/redirect.go?target=portfolio&portfolio_page=transactions&request_locale=en-us&source=overview&fsd=y";
const INVESTOR_360_URL: &str = "https://my.investor360.com/nce/Holdings";

const TITLE_FILE_PICKER: &str = "Financial Accounts";
const LAST_DATE_SCALE: u32 = 4;
const EDGE_PADDING: usize = 4;
const PADDING: u16 = 1;
const CHECKBOX_SPACING: f32 = 12.5;
const COLUMN_SPACING: f32 = 0.3;
const ROW_SPACING: u16 = 5;
const TEXT_SIZE: u16 = 24;

/// The financial-accounts application.
#[derive(Debug)]
pub struct App {
    accounts: Accounts,
    file: Option<File>,
    account_name: String,
    crypto_currency: Option<Fiat>,
    crypto_currency_selector: State<Fiat>,
    crypto_description: String,
    crypto_symbol: String,
    fiat: Option<Fiat>,
    fiat_selector: State<Fiat>,
    metal_currency: Option<Fiat>,
    metal_currency_selector: State<Fiat>,
    metal_description: String,
    metal_symbol: String,
    progress_bar: Option<f32>,
    stock_plus_description: String,
    stock_plus_symbol: String,
    currency: Option<Currency>,
    currency_selector: State<Currency>,
    duration: Duration,
    project_months: Option<u16>,
    screen: Screen,
    errors: Option<Arc<Vec<anyhow::Error>>>,
}

impl App {
    fn add_crypto(&mut self) {
        if let Some(fiat) = &self.crypto_currency {
            self.accounts.crypto.push(Crypto {
                currency: fiat.clone(),
                description: self.crypto_description.clone(),
                symbol: self.crypto_symbol.clone(),
            });
            self.currency_selector = State::new(self.accounts.get_currencies());
            self.save();
        }
    }

    fn add_fiat(&mut self) {
        if let Some(fiat) = &self.fiat {
            self.accounts.fiats.push(fiat.clone());
            self.fiat_selector = State::new(Fiat::all_minus_existing(&self.accounts.fiats));
            self.currency_selector = State::new(self.accounts.get_currencies());
            self.save();
        }
    }

    fn add_group(&mut self) {
        let members = (0..)
            .zip(self.accounts.inner.iter())
            .filter_map(
                |(index, account)| {
                    if account.check_box {
                        Some(index)
                    } else {
                        None
                    }
                },
            )
            .collect();

        let group = Group {
            name: self.account_name.clone(),
            members,
        };

        self.accounts.groups.push(group);
        self.save();
    }

    fn add_metal(&mut self) {
        if let Some(fiat) = &self.metal_currency {
            self.accounts.metals.push(Metal {
                currency: fiat.clone(),
                description: self.metal_description.clone(),
                symbol: self.metal_symbol.clone(),
            });
            self.currency_selector = State::new(self.accounts.get_currencies());
            self.save();
        }
    }

    fn add_stock_plus(&mut self) {
        self.accounts.stocks_plus.push(StockPlus {
            description: self.stock_plus_description.clone(),
            symbol: self.stock_plus_symbol.clone(),
        });
        self.currency_selector = State::new(self.accounts.get_currencies());
        self.save();
    }

    fn config(&self) -> Scrollable<Message> {
        let mut crypto_current = Column::new();
        for crypto in &self.accounts.crypto {
            let crypto = format!("{crypto:?}");
            crypto_current = crypto_current.push(text_cell(crypto));
        }

        let add_crypto = row![
            button_cell(button("Add Crypto").on_press(Message::AddCrypto)),
            ComboBox::new(
                &self.crypto_currency_selector,
                "fiats",
                self.crypto_currency.as_ref(),
                |fiat| { Message::UpdateCryptoCurrency(fiat) }
            ),
            text_cell("Description:"),
            text_input("Description", &self.crypto_description)
                .on_input(Message::UpdateCryptoDescription)
                .on_paste(Message::UpdateCryptoDescription),
            text_cell("Symbol:"),
            text_input("Symbol", &self.crypto_symbol)
                .on_input(Message::UpdateCryptoSymbol)
                .on_paste(Message::UpdateCryptoSymbol)
        ];

        let mut fiats_current = Column::new();
        for fiat in &self.accounts.fiats {
            fiats_current = fiats_current.push(text_cell(fiat.to_string()));
        }

        let add_fiat = row![
            button_cell(button("Add Fiat").on_press(Message::AddFiat)),
            ComboBox::new(&self.fiat_selector, "fiats", self.fiat.as_ref(), |fiat| {
                Message::UpdateFiat(fiat)
            }),
        ];

        let mut metals_current = Column::new();
        for metal in &self.accounts.metals {
            let metal = format!("{metal:?}");
            metals_current = metals_current.push(text_cell(metal));
        }

        let add_metal = row![
            button_cell(button("Add Metal").on_press(Message::AddMetal)),
            ComboBox::new(
                &self.metal_currency_selector,
                "fiats",
                self.metal_currency.as_ref(),
                |fiat| { Message::UpdateMetalCurrency(fiat) }
            ),
            text_cell("Description:"),
            text_input("Description", &self.metal_description)
                .on_input(Message::UpdateMetalDescription)
                .on_paste(Message::UpdateMetalDescription),
            text_cell("Symbol:"),
            text_input("Symbol", &self.metal_symbol)
                .on_input(Message::UpdateMetalSymbol)
                .on_paste(Message::UpdateMetalSymbol),
        ];

        let mut stock_plus_current = Column::new();
        for stock_plus in &self.accounts.stocks_plus {
            let stock_plus = format!("{stock_plus:?}");
            stock_plus_current = stock_plus_current.push(text_cell(stock_plus));
        }

        let add_stock_plus = row![
            button_cell(button("Add Stock Plus").on_press(Message::AddStockPlus)),
            text_cell("Description:"),
            text_input("Description", &self.stock_plus_description)
                .on_input(Message::UpdateStockPlusDescription)
                .on_paste(Message::UpdateStockPlusDescription),
            text_cell("Symbol:"),
            text_input("Symbol", &self.stock_plus_symbol)
                .on_input(Message::UpdateStockPlusSymbol)
                .on_paste(Message::UpdateStockPlusSymbol),
        ];

        let mut column_errors = Column::new();
        if let Some(errors) = &self.errors {
            for error in errors.iter() {
                column_errors = column_errors.push(text_cell_red(error.to_string()));
            }
        }

        let cols = column![
            crypto_current,
            add_crypto,
            fiats_current,
            add_fiat,
            metals_current,
            add_metal,
            stock_plus_current,
            add_stock_plus,
            column_errors,
            button_cell(button("Back").on_press(Message::Back)),
        ];

        Scrollable::new(cols)
    }

    fn display_error(&mut self, error: anyhow::Error) {
        match self.errors {
            Some(ref mut errors) => {
                let errors = Arc::get_mut(errors).unwrap();
                errors.push(error);
            }
            None => self.errors = Some(Arc::new(vec![error])),
        }
    }

    fn load_file(&mut self) {
        let result = rfd::FileDialog::new()
            .set_title(TITLE_FILE_PICKER)
            .add_filter("ron", &["ron"])
            .pick_file()
            .context("You must choose a file name for your configuration file.");

        match result {
            Ok(file_path) => match Accounts::load(take(&mut self.file), file_path) {
                Ok((accounts, file)) => {
                    self.accounts = accounts;
                    self.file = Some(file);
                }
                Err(error) => self.display_error(error),
            },
            Err(error) => self.display_error(error),
        }
    }

    fn open_url(&mut self, url: &str) {
        if let Err(error) = webbrowser::open(url) {
            self.display_error(error.into());
        }
    }

    fn save_file(&mut self) {
        let result = rfd::FileDialog::new()
            .set_title(TITLE_FILE_PICKER)
            .add_filter("ron", &["ron"])
            .save_file()
            .context("You must choose a file name for your configuration file.");

        match result {
            Ok(file_path) => match self.accounts.save_dialogue(take(&mut self.file), file_path) {
                Ok(file) => self.file = Some(file),
                Err(error) => self.display_error(error),
            },
            Err(error) => self.display_error(error),
        }
    }

    fn save(&mut self) {
        match self.accounts.save(take(&mut self.file)) {
            Ok(file) => self.file = Some(file),
            Err(error) => self.display_error(error),
        }
    }

    fn new(accounts: Accounts, file: Option<File>) -> Self {
        let currencies = accounts.get_currencies();

        Self {
            fiat_selector: State::new(Fiat::all_minus_existing(&accounts.fiats)),

            accounts,
            file,
            account_name: String::new(),
            crypto_currency: None,
            crypto_currency_selector: State::new(Fiat::all()),
            crypto_description: String::new(),
            crypto_symbol: String::new(),
            fiat: None,
            metal_currency: None,
            metal_currency_selector: State::new(Fiat::all()),
            metal_description: String::new(),
            metal_symbol: String::new(),
            progress_bar: None,
            stock_plus_description: String::new(),
            stock_plus_symbol: String::new(),
            currency: None,
            currency_selector: State::new(currencies),
            duration: Duration::default(),
            project_months: None,
            screen: Screen::Accounts,
            errors: None,
        }
    }

    fn change_project_months(&mut self, months: &str) {
        if months.is_empty() {
            self.project_months = None;
        } else if let Ok(months) = months.parse() {
            self.project_months = Some(months);
        }
    }

    fn check_account_name(&mut self, name: &str) -> anyhow::Result<()> {
        for account in &self.accounts.inner {
            if name == account.name {
                return Err(anyhow::Error::msg("Duplicate name!"));
            }
        }
        Ok(())
    }

    fn delete(&mut self, i: usize) {
        match self.screen {
            Screen::Accounts => {
                self.accounts.inner.remove(i);
                for group in &mut self.accounts.groups {
                    group.remove(i);
                }
            }
            Screen::Account(j) => {
                self.accounts[j].txs_1st.txs.remove(i);
            }
            Screen::AccountSecondary(j) => {
                self.accounts[j].txs_2nd.as_mut().unwrap().txs.remove(i);
            }
            Screen::Configuration => panic!("Nothing to delete!"),
        }

        self.save();
    }

    fn delete_group(&mut self, i: usize) {
        self.accounts.groups.remove(i);
        self.save();
    }

    fn import_investor_360(&mut self, file_xls: &PathBuf) -> anyhow::Result<()> {
        let file_csv = file_xls.file_stem().unwrap();
        let mut file_csv = PathBuf::from_str(file_csv.to_str().unwrap())?;
        file_csv.set_extension("csv");

        if fs::exists(&file_csv)? {
            return Err(anyhow::Error::msg(format!(
                "\"{}\" already exists!",
                &file_csv.display()
            )));
        }

        std::process::Command::new("libreoffice")
            .arg("--convert-to")
            .arg("csv")
            .arg(file_xls)
            .status()
            .context(
                r#"Couldn't execute "libreoffice --convert-to csv", you must install libreoffice."#,
            )?;

        for investor_360_record in csv::Reader::from_path(&file_csv)?.deserialize() {
            let investor_360_record: Investor360 = investor_360_record?;

            // Skip some junk records.
            if investor_360_record.symbol.is_empty() {
                continue;
            }

            let balance = investor_360_record.quantity.unwrap();
            let mut tx = Transaction {
                amount: dec!(0),
                balance,
                comment: investor_360_record.description.clone(),
                date: Utc::now(),
            };

            let name = format!("Investor 360: {}", &investor_360_record.symbol);
            let mut name_matches = false;
            for account in &mut self.accounts.inner {
                if account.name == name {
                    if investor_360_record.price.unwrap() == dec!(1) {
                        tx.amount = balance - account.txs_1st.txs.last().unwrap().balance;
                        account.txs_1st.txs.push(tx.clone());
                    } else {
                        tx.amount = balance
                            - account
                                .txs_2nd
                                .as_ref()
                                .unwrap()
                                .txs
                                .last()
                                .unwrap()
                                .balance;
                        account.txs_2nd.as_mut().unwrap().txs.push(tx.clone());
                    }
                    name_matches = true;
                    break;
                }
            }

            if !name_matches {
                tx.amount = balance;
                let txs = vec![tx];

                if investor_360_record.price.unwrap() == dec!(1) {
                    let currency = Fiat::Usd;
                    let transactions = Transactions {
                        currency: currency.clone(),
                        txs,
                    };
                    let mut account = Account::new(name, Currency::Fiat(currency));
                    account.txs_1st = transactions;
                    self.accounts.inner.push(account);
                } else {
                    let stock = StockPlus {
                        description: investor_360_record.description,
                        symbol: investor_360_record.symbol,
                    };
                    let currency = Currency::StockPlus(stock);
                    let transactions = Transactions {
                        currency: currency.clone(),
                        txs,
                    };
                    let mut account = Account::new(name, currency);
                    account.txs_2nd = Some(transactions);
                    self.accounts.inner.push(account);
                }
            }
        }

        fs::remove_file(&file_csv)?;
        Ok(())
    }

    #[rustfmt::skip]
    fn display_groups(&self) -> GroupColumnDisplay {
        let mut a_ = column![text_cell(""), text_cell("Group")];
        let mut b_ = column![text_cell(""), text_cell("")].align_x(Alignment::End);
        let mut c_ = column![text_cell(""), text_cell("")].align_x(Alignment::End);
        let mut d_ = column![text_cell(""), text_cell("")].align_x(Alignment::End);
        let mut e_ = column![text_cell(""), text_cell("")].align_x(Alignment::End);
        let mut f_ = column![text_cell(""), text_cell("")];

        for (index, group) in self.accounts.groups.iter().enumerate() {
            a_ = a_.push(text_cell(&group.name));
            let mut sum_before_last_week = dec!(0);
            let mut sum_last_week = dec!(0);
            let mut sum_before_last_month = dec!(0);
            let mut sum_last_month = dec!(0);
            let mut sum_before_last_year = dec!(0);
            let mut sum_last_year = dec!(0);
            let mut balance = dec!(0);

            for index in &group.members {
                let (before_last_week, last_week) = self.accounts.inner[*index].sum_last_week();
                let (before_last_month, last_month) = self.accounts.inner[*index].sum_last_month();
                let (before_last_year, last_year) = self.accounts.inner[*index].sum_last_year();

                sum_before_last_week += before_last_week;
                sum_last_week += last_week;
                sum_before_last_month += before_last_month;
                sum_last_month += last_month;
                sum_before_last_year += before_last_year;
                sum_last_year += last_year;
                balance += self.accounts.inner[*index].balance_1st();
            }

            sum_last_week = div_0_ok(sum_last_week, sum_before_last_week);
            sum_last_month = div_0_ok(sum_last_month, sum_before_last_month);
            sum_last_year = div_0_ok(sum_last_year, sum_before_last_year);

            sum_last_week.rescale(LAST_DATE_SCALE);
            sum_last_month.rescale(LAST_DATE_SCALE);
            sum_last_year.rescale(LAST_DATE_SCALE);
            balance.rescale(2);

            b_ = b_.push(number_cell(sum_last_week));
            c_ = c_.push(number_cell(sum_last_month));
            d_ = d_.push(number_cell(sum_last_year));
            e_ = e_.push(number_cell(balance));
            f_ = f_.push(button_cell(button("Delete").on_press(Message::DeleteGroup(index))));
        }

        GroupColumnDisplay { a: a_, b: b_, c: c_, d: d_, e: e_, f: f_}
    }

    fn display_totals(&self, currency: &Fiat) -> TotalsColumnDisplay {
        let (before_last_week, mut last_week) = self.accounts.total_for_last_week(currency);
        let (before_last_month, mut last_month) = self.accounts.total_for_last_month(currency);
        let (before_last_year, mut last_year) = self.accounts.total_for_last_year(currency);
        let mut balance = self.accounts.balance(currency);

        last_week = div_0_ok(last_week, before_last_week);
        last_month = div_0_ok(last_month, before_last_month);
        last_year = div_0_ok(last_year, before_last_year);

        last_week.rescale(LAST_DATE_SCALE);
        last_month.rescale(LAST_DATE_SCALE);
        last_year.rescale(LAST_DATE_SCALE);
        balance.rescale(2);

        let a_ = column![text_cell(format!("{currency} Total:"))];
        let b_ = column![number_cell(last_week)];
        let c_ = column![number_cell(last_month)];
        let d_ = column![number_cell(last_year)];
        let e_ = column![number_cell(balance)];
        let f_ = column![text_cell("")];

        TotalsColumnDisplay {
            a: a_,
            b: b_,
            c: c_,
            d: d_,
            e: e_,
            f: f_,
        }
    }

    #[rustfmt::skip]
    fn rows(&self) -> Row<Message> {
        let mut col_0 = column![text_cell(" Account "), text_cell("")];
        let mut col_1 = column![button_cell(button("Week").on_press(Message::ChartWeek)), text_cell("")].align_x(Alignment::End);
        let mut col_2 = column![button_cell(button("Month").on_press(Message::ChartMonth)), text_cell("")].align_x(Alignment::End);
        let mut col_3 = column![button_cell(button("Year").on_press(Message::ChartYear)), text_cell("")].align_x(Alignment::End);
        let mut col_4 = column![button_cell(button("Balance").on_press(Message::ChartAll)), text_cell("")].align_x(Alignment::End);
        let mut col_5 = column![text_cell("Price"), text_cell("")].align_x(Alignment::End);
        let mut col_6 = column![text_cell("Quantity"), text_cell("")].align_x(Alignment::End);
        let mut col_7 = column![Checkbox::new("", false), Checkbox::new("", false)].spacing(CHECKBOX_SPACING);
        let mut col_8 = column![text_cell(""), text_cell("")].spacing(COLUMN_SPACING);
        let mut col_9 = column![text_cell(""), text_cell("")].spacing(COLUMN_SPACING);
        let mut col_a = column![text_cell(""), text_cell("")].spacing(COLUMN_SPACING);
        let mut col_b = column![text_cell(""), text_cell("")].spacing(COLUMN_SPACING);
        let mut col_c = column![text_cell(""), text_cell("")].spacing(COLUMN_SPACING);
        let mut col_d = column![text_cell(""), text_cell("")].spacing(COLUMN_SPACING);

        for (i, account) in self.accounts.inner.iter().enumerate() {
            let (before_last_week, mut last_week) = account.sum_last_week();
            let (before_last_month, mut last_month) = account.sum_last_month();
            let (before_last_year, mut last_year) = account.sum_last_year();
            let mut value = account.balance_1st();

            let mut quantity = text_cell("");
            let mut price = text_cell("");
            if let Some(mut quantity_) = account.balance_2nd() {
                let mut price_ = value / quantity_;

                quantity_.rescale(8);
                price_.rescale(2);
                quantity = number_cell(quantity_);
                price = number_cell(price_);
            }

            last_week = div_0_ok(last_week, before_last_week);
            last_month = div_0_ok(last_month, before_last_month);
            last_year = div_0_ok(last_year, before_last_year);

            last_week.rescale(LAST_DATE_SCALE);
            last_month.rescale(LAST_DATE_SCALE);
            last_year.rescale(LAST_DATE_SCALE);
            value.rescale(2);

            col_0 = col_0.push(text_cell(&account.name));
            col_1 = col_1.push(number_cell(last_week));
            col_2 = col_2.push(number_cell(last_month));
            col_3 = col_3.push(number_cell(last_year));
            col_4 = col_4.push(number_cell(value));
            col_5 = col_5.push(price);
            col_6 = col_6.push(quantity);
            col_7 = col_7.push(Checkbox::new("", self.accounts[i].check_box).on_toggle(move |b| Message::Checkbox((i, b))));
            col_8 = col_8.push(button_cell(button("Tx").on_press(Message::SelectAccount(i))));
            let mut txs_2nd = button("Tx 2nd");
            if let Some(account) = &account.txs_2nd {
                if account.has_txs_2nd() {
                    txs_2nd = txs_2nd.on_press(Message::SelectAccountSecondary(i));
                }
            }
            col_9 = col_9.push(button_cell(txs_2nd));
            let mut update_name = button("Update Name");
            if !self.account_name.is_empty() {
                update_name = update_name.on_press(Message::UpdateAccountName(i));
            }
            col_a = col_a.push(button_cell(update_name));
            let mut import_boa = button("Import BoA");
            if account.txs_2nd.is_none() {
                import_boa = import_boa.on_press(Message::ImportBoa(i));
            }
            col_b = col_b.push(button_cell(import_boa));
            let mut get_price = button("Get Price");
            if account.txs_2nd.is_some() {
                get_price = get_price.on_press(Message::GetPrice(i));
            }
            col_c = col_c.push(button_cell(get_price));
            col_d = col_d.push(button_cell(button("Delete").on_press(Message::Delete(i))));
        }

        for currency in self.accounts.currencies() {
            let totals_display = self.display_totals(&currency);
            col_0 = col_0.push(totals_display.a);
            col_1 = col_1.push(totals_display.b);
            col_2 = col_2.push(totals_display.c);
            col_3 = col_3.push(totals_display.d);
            col_4 = col_4.push(totals_display.e);
            col_d = col_d.push(totals_display.f);
        }

        let group_display = self.display_groups();
        col_0 = col_0.push(group_display.a);
        col_1 = col_1.push(group_display.b);
        col_2 = col_2.push(group_display.c);
        col_3 = col_3.push(group_display.d);
        col_4 = col_4.push(group_display.e);
        col_d = col_d.push(group_display.f);

        row![col_0, col_1, col_2, col_3, col_4, col_5, col_6, col_7, col_8, col_9, col_a, col_b, col_c, col_d]
    }

    #[rustfmt::skip]
    fn list_accounts(&self) -> Scrollable<Message> {
        let mut charts = Column::new();
        for currency in self.accounts.currencies() {
            let chart = Chart {
                txs: self.accounts.all_accounts_txs_1st(currency),
                duration: self.duration.clone(),
            };
            let chart = ChartWidget::new(chart).height(Length::Fixed(400.0));
            charts = charts.push(chart);
        }

        let rows = self.rows();

        let mut column_errors = Column::new();
        if let Some(errors) = &self.errors {
            for error in errors.iter() {
                column_errors = column_errors.push(text_cell_red(error.to_string()));
            }
        }

        let name = text_input("Name", &self.account_name)
            .on_input(Message::ChangeAccountName)
            .on_paste(Message::ChangeAccountName);

        let currency = ComboBox::new(&self.currency_selector, "Currency", self.currency.as_ref(), Message::UpdateCurrency);

        let mut add = button("Add");
        if !self.account_name.is_empty() && self.currency.is_some() {
            add = add.on_press(Message::SubmitAccount);
        }

        let mut add_group = button("Add Group");
        if !self.account_name.is_empty() {
            add_group = add_group.on_press(Message::AddGroup);
        }

        let mut all_prices = row![button_cell(button("Get All Prices").on_press(Message::GetPriceAll))];
        if let Some(progress) = self.progress_bar {
            all_prices = all_prices.push(ProgressBar::<Theme>::new(0.0..=100.0, progress));
        }
        all_prices = all_prices.push(widget::text(" ".repeat(EDGE_PADDING)));

        let cols = column![
            charts,
            rows.spacing(ROW_SPACING),
            column_errors,
            text_cell(""),
            row![
                widget::text("Account").size(TEXT_SIZE),
                name,
                currency,
                add,
                add_group,
                widget::text(" ".repeat(EDGE_PADDING)),

            ].padding(PADDING).spacing(ROW_SPACING),
            all_prices,
            row![
                button_cell(button("Open BoA URL").on_press(Message::OpenBoaUrl)),
                button_cell(button("Open Investor 360 URL").on_press(Message::OpenInvestor360Url)),
                button_cell(button("Import Investor 360").on_press(Message::ImportInvestor360)),
            ].padding(PADDING),
            row![
                button_cell(button("Exit").on_press(Message::Exit)),
                button_cell(button("Load").on_press(Message::FileLoad)),
                button_cell(button("Save As").on_press(Message::FileSaveAs)),
                button_cell(button("Configuration").on_press(Message::Configuration)),
            ]
        ];

        Scrollable::new(cols)
    }

    fn select_account(&mut self, message: account::Message) {
        if let Some(account) = match self.screen {
            Screen::Accounts | Screen::Configuration => None,
            Screen::Account(account) | Screen::AccountSecondary(account) => Some(account),
        } {
            if self.accounts[account].update(&self.screen, message) {
                self.save();
            }
        }
    }

    fn insert_new_account(&mut self, new_account: Account) {
        if self.accounts.inner.is_empty() {
            self.accounts.inner.push(new_account);
            return;
        }

        for (i, account) in self.accounts.inner.iter().enumerate() {
            if account.name > new_account.name {
                self.accounts.inner.insert(i, new_account);
                for group in &mut self.accounts.groups {
                    for index in &mut group.members {
                        if *index > i {
                            *index += 1;
                        }
                    }
                }
                return;
            }
        }

        self.accounts.inner.push(new_account);
    }

    fn remove_account(&mut self, index: usize) -> Account {
        for group in &mut self.accounts.groups {
            group.members.remove(index);
        }
        self.accounts.inner.remove(index)
    }

    fn submit_account(&mut self) {
        let name = self.account_name.trim().to_string();
        if let Err(error) = self.check_account_name(&name) {
            self.display_error(error);
            return;
        }

        let new_account = Account::new(name, self.currency.clone().unwrap());

        self.insert_new_account(new_account);
        self.save();
    }

    pub fn theme(&self) -> Theme {
        Theme::SolarizedLight
    }

    fn update_account_name(&mut self, i: usize) {
        let name = self.account_name.trim().to_string();
        if let Err(error) = self.check_account_name(&name) {
            self.display_error(error);
            return;
        }

        let mut account = self.remove_account(i);
        account.name = self.account_name.trim().to_string();
        // add the new group

        self.insert_new_account(account);
        self.save();
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        self.errors = None;

        match message {
            Message::AddCrypto => self.add_crypto(),
            Message::AddFiat => self.add_fiat(),
            Message::AddGroup => self.add_group(),
            Message::AddMetal => self.add_metal(),
            Message::AddStockPlus => self.add_stock_plus(),
            Message::Account(message) => self.select_account(message),
            Message::Back => self.screen = Screen::Accounts,
            Message::ChangeAccountName(name) => self.account_name = name,
            Message::ChangeProjectMonths(months) => self.change_project_months(&months),
            Message::ChartWeek => self.duration = Duration::Week,
            Message::ChartMonth => self.duration = Duration::Month,
            Message::ChartYear => self.duration = Duration::Year,
            Message::ChartAll => self.duration = Duration::All,
            Message::Checkbox((i, b)) => self.accounts[i].check_box = b,
            Message::Configuration => self.screen = Screen::Configuration,
            Message::Delete(i) => self.delete(i),
            Message::DeleteGroup(i) => self.delete_group(i),
            Message::FileLoad => self.load_file(),
            Message::FileSaveAs => self.save_file(),
            Message::GetPrice(i) => {
                let account = &mut self.accounts[i];

                match futures::executor::block_on(account.submit_price_as_transaction()) {
                    Ok(tx) => {
                        account.txs_1st.txs.push(tx);
                        account.txs_1st.sort();
                        self.save();
                    }
                    Err(error) => {
                        self.display_error(error);
                    }
                }
            }
            Message::GetPriceAll => {
                let errors = futures::executor::block_on(self.accounts.get_all_prices());
                if !errors.is_empty() {
                    self.errors = Some(Arc::new(errors));
                }
                self.save();
            }
            Message::ImportBoa(i) => {
                let account = &mut self.accounts[i];

                if let Some(file_path) = rfd::FileDialog::new()
                    .set_title(TITLE_FILE_PICKER)
                    .add_filter("csv", &["csv"])
                    .pick_file()
                {
                    if let Err(error) = account.import_boa(file_path) {
                        self.display_error(error);
                    } else {
                        self.save();
                    }
                    self.screen = Screen::Accounts;
                }
            }
            Message::ImportInvestor360 => {
                if let Some(file_path) = rfd::FileDialog::new()
                    .set_title(TITLE_FILE_PICKER)
                    .add_filter("xls", &["xls"])
                    .pick_file()
                {
                    if let Err(error) = self.import_investor_360(&file_path) {
                        self.display_error(error);
                    } else {
                        self.accounts.sort();
                        self.save();
                    }
                }
            }
            Message::OpenBoaUrl => self.open_url(BOA_URL),
            Message::OpenInvestor360Url => self.open_url(INVESTOR_360_URL),
            Message::UpdateAccountName(i) => self.update_account_name(i),
            Message::UpdateCurrency(currency) => self.currency = Some(currency),
            Message::UpdateCryptoCurrency(fiat) => self.crypto_currency = Some(fiat),
            Message::UpdateCryptoDescription(description) => self.crypto_description = description,
            Message::UpdateCryptoSymbol(symbol) => self.crypto_symbol = symbol,
            Message::UpdateFiat(fiat) => self.fiat = Some(fiat),
            Message::UpdateMetalCurrency(fiat) => self.metal_currency = Some(fiat),
            Message::UpdateMetalDescription(description) => self.metal_description = description,
            Message::UpdateMetalSymbol(symbol) => self.metal_symbol = symbol,
            Message::UpdateStockPlusDescription(description) => {
                self.stock_plus_description = description;
            }
            Message::UpdateStockPlusSymbol(symbol) => self.stock_plus_symbol = symbol,
            Message::SelectAccount(i) => self.screen = Screen::Account(i),
            Message::SelectAccountSecondary(i) => self.screen = Screen::AccountSecondary(i),
            Message::SubmitAccount => self.submit_account(),
            Message::Exit => {
                return iced::exit();
            }
        }

        Task::none()
    }

    pub fn view(&self) -> Element<Message> {
        match self.screen {
            Screen::Accounts => self.list_accounts().into(),
            Screen::Account(i) => self.accounts[i].list_transactions().into(),
            Screen::AccountSecondary(i) => self.accounts[i].list_transactions_2nd().into(),
            Screen::Configuration => self.config().into(),
        }
    }
}

impl Default for App {
    fn default() -> Self {
        match command_line::get_configuration_file() {
            command_line::File::Load(file_path) => {
                let file_path_ = file_path.clone();
                let (accounts, file) = Accounts::load(None, file_path).unwrap_or_else(|err| {
                    panic!("error loading {}: {}", &file_path_.display(), err)
                });
                Self::new(accounts, Some(file))
            }
            command_line::File::New(file_path) => {
                let accounts = Accounts::new();
                let file_path_ = file_path.clone();
                let file = accounts.save_first(file_path).unwrap_or_else(|error| {
                    panic!("error creating {}: {}", &file_path_.display(), error)
                });

                Self::new(accounts, Some(file))
            }
            command_line::File::None => Self::new(Accounts::new(), None),
        }
    }
}

struct GroupColumnDisplay<'a> {
    a: Column<'a, Message>,
    b: Column<'a, Message>,
    c: Column<'a, Message>,
    d: Column<'a, Message>,
    e: Column<'a, Message>,
    f: Column<'a, Message>,
}

struct TotalsColumnDisplay<'a> {
    a: Column<'a, Message>,
    b: Column<'a, Message>,
    c: Column<'a, Message>,
    d: Column<'a, Message>,
    e: Column<'a, Message>,
    f: Column<'a, Message>,
}

fn div_0_ok(dividend: Decimal, divisor: Decimal) -> Decimal {
    if divisor.is_zero() {
        dec!(0)
    } else {
        dividend / divisor
    }
}

fn some_or_empty<T: ToString>(value: Option<&T>) -> String {
    value.map_or_else(String::new, ToString::to_string)
}

fn set_amount(amount: &mut Option<Decimal>, string: &str) {
    if string.is_empty() {
        *amount = None;
    } else if let Ok(amount_) = string.parse() {
        *amount = Some(amount_);
    }
}

fn button_cell(button: Button<Message>) -> Row<Message> {
    row![button].padding(PADDING)
}

fn number_cell<'a>(num: Decimal) -> Row<'a, Message> {
    let text = match num.cmp(&dec!(0)) {
        Ordering::Greater => widget::text(num.separate_with_commas()).color(solarized::green()),
        Ordering::Less => widget::text(num.separate_with_commas()).color(solarized::red()),
        Ordering::Equal => widget::text(num.separate_with_commas()),
    };

    row![text.size(TEXT_SIZE)].padding(PADDING)
}

fn text_cell<'a>(s: impl ToString + IntoFragment<'a>) -> Row<'a, Message> {
    row![widget::text(s).size(TEXT_SIZE)].padding(PADDING)
}

fn text_cell_red<'a>(s: impl ToString + IntoFragment<'a>) -> Row<'a, Message> {
    row![widget::text(s).color(solarized::red()).size(TEXT_SIZE)].padding(PADDING)
}

#[derive(Clone, Debug, Default)]
enum Duration {
    Week,
    Month,
    Year,
    #[default]
    All,
}

#[derive(Debug)]
struct File {
    path: PathBuf,
    inner: fs::File,
}

#[derive(Debug, Deserialize)]
struct Investor360 {
    #[serde(rename = "Description")]
    description: String,
    #[serde(rename = "Symbol")]
    symbol: String,
    #[serde(rename = "Quantity")]
    quantity: Option<Decimal>,
    #[serde(rename = "Price ($)")]
    price: Option<Decimal>,
    #[serde(rename = "Value ($)")]
    _value: Option<Decimal>,
    #[serde(rename = "Assets (%)")]
    _assets: Option<Decimal>,
}
