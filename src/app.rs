mod accounts;
mod file_picker;
mod message;
mod screen;

use std::{mem, path::PathBuf};

use clap::{command, Parser};
use iced::{
    executor,
    keyboard::{self, KeyCode, Modifiers},
    subscription,
    widget::{button, column, row, text, text_input, Scrollable},
    Alignment, Application, Command, Element, Event, Theme,
};
use thousands::Separable;

use crate::app::{
    accounts::{account::Account, Accounts},
    accounts::account::transaction::{TransactionMonthly, TransactionToSubmit},
    file_picker::FilePicker,
    message::Message,
    screen::Screen,
};

const PADDING: u16 = 1;
const EDGE_PADDING: usize = 4;
const TEXT_SIZE: u16 = 24;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Name of the file to load
    #[arg(long, value_name = "FILE", exclusive = true)]
    load: Option<String>,

    /// Name of the new file
    #[arg(long, value_name = "FILE", exclusive = true)]
    new: Option<String>,
}

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
            let mut update_name = button("Update Name");
            if !self.account_name.is_empty() {
                update_name = update_name.on_press(Message::UpdateAccount(i));
            }
            col_8 = col_8.push(row![update_name].padding(PADDING));
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
            row![text("")],
            totals,
            row![text("")],
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
            // text(format!("Checked Up To: {}", self.checked_up_to.to_string())).size(TEXT_SIZE),
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
        let args = Args::parse();

        if let Some(arg) = args.load {
            let path_buf = PathBuf::from(arg);
            let mut accounts = Accounts::load(&path_buf)
                .unwrap_or_else(|err| panic!("error loading {:?}: {}", &path_buf, err));
            accounts.check_monthly();
            accounts.save(&path_buf);
            return (
                App::new(accounts, path_buf, Screen::Accounts),
                Command::none(),
            );
        }
        if let Some(arg) = args.new {
            let path_buf = PathBuf::from(arg);
            let accounts = Accounts::new();
            accounts
                .save_first(&path_buf)
                .unwrap_or_else(|err| panic!("error creating {:?}: {}", &path_buf, err));
            return (
                App::new(accounts, path_buf, Screen::Accounts),
                Command::none(),
            );
        }

        (
            App::new(Accounts::new(), PathBuf::new(), Screen::NewOrLoadFile),
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Fin Stat")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        let list_monthly = self.list_monthly();
        let selected_account = self.selected_account();

        match message {
            Message::NewFile(mut file) => {
                if file.as_os_str().is_empty() {
                    return Command::none();
                }
                let mut file_path = self.file_picker.current.clone();
                file.set_extension("json");
                file_path.push(file);
                let accounts = Accounts::new();
                if let Err(err) = accounts.save_first(&file_path) {
                    self.file_picker.error = format!("error creating {:?}: {}", &file_path, err);
                    return Command::none();
                }
                self.new_(accounts, file_path, Screen::Accounts);
            }
            Message::LoadFile(file_path) => match Accounts::load(&file_path) {
                Ok(mut accounts) => {
                    accounts.check_monthly();
                    accounts.save(&file_path);
                    self.new_(accounts, file_path, Screen::Accounts);
                }
                Err(err) => {
                    self.file_picker.error = format!("error loading {:?}: {}", &file_path, err);
                }
            },
            Message::ChangeDir(path_buf) => {
                self.file_picker.current = path_buf;
                self.file_picker.error = String::new();
            }
            Message::ChangeFileName(file) => {
                self.file_picker.filename = file.trim().to_string();
                self.file_picker.error = String::new();
            }
            Message::HiddenFilesToggle => {
                self.file_picker.show_hidden_files = !self.file_picker.show_hidden_files;
            }
            Message::Back => self.screen = Screen::Accounts,
            Message::ChangeAccountName(name) => self.account_name = name.trim().to_string(),
            Message::ChangeTx(tx) => {
                if tx.is_empty() {
                    self.accounts[selected_account.unwrap()].tx.amount = None;
                }
                if let Ok(amount) = tx.parse() {
                    self.accounts[selected_account.unwrap()].tx.amount = Some(amount);
                }
            }
            Message::ChangeDate(date) => self.accounts[selected_account.unwrap()].tx.date = date,
            Message::ChangeComment(comment) => {
                self.accounts[selected_account.unwrap()].tx.comment = comment.trim().to_string();
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
        subscription::events_with(|event, _status| {
            let mut subscription = None;
            if let Event::Keyboard(keyboard::Event::KeyPressed {
                key_code,
                modifiers,
            }) = event
            {
                if key_code == KeyCode::H && modifiers == Modifiers::CTRL {
                    subscription = Some(Message::HiddenFilesToggle);
                }
            }
            subscription
        })
    }
}
