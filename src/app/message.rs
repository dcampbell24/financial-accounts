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
    ChangeBalance(String),
    ChangeTx(String),
    ChangeDate(String),
    ChangeComment(String),
    ChangeFilterDateYear(String),
    ChangeFilterDateMonth(String),
    ChangeProjectMonths(String),
    ClearDate,
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
    SubmitBalance,
    SubmitTx,
    SubmitFilterDate,
    Exit,
}
