use std::path::PathBuf;

use iced::{
    widget::{button, column, row, text, text_input, Column},
    Alignment,
};
use thousands::Separable;

use crate::{
    accounts::{Accounts, Screen},
    file_picker::FilePicker,
    message::Message,
    PADDING, TEXT_SIZE,
};

#[derive(Clone, Debug)]
pub struct App {
    pub accounts: Accounts,
    pub error_str: String,
    pub file_path: PathBuf,
    pub file_picker: FilePicker,
    pub name: String,
    pub project_months: u64,
    pub project_months_str: String,
    pub screen: Screen,
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
        let mut col_0 = column![text("Account").size(TEXT_SIZE)].padding(PADDING);
        let mut col_1 = column![text("Current Month").size(TEXT_SIZE)].padding(PADDING).align_items(Alignment::End);
        let mut col_2 = column![text("Last Month").size(TEXT_SIZE)].padding(PADDING).align_items(Alignment::End);
        let mut col_3 = column![text("Current Year").size(TEXT_SIZE)].padding(PADDING).align_items(Alignment::End);
        let mut col_4 = column![text("Last Year").size(TEXT_SIZE)].padding(PADDING).align_items(Alignment::End);
        let mut col_5 = column![text("Balance").size(TEXT_SIZE)].padding(PADDING).align_items(Alignment::End);
        let mut col_6 = column![text("").size(TEXT_SIZE)].padding(PADDING);
        let mut col_7 = column![text("").size(TEXT_SIZE)].padding(PADDING);
        let mut col_8 = column![text("").size(TEXT_SIZE)].padding(PADDING);
        let mut col_9 = column![text("").size(TEXT_SIZE)].padding(PADDING);

        for (i, account) in self.accounts.inner.iter().enumerate() {
            let total = account.sum();
            let current_month = account.sum_current_month();
            let last_month = account.sum_last_month();
            let current_year = account.sum_current_year();
            let last_year = account.sum_last_year();
            col_0 = col_0.push(text(&account.name).size(TEXT_SIZE));
            col_1 = col_1.push(text(current_month.separate_with_commas()).size(TEXT_SIZE));
            col_2 = col_2.push(text(last_month.separate_with_commas()).size(TEXT_SIZE));
            col_3 = col_3.push(text(current_year.separate_with_commas()).size(TEXT_SIZE));
            col_4 = col_4.push(text(last_year.separate_with_commas()).size(TEXT_SIZE));
            col_5 = col_5.push(text(total.separate_with_commas()).size(TEXT_SIZE));
            col_6 = col_6.push(button("Tx").on_press(Message::SelectAccount(i)));
            col_7 = col_7.push(button("Monthly Tx").on_press(Message::SelectMonthly(i)));
            col_8 = col_8.push(button("Update Name").on_press(Message::UpdateAccount(i)));
            col_9 = col_9.push(button("Delete").on_press(Message::Delete(i)));
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
            totals,
            row![
                text("Account ").size(TEXT_SIZE),
                text_input("Name", &self.name)
                    .on_submit(Message::NewAccount)
                    .on_input(Message::ChangeAccountName)
            ],
            row![
                text("Project ").size(TEXT_SIZE),
                text_input("Months", &self.project_months_str)
                    .on_input(Message::ChangeProjectMonths)
                    .on_submit(Message::ProjectMonths),
                text((self.accounts.total() + self.accounts.total_for_months(self.project_months)).separate_with_commas()).size(TEXT_SIZE),
            ],
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
