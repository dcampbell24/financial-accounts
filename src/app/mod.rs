mod account;
mod accounts;
mod file_picker;
mod message;
mod screen;

use std::{mem, path::PathBuf};

use iced::{
    event, executor,
    keyboard::{self, Key, Modifiers},
    widget::{button, column, row, text, text_input, Button, Row, Scrollable},
    Alignment, Application, Command, Element, Event, Theme,
};
use rust_decimal::Decimal;
use thousands::Separable;

use crate::app::{
    account::transaction::TransactionToSubmit, account::Account, accounts::Accounts,
    file_picker::FilePicker, message::Message, screen::Screen,
};

const PADDING: u16 = 1;
const EDGE_PADDING: usize = 4;
const TEXT_SIZE: u16 = 24;

/// The fin-stat application.
#[derive(Clone, Debug)]
pub struct App {
    accounts: Accounts,
    file_path: PathBuf,
    file_picker: FilePicker,
    account_name: String,
    project_months: Option<u16>,
    screen: Screen,
}

impl App {
    fn new(accounts: Accounts, file_path: PathBuf, screen: Screen) -> Self {
        App {
            accounts,
            file_path,
            file_picker: FilePicker::new(),
            account_name: String::new(),
            project_months: None,
            screen,
        }
    }

    fn new_(&mut self, accounts: Accounts, file_path: PathBuf, screen: Screen) {
        self.accounts = accounts;
        self.file_path = file_path;
        self.screen = screen;
    }

    #[rustfmt::skip]
    fn list_accounts(&self) -> Scrollable<Message> {
        let mut col_0 = column![text_cell(" Account ")];
        let mut col_1 = column![text_cell(" Current Month ")].align_items(Alignment::End);
        let mut col_2 = column![text_cell(" Last Month ")].align_items(Alignment::End);
        let mut col_3 = column![text_cell(" Current Year ")].align_items(Alignment::End);
        let mut col_4 = column![text_cell(" Last Year ")].align_items(Alignment::End);
        let mut col_5 = column![text_cell(" Balance ")].align_items(Alignment::End);
        let mut col_6 = column![text_cell("")];
        let mut col_7 = column![text_cell("")];
        let mut col_8 = column![text_cell("")];
        let mut col_9 = column![text_cell("")];

        col_0 = col_0.push(text_cell(""));
        col_1 = col_1.push(text_cell(""));
        col_2 = col_2.push(text_cell(""));
        col_3 = col_3.push(text_cell(""));
        col_4 = col_4.push(text_cell(""));
        col_5 = col_5.push(text_cell(""));
        col_6 = col_6.push(text_cell(""));
        col_7 = col_7.push(text_cell(""));
        col_8 = col_8.push(text_cell(""));
        col_9 = col_9.push(text_cell(""));

        for (i, account) in self.accounts.inner.iter().enumerate() {
            let total = account.sum();
            let current_month = account.sum_current_month();
            let last_month = account.sum_last_month();
            let current_year = account.sum_current_year();
            let last_year = account.sum_last_year();
            col_0 = col_0.push(text_cell(&account.name));
            col_1 = col_1.push(text_cell(current_month.separate_with_commas()));
            col_2 = col_2.push(text_cell(last_month.separate_with_commas()));
            col_3 = col_3.push(text_cell(current_year.separate_with_commas()));
            col_4 = col_4.push(text_cell(last_year.separate_with_commas()));
            col_5 = col_5.push(text_cell(total.separate_with_commas()).padding(PADDING));
            col_6 = col_6.push(button_cell(button("Tx").on_press(Message::SelectAccount(i))));
            col_7 = col_7.push(button_cell(button("Monthly Tx").on_press(Message::SelectMonthly(i))));
            let mut update_name = button("Update Name");
            if !self.account_name.is_empty() {
                update_name = update_name.on_press(Message::UpdateAccount(i));
            }
            col_8 = col_8.push(row![update_name].padding(PADDING));
            col_9 = col_9.push(row![button("Delete").on_press(Message::Delete(i))].padding(PADDING));
        }
        let rows = row![col_0, col_1, col_2, col_3, col_4, col_5, col_6, col_7, col_8, col_9];

        let col_1 = column![
            text_cell("total current month: "),
            text_cell("total last month: "),
            text_cell("total current year: "),
            text_cell("total last year: "),
            text_cell("total: "),
        ];
        let col_2 = column![
            text_cell(self.accounts.total_for_current_month().separate_with_commas()),
            text_cell(self.accounts.total_for_last_month().separate_with_commas()),
            text_cell(self.accounts.total_for_current_year().separate_with_commas()),
            text_cell(self.accounts.total_for_last_year().separate_with_commas()),
            text_cell(self.accounts.total().separate_with_commas()),
        ].align_items(Alignment::End);
        let totals = row![col_1, col_2];

        let mut name = text_input("Name", &self.account_name)
            .on_input(Message::ChangeAccountName);
        if !self.account_name.is_empty() {
            name = name.on_submit(Message::NewAccount);
        }

        let mut months = match self.project_months {
            Some(months) => text_input("Months", &months.to_string()),
            None => text_input("Months", ""),
        };
        months = months.on_input(Message::ChangeProjectMonths);

        let cols = column![
            rows,
            text_cell(""),
            totals,
            text_cell(""),
            row![
                text("Account ").size(TEXT_SIZE),
                name,
                text(" ".repeat(EDGE_PADDING)),
            ].padding(PADDING),
            row![
                text("Project ").size(TEXT_SIZE),
                months,
                text((self.accounts.project_months(self.project_months)).separate_with_commas()).size(TEXT_SIZE),
                text(" ".repeat(EDGE_PADDING)),
            ].padding(PADDING),
            // text_(format!("Checked Up To: {}", self.checked_up_to.to_string())).size(TEXT_SIZE),
        ];

        Scrollable::new(cols)
    }

