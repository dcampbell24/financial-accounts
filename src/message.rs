//! The messages that can be passed to iced.

use std::path::PathBuf;

#[derive(Clone, Debug)]
pub enum Message {
    LoadFile(PathBuf),
    ChangeDir(PathBuf),
    Back,
    ChangeAccountName(String),
    ChangeTx(String),
    ChangeDate(String),
    ChangeComment(String),
    ChangeProjectMonths(String),
    ChangeFilterDateYear(String),
    ChangeFilterDateMonth(String),
    Delete(usize),
    NewAccount,
    UpdateAccount(usize),
    ProjectMonths,
    SelectAccount(usize),
    SelectMonthly(usize),
    SubmitTx,
    SubmitFilterDate,
}
