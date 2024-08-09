#[derive(Clone, Debug)]
pub enum Screen {
    NewOrLoadFile,
    Accounts,
    Account(usize),
    AccountSecondary(usize),
    ImportBoa(usize),
    ImportInvestor360,
    Monthly(usize),
}