    fn selected_account(&self) -> Option<usize> {
        match self.screen {
            Screen::NewOrLoadFile | Screen::Accounts => None,
            Screen::Account(account) | Screen::Monthly(account) => Some(account),
        }
    }

    fn list_monthly(&self) -> bool {
        match self.screen {
            Screen::NewOrLoadFile | Screen::Accounts | Screen::Account(_) => false,
            Screen::Monthly(_) => true,
        }
    }
}

impl Application for App {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        match FilePicker::load_or_new_file() {
            Some((accounts, path_buf)) => (
                App::new(accounts, path_buf, Screen::Accounts),
                Command::none(),
            ),
            None => (
                App::new(Accounts::new(), PathBuf::new(), Screen::NewOrLoadFile),
                Command::none(),
            ),
        }
    }

    fn title(&self) -> String {
        String::from("Fin Stat")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        let list_monthly = self.list_monthly();
        let selected_account = self.selected_account();

        match message {
            Message::NewFile(file) => {
                if let Some((accounts, file_path)) = self.file_picker.new_file(file) {
                    self.new_(accounts, file_path, Screen::Accounts);
                }
            }
            Message::LoadFile(file_path) => {
                if let Some(accounts) = self.file_picker.load_file(&file_path) {
                    self.new_(accounts, file_path, Screen::Accounts);
                }
            }
            Message::ChangeDir(path) => self.file_picker.change_dir(path),
            Message::ChangeFileName(file) => self.file_picker.change_file_name(&file),
            Message::HiddenFilesToggle => self.file_picker.show_hidden_files_toggle(),
            Message::Back => self.screen = Screen::Accounts,
            Message::ChangeAccountName(name) => self.account_name = name.trim().to_string(),
            Message::ChangeTx(tx) => {
                let account = &mut self.accounts[selected_account.unwrap()];
                if list_monthly {
                    set_amount(&mut account.tx_monthly.amount, &tx);
                } else {
                    set_amount(&mut account.tx.amount, &tx);
                }
            }
            Message::ChangeDate(date) => self.accounts[selected_account.unwrap()].tx.date = date,
            Message::ChangeComment(comment) => {
                let account = &mut self.accounts[selected_account.unwrap()];
                if list_monthly {
                    account.tx_monthly.comment = comment.trim().to_string();
                } else {
                    account.tx.comment = comment.trim().to_string();
                }
            }
            Message::ChangeFilterDateYear(date) => {
                if date.is_empty() {
                    self.accounts[selected_account.unwrap()].filter_date_year = None;
                }
                if let Ok(date) = date.parse() {
                    if (0..3_000).contains(&date) {
                        self.accounts[selected_account.unwrap()].filter_date_year = Some(date)
                    }
                }
            }
            Message::ChangeFilterDateMonth(date) => {
                if date.is_empty() {
                    self.accounts[selected_account.unwrap()].filter_date_month = None;
                }
                if let Ok(date) = date.parse() {
                    if (1..13).contains(&date) {
                        self.accounts[selected_account.unwrap()].filter_date_month = Some(date)
                    }
                }
            }
            Message::ChangeProjectMonths(months) => {
                if months.is_empty() {
                    self.project_months = None;
                }
                if let Ok(months) = months.parse() {
                    self.project_months = Some(months);
                }
            }
            Message::Delete(i) => match self.screen {
                Screen::NewOrLoadFile => {
                    panic!("Screen::NewOrLoadFile can't be reached here");
                }
                Screen::Accounts => {
                    self.accounts.inner.remove(i);
                    self.accounts.save(&self.file_path);
                }
                Screen::Account(j) => {
                    self.accounts[j].data.remove(i);
                    self.accounts.save(&self.file_path);
                }
                Screen::Monthly(j) => {
                    self.accounts[j].monthly.remove(i);
                    self.accounts.save(&self.file_path);
                }
            },
            Message::NewAccount => {
                self.accounts
                    .inner
                    .push(Account::new(mem::take(&mut self.account_name)));
                self.accounts.save(&self.file_path);
            }
            Message::UpdateAccount(i) => {
                self.accounts[i].name = mem::take(&mut self.account_name);
                self.accounts.save(&self.file_path);
            }
            Message::SelectAccount(i) => self.screen = Screen::Account(i),
            Message::SelectMonthly(i) => self.screen = Screen::Monthly(i),
            Message::SubmitTx => {
                let account = &mut self.accounts[selected_account.unwrap()];

                if list_monthly {
                    account.submit_tx_monthly();
                } else {
                    match account.submit_tx() {
                        Ok(tx) => {
                            account.data.push(tx);
                            account.data.sort_by_key(|tx| tx.date);
                            account.error_str = String::new();
                            account.tx = TransactionToSubmit::new();
                            self.accounts.save(&self.file_path);
                        }
                        Err(err) => {
                            account.error_str = err;
                        }
                    }
                }
            }
            Message::SubmitFilterDate => {
                let account = &mut self.accounts[selected_account.unwrap()];
                account.filter_date = account.submit_filter_date();
                account.error_str = String::new();
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        match self.screen {
            Screen::NewOrLoadFile => self.file_picker.view().into(),
            Screen::Accounts => self.list_accounts().into(),
            Screen::Account(i) => self.accounts[i].list_transactions().into(),
            Screen::Monthly(i) => self.accounts[i].list_monthly().into(),
        }
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        event::listen_with(|event, _status| {
            let mut subscription = None;
            if let Event::Keyboard(keyboard::Event::KeyPressed {
                key,
                location: _location,
                modifiers,
                text: _text,
            }) = event
            {
                if key == Key::Character("h".into()) && modifiers == Modifiers::CTRL {
                    subscription = Some(Message::HiddenFilesToggle);
                }
            }
            subscription
        })
    }
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

fn text_cell<'a>(s: impl ToString) -> Row<'a, Message> {
    row![text(s).size(TEXT_SIZE)].padding(PADDING)
}
