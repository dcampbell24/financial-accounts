use std::{mem, path::PathBuf};

use chrono::{DateTime, Utc};
use clap::{command, Parser};
use iced::{
    widget::{button, column, row, text, text_input, Column},
    Alignment, Element, Sandbox,
};
use thousands::Separable;

use crate::{
    account::Account,
    accounts::{Accounts, Screen},
    file_picker::FilePicker,
    message::Message,
    transaction::{TransactionMonthly, TransactionToSubmit},
    PADDING, TEXT_SIZE,
};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Name of the file to load
    #[arg(long, value_name = "FILE", exclusive = true)]
    load: Option<String>,

    /// Name of the new file
    #[arg(long, value_name = "FILE", exclusive = true)]
    new: Option<String>,
}

#[derive(Clone, Debug)]
pub struct App {
    accounts: Accounts,
    error_str: String,
    file_path: PathBuf,
    file_picker: FilePicker,
    name: String,
    project_months: u64,
    project_months_str: String,
    screen: Screen,
}

impl App {
    pub fn new(accounts: Accounts, file_path: &PathBuf, screen: Screen) -> Self {
        App {
            accounts,
            error_str: String::new(),
            file_path: file_path.to_owned(),
            file_picker: FilePicker::new(),
            name: String::new(),
            project_months: 0,
            project_months_str: String::new(),
            screen,
        }
    }

    #[rustfmt::skip]
    pub fn list_accounts(&self) -> Column<Message> {
        let mut col_0 = column![text(" Account ").size(TEXT_SIZE)];
        let mut col_1 = column![text(" Current Month ").size(TEXT_SIZE)].align_items(Alignment::End);
        let mut col_2 = column![text(" Last Month ").size(TEXT_SIZE)].align_items(Alignment::End);
        let mut col_3 = column![text(" Current Year ").size(TEXT_SIZE)].align_items(Alignment::End);
        let mut col_4 = column![text(" Last Year ").size(TEXT_SIZE)].align_items(Alignment::End);
        let mut col_5 = column![text(" Balance ").size(TEXT_SIZE)].align_items(Alignment::End);
        let mut col_6 = column![text("").size(TEXT_SIZE)];
        let mut col_7 = column![text("").size(TEXT_SIZE)];
        let mut col_8 = column![text("").size(TEXT_SIZE)];
        let mut col_9 = column![text("").size(TEXT_SIZE)];

        for (i, account) in self.accounts.inner.iter().enumerate() {
            let total = account.sum();
            let current_month = account.sum_current_month();
            let last_month = account.sum_last_month();
            let current_year = account.sum_current_year();
            let last_year = account.sum_last_year();
            col_0 = col_0.push(row![text(&account.name).size(TEXT_SIZE)].padding(PADDING));
            col_1 = col_1.push(row![text(current_month.separate_with_commas()).size(TEXT_SIZE)].padding(PADDING));
            col_2 = col_2.push(row![text(last_month.separate_with_commas()).size(TEXT_SIZE)].padding(PADDING));
            col_3 = col_3.push(row![text(current_year.separate_with_commas()).size(TEXT_SIZE)].padding(PADDING));
            col_4 = col_4.push(row![text(last_year.separate_with_commas()).size(TEXT_SIZE)].padding(PADDING));
            col_5 = col_5.push(row![text(total.separate_with_commas()).size(TEXT_SIZE)].padding(PADDING));
            col_6 = col_6.push(row![button("Tx").on_press(Message::SelectAccount(i))].padding(PADDING));
            col_7 = col_7.push(row![button("Monthly Tx").on_press(Message::SelectMonthly(i))].padding(PADDING));
            col_8 = col_8.push(row![button("Update Name").on_press(Message::UpdateAccount(i))].padding(PADDING));
            col_9 = col_9.push(row![button("Delete").on_press(Message::Delete(i))].padding(PADDING));
        }
        let rows = row![col_0, col_1, col_2, col_3, col_4, col_5, col_6, col_7, col_8, col_9];

        let col_1 = column![
            text("total current month: ").size(TEXT_SIZE),
            text("total last month: ").size(TEXT_SIZE),
            text("total current year: ").size(TEXT_SIZE),
            text("total last year: ").size(TEXT_SIZE),
            text("total: ").size(TEXT_SIZE),
        ];
        let col_2 = column![
            text(self.accounts.total_for_current_month().separate_with_commas()).size(TEXT_SIZE),
            text(self.accounts.total_for_last_month().separate_with_commas()).size(TEXT_SIZE),
            text(self.accounts.total_for_current_year().separate_with_commas()).size(TEXT_SIZE),
            text(self.accounts.total_for_last_year().separate_with_commas()).size(TEXT_SIZE),
            text(self.accounts.total().separate_with_commas()).size(TEXT_SIZE),
        ].align_items(Alignment::End);
        let totals = row![col_1, col_2];

        let cols = column![
            rows,
            row![text("")],
            totals,
            row![text("")],
            row![
                text("Account ").size(TEXT_SIZE),
                text_input("Name", &self.name)
                    .on_submit(Message::NewAccount)
                    .on_input(Message::ChangeAccountName)
            ].padding(PADDING),
            row![
                text("Project ").size(TEXT_SIZE),
                text_input("Months", &self.project_months_str)
                    .on_input(Message::ChangeProjectMonths)
                    .on_submit(Message::ProjectMonths),
                text((self.accounts.total() + self.accounts.total_for_months(self.project_months)).separate_with_commas()).size(TEXT_SIZE),
            ].padding(PADDING),
            text(&self.error_str).size(TEXT_SIZE),
            // text(format!("Checked Up To: {}", self.checked_up_to.to_string())).size(TEXT_SIZE),
        ];
        cols
    }

