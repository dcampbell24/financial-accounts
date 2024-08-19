use super::{
    account,
    money::{Currency, Fiat},
};

#[derive(Clone, Debug)]
pub enum Message {
    AddFiat,
    Account(account::Message),
    Back,
    ChartWeek,
    ChartMonth,
    ChartYear,
    ChartAll,
    ChangeAccountName(String),
    ChangeProjectMonths(String),
    CheckMonthly,
    Configuration,
    Delete(usize),
    FileLoad,
    FileSaveAs,
    GetPrice(usize),
    GetPriceAll,
    ImportBoa(usize),
    ImportInvestor360,
    UpdateAccount(usize),
    UpdateCurrency(Currency),
    UpdateFiat(Fiat),
    SelectAccount(usize),
    SelectAccountSecondary(usize),
    SelectMonthly(usize),
    SubmitAccount,
    Exit,
}
