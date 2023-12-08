//! The messages that can be passed to iced.

use std::path::PathBuf;

#[derive(Clone, Debug)]
pub enum Message {
    NewFile(PathBuf),
    LoadFile(PathBuf),
    ChangeDir(PathBuf),
    ChangeFileName(String),
    Back,
    ChangeAccountName(String),
    ChangeTx(String),
    ChangeDate(String),
    ChangeComment(String),
    ChangeFilterDateYear(String),
    ChangeFilterDateMonth(String),
    ChangeProjectMonths(String),
    Delete(usize),
    NewAccount,
    UpdateAccount(usize),
    SelectAccount(usize),
    SelectMonthly(usize),
    SubmitTx,
    SubmitFilterDate,
}
