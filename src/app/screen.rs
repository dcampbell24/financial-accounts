#[derive(Clone, Debug)]
pub enum Screen {
    NewOrLoadFile,
    Accounts,
    Account(usize),
    AccountSecondary(usize),
    ImportBoa(usize),
    Monthly(usize),
}
