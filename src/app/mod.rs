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
    executor, theme,
    widget::{
        button, column,
        combo_box::{ComboBox, State},
        row, text, text_input, Button, Checkbox, Column, Row, Scrollable,
    },
    window, Alignment, Application, Element, Length, Theme,
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
            fiats_current = fiats_current.push(text_cell(fiat));
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
                column_errors = column_errors.push(text_cell_red(error));
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
            stock_plus_description: String::new(),
            stock_plus_symbol: String::new(),
            currency: None,
            currency_selector: State::new(currencies),
            duration: Duration::All,
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

    fn check_monthly(&mut self) {
        self.accounts.check_monthly();
        self.save();
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
            Screen::Monthly(j) => {
                self.accounts[j].txs_monthly.remove(i);
            }
        };
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
                "\"{:?}\" already exists!",
                &file_csv
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
    fn rows(&self) -> Row<Message> {
        let mut col_0 = column![text_cell(" Account "), text_cell("")];
        let mut col_1 = column![button_cell(button("Week").on_press(Message::ChartWeek)), text_cell("")].align_items(Alignment::End);
        let mut col_2 = column![button_cell(button("Month").on_press(Message::ChartMonth)), text_cell("")].align_items(Alignment::End);
        let mut col_3 = column![button_cell(button("Year").on_press(Message::ChartYear)), text_cell("")].align_items(Alignment::End);
        let mut col_4 = column![button_cell(button("Balance").on_press(Message::ChartAll)), text_cell("")].align_items(Alignment::End);
        let mut col_5 = column![text_cell("Quantity"), text_cell("")].align_items(Alignment::End);
        let mut col_6 = column![Checkbox::new("", false), Checkbox::new("", false)].spacing(CHECKBOX_SPACING);
        let mut col_7 = column![text_cell(""), text_cell("")].spacing(COLUMN_SPACING);
        let mut col_8 = column![text_cell(""), text_cell("")].spacing(COLUMN_SPACING);
        let mut col_9 = column![text_cell(""), text_cell("")].spacing(COLUMN_SPACING);
        let mut col_a = column![text_cell(""), text_cell("")].spacing(COLUMN_SPACING);
        let mut col_b = column![text_cell(""), text_cell("")].spacing(COLUMN_SPACING);
        let mut col_c = column![text_cell(""), text_cell("")].spacing(COLUMN_SPACING);
        let mut col_d = column![text_cell(""), text_cell("")].spacing(COLUMN_SPACING);

        for (i, account) in self.accounts.inner.iter().enumerate() {
            let mut last_week = account.sum_last_week();
            let mut last_month = account.sum_last_month();
            let mut last_year = account.sum_last_year();
            let mut balance_1st = account.balance_1st();

            let balance_2nd = account.balance_2nd().map_or_else(|| text_cell(""), |mut balance| {
                balance.rescale(8);
                number_cell(balance)
            });

            last_week.rescale(2);
            last_month.rescale(2);
            last_year.rescale(2);
            balance_1st.rescale(2);

            col_0 = col_0.push(text_cell(&account.name));
            col_1 = col_1.push(number_cell(last_week));
            col_2 = col_2.push(number_cell(last_month));
            col_3 = col_3.push(number_cell(last_year));
            col_4 = col_4.push(number_cell(balance_1st));
            col_5 = col_5.push(balance_2nd);
            col_6 = col_6.push(Checkbox::new("", self.accounts[i].check_box).on_toggle(move |b| Message::Checkbox((i, b))));
            col_7 = col_7.push(button_cell(button("Tx").on_press(Message::SelectAccount(i))));
            let mut txs_2nd = button("Tx 2nd");
            if let Some(account) = &account.txs_2nd {
                if account.has_txs_2nd() {
                    txs_2nd = txs_2nd.on_press(Message::SelectAccountSecondary(i));
                }
            }
            col_8 = col_8.push(button_cell(txs_2nd));
            col_9 = col_9.push(button_cell(button("Monthly Tx").on_press(Message::SelectMonthly(i))));
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

        let mut total_for_last_week_usd = self.accounts.total_for_last_week_usd();
        let mut total_for_last_month_usd = self.accounts.total_for_last_month_usd();
        let mut total_for_last_year_usd = self.accounts.total_for_last_year_usd();
        let mut balance = self.accounts.balance_usd();

        total_for_last_week_usd.rescale(2);
        total_for_last_month_usd.rescale(2);
        total_for_last_year_usd.rescale(2);
        balance.rescale(2);

        col_0 = col_0.push(text_cell("Total"));
        col_1 = col_1.push(number_cell(total_for_last_week_usd));
        col_2 = col_2.push(number_cell(total_for_last_month_usd));
        col_3 = col_3.push(number_cell(total_for_last_year_usd));
        col_4 = col_4.push(number_cell(balance));
        col_d = col_d.push(text_cell(""));

        for (index, group) in self.accounts.groups.iter().enumerate() {
            col_0 = col_0.push(text_cell(&group.name));
            let mut week = dec!(0);
            let mut month = dec!(0);
            let mut year = dec!(0);
            let mut balance = dec!(0);
            for index in &group.members {
                week += self.accounts.inner[*index].sum_last_week();
                month += self.accounts.inner[*index].sum_last_month();
                year += self.accounts.inner[*index].sum_last_year();
                balance += self.accounts.inner[*index].balance_1st();
            }

            week.rescale(2);
            month.rescale(2);
            year.rescale(2);
            balance.rescale(2);

            col_1 = col_1.push(number_cell(week));
            col_2 = col_2.push(number_cell(month));
            col_3 = col_3.push(number_cell(year));
            col_4 = col_4.push(number_cell(balance));
            col_d = col_d.push(button_cell(button("Delete").on_press(Message::DeleteGroup(index))));
        }

        row![col_0, col_1, col_2, col_3, col_4, col_5, col_6, col_7, col_8, col_9, col_a, col_b, col_c, col_d]
    }

    #[rustfmt::skip]
    fn list_accounts(&self) -> Scrollable<Message> {
        let chart = Chart {
            txs: self.accounts.all_accounts_txs_1st(),
            duration: self.duration.clone(),
        };
        let chart = ChartWidget::new(chart).height(Length::Fixed(400.0));
        let rows = self.rows();

        let mut column_errors = Column::new();
        if let Some(errors) = &self.errors {
            for error in errors.iter() {
                column_errors = column_errors.push(text_cell_red(error));
            }
        }

        let name = text_input("Name", &self.account_name)
            .on_input(Message::ChangeAccountName)
            .on_paste(Message::ChangeAccountName);

        let currency = ComboBox::new(&self.currency_selector, "Currency", self.currency.as_ref(), Message::UpdateCurrency);

        let months = text_input("Months", &some_or_empty(&self.project_months))
            .on_input(Message::ChangeProjectMonths);

        let mut add = button("Add");
        if !self.account_name.is_empty() && self.currency.is_some() {
            add = add.on_press(Message::SubmitAccount);
        }

        let mut add_group = button("Add Group");
        if !self.account_name.is_empty() {
            add_group = add_group.on_press(Message::AddGroup);
        }

        let cols = column![
            chart,
            rows.spacing(ROW_SPACING),
            column_errors,
            text_cell(""),
            row![
                text("Account").size(TEXT_SIZE),
                name,
                currency,
                add,
                add_group,
                text(" ".repeat(EDGE_PADDING)),

            ].padding(PADDING).spacing(ROW_SPACING),
            row![
                text("Project").size(TEXT_SIZE),
                months,
                text((self.accounts.project_months(self.project_months)).separate_with_commas()).size(TEXT_SIZE),
                text(" ".repeat(EDGE_PADDING)),
            ].padding(PADDING).spacing(ROW_SPACING),
            row![
                button_cell(button("Exit").on_press(Message::Exit)),
                button_cell(button("Load").on_press(Message::FileLoad)),
                button_cell(button("Save As").on_press(Message::FileSaveAs)),
                button_cell(button("Get All Prices").on_press(Message::GetPriceAll)),
                button_cell(button("Check Monthly").on_press(Message::CheckMonthly)),
                button_cell(button("Configuration").on_press(Message::Configuration)),
            ].padding(PADDING).spacing(ROW_SPACING),
            row![
                button_cell(button("Open BoA URL").on_press(Message::OpenBoaUrl)),
                button_cell(button("Open Investor 360 URL URL").on_press(Message::OpenInvestor360Url)),
                button_cell(button("Import Investor 360").on_press(Message::ImportInvestor360)),
            ]
            // text_(format!("Checked Up To: {}", self.checked_up_to.to_string())).size(TEXT_SIZE),
        ];

        Scrollable::new(cols)
    }

    fn select_account(&mut self, message: account::Message) {
        if let Some(account) = match self.screen {
            Screen::Accounts | Screen::Configuration => None,
            Screen::Account(account)
            | Screen::AccountSecondary(account)
            | Screen::Monthly(account) => Some(account),
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
}

impl Application for App {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Message>) {
        match command_line::get_configuration_file() {
            command_line::File::Load(file_path) => {
                let file_path_ = file_path.clone();
                let (accounts, file) = Accounts::load(None, file_path)
                    .unwrap_or_else(|err| panic!("error loading {:?}: {}", &file_path_, err));
                (
                    Self::new(accounts, Some(file)),
                    window::maximize(window::Id::MAIN, true),
                )
            }
            command_line::File::New(file_path) => {
                let accounts = Accounts::new();
                let file_path_ = file_path.clone();
                let file = accounts
                    .save_first(file_path)
                    .unwrap_or_else(|error| panic!("error creating {:?}: {}", &file_path_, error));

                (
                    Self::new(accounts, Some(file)),
                    window::maximize(window::Id::MAIN, true),
                )
            }
            command_line::File::None => (
                Self::new(Accounts::new(), None),
                window::maximize(window::Id::MAIN, true),
            ),
        }
    }

    fn theme(&self) -> Self::Theme {
        Theme::SolarizedLight
    }

    fn title(&self) -> String {
        String::from("Financial Accounts")
    }

    fn update(&mut self, message: Message) -> iced::Command<Message> {
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
            Message::CheckMonthly => self.check_monthly(),
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
            Message::SelectMonthly(i) => self.screen = Screen::Monthly(i),
            Message::SubmitAccount => self.submit_account(),
            Message::Exit => {
                return window::close(window::Id::MAIN);
            }
        }
        iced::Command::none()
    }

    fn view(&self) -> Element<Message> {
        match self.screen {
            Screen::Accounts => self.list_accounts().into(),
            Screen::Account(i) => {
                let account = &self.accounts[i];
                self.accounts[i]
                    .list_transactions(
                        account.txs_1st.clone(),
                        account.total_1st(),
                        account.balance_1st(),
                    )
                    .into()
            }
            Screen::AccountSecondary(i) => {
                let account = &self.accounts[i];
                let txs = account.txs_2nd.as_ref().unwrap();
                self.accounts[i]
                    .list_transactions(
                        txs.clone(),
                        account.total_2nd(),
                        account.balance_2nd().unwrap(),
                    )
                    .into()
            }
            Screen::Configuration => self.config().into(),
            Screen::Monthly(i) => self.accounts[i].list_monthly().into(),
        }
    }
}

fn some_or_empty<T: ToString>(value: &Option<T>) -> String {
    value
        .as_ref()
        .map_or_else(String::new, std::string::ToString::to_string)
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
        Ordering::Greater => {
            text(num.separate_with_commas()).style(theme::Text::Color(solarized::green()))
        }
        Ordering::Less => {
            text(num.separate_with_commas()).style(theme::Text::Color(solarized::red()))
        }
        Ordering::Equal => text(num.separate_with_commas()),
    };

    row![text.size(TEXT_SIZE)].padding(PADDING)
}

fn text_cell<'a>(s: impl ToString) -> Row<'a, Message> {
    row![text(s).size(TEXT_SIZE)].padding(PADDING)
}

fn text_cell_red<'a>(s: impl ToString) -> Row<'a, Message> {
    row![text(s)
        .style(theme::Text::Color(solarized::red()))
        .size(TEXT_SIZE)]
    .padding(PADDING)
}

#[derive(Clone, Debug)]
enum Duration {
    Week,
    Month,
    Year,
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
