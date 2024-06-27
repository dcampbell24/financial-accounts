use std::path::PathBuf;

use super::money::Currency;

#[derive(Clone, Debug)]
pub enum Message {
    NewFile(PathBuf),
    LoadFile(PathBuf),
    ChangeDir(PathBuf),
    ChangeFileName(String),
    HiddenFilesToggle,
    Back,
    ChangeAccountName(String),
    ChangeTx(String),
    ChangeDate(String),
    ChangeComment(String),
    ChangeFilterDateYear(String),
    ChangeFilterDateMonth(String),
    ChangeProjectMonths(String),
    Delete(usize),
    UpdateAccount(usize),
    UpdateCurrency(Currency),
    SelectAccount(usize),
    SelectMonthly(usize),
    SubmitAccount,
    SubmitTx,
    SubmitFilterDate,
}
