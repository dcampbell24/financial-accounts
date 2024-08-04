use std::path::PathBuf;

use super::{account, file_picker, money::Currency};

#[derive(Clone, Debug)]
pub enum Message {
    Account(account::Message),
    Back,
    ChangeAccountName(String),
    ChangeProjectMonths(String),
    Delete(usize),
    FilePicker(file_picker::Message),
    GetOhlc(usize),
    ImportBoa(usize, PathBuf),
    ImportBoaScreen(usize),
    UpdateAccount(usize),
    UpdateCurrency(Currency),
    SelectAccount(usize),
    SelectAccountSecondary(usize),
    SelectMonthly(usize),
    SubmitAccount,
    Exit,
}
