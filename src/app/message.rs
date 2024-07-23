use std::path::PathBuf;

use super::{account::MessageAccount, money::Currency};

#[derive(Clone, Debug)]
pub enum Message {
    Account(MessageAccount),
    NewFile(PathBuf),
    LoadFile(PathBuf),
    ChangeDir(PathBuf),
    ChangeFileName(String),
    HiddenFilesToggle,
    Back,
    ChangeAccountName(String),
    ChangeProjectMonths(String),
    Delete(usize),
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