    pub fn selected_account(&self) -> Option<usize> {
        match self.screen {
            Screen::NewOrLoadFile | Screen::Accounts => None,
            Screen::Account(account) | Screen::Monthly(account) => Some(account),
        }
    }

    pub fn list_monthly(&self) -> bool {
        match self.screen {
            Screen::NewOrLoadFile | Screen::Accounts | Screen::Account(_) => false,
            Screen::Monthly(_) => true,
        }
    }
}

impl Sandbox for App {
    type Message = Message;

    fn new() -> Self {
        let args = Args::parse();
        let screen = Screen::Accounts;

        if let Some(arg) = args.load {
            let path_buf = PathBuf::from(arg);
            let mut accounts = Accounts::load(&path_buf).unwrap();
            accounts.check_monthly();
            accounts.save(&path_buf);
            return App::new(accounts, &path_buf, screen);
        }
        if let Some(arg) = args.new {
            let path_buf = PathBuf::from(arg);
            let accounts = Accounts::empty_accounts();
            accounts.save_first(&path_buf);
            return App::new(accounts, &path_buf, screen);
        }

        let path_buf = PathBuf::new();
        let accounts = Accounts::empty_accounts();
        let screen = Screen::NewOrLoadFile;
        App::new(accounts, &path_buf, screen)
    }

    fn title(&self) -> String {
        String::from("Fin Stat")
    }

    fn update(&mut self, message: Message) {
        let list_monthly = self.list_monthly();
        let selected_account = self.selected_account();

        match message {
            Message::NewFile(mut file) => {
                file.set_extension("json");
                self.file_picker.current.push(file);
                let accounts = Accounts::empty_accounts();
                accounts.save_first(&self.file_picker.current);
                self.accounts = accounts;
                self.file_path = self.file_picker.current.clone();
                self.screen = Screen::Accounts;
            }
            Message::LoadFile(file) => {
                let accounts = Accounts::load(&file);
                match accounts {
                    Ok(mut accounts) => {
                        accounts.check_monthly();
                        accounts.save(&file);
                        self.accounts = accounts;
                        self.file_path = file;
                        self.screen = Screen::Accounts;
                    }
                    Err(err) => {
                        self.file_picker.error = format!("{:?}", err);
                    }
                }
            }
            Message::ChangeDir(path_buf) => {
                self.file_picker.current = path_buf;
                self.file_picker.error = String::new();
            }
            Message::ChangeFileName(file) => {
                self.file_picker.filename = file;
                self.file_picker.error = String::new();
            }
            Message::Back => self.screen = Screen::Accounts,
            Message::ChangeAccountName(name) => self.name = name,
            Message::ChangeTx(tx) => self.accounts[selected_account.unwrap()].tx.amount = tx,
            Message::ChangeDate(date) => self.accounts[selected_account.unwrap()].tx.date = date,
            Message::ChangeComment(comment) => {
                self.accounts[selected_account.unwrap()].tx.comment = comment;
            }
            Message::ChangeProjectMonths(i) => self.project_months_str = i,
            Message::ChangeFilterDateYear(date) => {
                self.accounts[selected_account.unwrap()].filter_date_year = date;
            }
            Message::ChangeFilterDateMonth(date) => {
                self.accounts[selected_account.unwrap()].filter_date_month = date;
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
                    .push(Account::new(mem::take(&mut self.name)));
                self.accounts.save(&self.file_path);
            }
            Message::UpdateAccount(i) => {
                self.accounts[i].name = mem::take(&mut self.name);
                self.accounts.save(&self.file_path);
            }
            Message::ProjectMonths => match self.project_months_str.parse() {
                Ok(i) => {
                    self.project_months = i;
                    self.error_str = String::new();
                }
                Err(err) => {
                    let mut msg = "Parse Project Months error: ".to_string();
                    msg.push_str(&err.to_string());
                    self.error_str = msg;
                }
            },
            Message::SelectAccount(i) => self.screen = Screen::Account(i),
            Message::SelectMonthly(i) => self.screen = Screen::Monthly(i),
            Message::SubmitTx => {
                let account = &mut self.accounts[selected_account.unwrap()];
                match account.submit_tx() {
                    Ok(tx) => {
                        if list_monthly {
                            account.monthly.push(TransactionMonthly::from(tx));
                        } else {
                            account.data.push(tx);
                            account.data.sort_by_key(|tx| tx.date);
                        }
                        account.error_str = String::new();
                        account.tx = TransactionToSubmit::new();
                        self.accounts.save(&self.file_path);
                    }
                    Err(err) => {
                        account.error_str = err;
                    }
                }
            }
            Message::SubmitFilterDate => {
                let account = &mut self.accounts[selected_account.unwrap()];
                match account.submit_filter_date() {
                    Ok(date) => {
                        account.filter_date = date;
                        account.error_str = String::new();
                    }
                    Err(err) => {
                        account.filter_date = DateTime::<Utc>::default();
                        account.error_str = err;
                    }
                }
            }
        }
    }

    fn view(&self) -> Element<Message> {
        match self.screen {
            Screen::NewOrLoadFile => self.file_picker.view().into(),
            Screen::Accounts => self.list_accounts().into(),
            Screen::Account(i) => self.accounts[i].list_transactions().into(),
            Screen::Monthly(i) => self.accounts[i].list_monthly().into(),
        }
    }
}
