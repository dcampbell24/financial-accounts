use super::{
    account,
    money::{Currency, Fiat},
};

#[derive(Clone, Debug)]
pub enum Message {
    AddCrypto,
    AddFiat,
    AddGroup,
    AddMetal,
    AddStockPlus,
    Account(account::Message),
    Back,
    ChartWeek,
    ChartMonth,
    ChartYear,
    ChartAll,
    ChangeAccountName(String),
    ChangeProjectMonths(String),
    Checkbox((usize, bool)),
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
    UpdateCryptoCurrency(Fiat),
    UpdateCryptoDescription(String),
    UpdateCryptoSymbol(String),
    UpdateFiat(Fiat),
    UpdateMetalCurrency(Fiat),
    UpdateMetalDescription(String),
    UpdateMetalSymbol(String),
    UpdateStockPlusDescription(String),
    UpdateStockPlusSymbol(String),
    SelectAccount(usize),
    SelectAccountSecondary(usize),
    SelectMonthly(usize),
    SubmitAccount,
    Exit,
}
