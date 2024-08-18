use super::{account, money::Currency};

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
    CheckMonthly,
    Delete(usize),
    FileLoad,
    FileSaveAs,
    GetPrice(usize),
    GetPriceAll,
    ImportBoa(usize),
    ImportInvestor360,
    UpdateAccount(usize),
    UpdateCurrency(Currency),
    SelectAccount(usize),
    SelectAccountSecondary(usize),
    SelectMonthly(usize),
    SubmitAccount,
    Exit,
}
