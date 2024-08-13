use std::path::PathBuf;

use super::{account, file_picker, money::Currency};

#[derive(Clone, Debug)]
pub enum Message {
    Account(account::Message),
    Back,
    ChartWeek,
    ChartMonth,
    ChartYear,
    ChartAll,
    ChangeAccountName(String),
    ChangeProjectMonths(String),
    Delete(usize),
    FilePicker(file_picker::Message),
    GetPrice(usize),
    GetPriceAll,
    ImportBoa(usize, PathBuf),
    ImportBoaScreen(usize),
    ImportInvestor360(PathBuf),
    ImportInvestor360Screen,
    UpdateAccount(usize),
    UpdateCurrency(Currency),
    SelectAccount(usize),
    SelectAccountSecondary(usize),
    SelectMonthly(usize),
    SubmitAccount,
    Exit,
}
