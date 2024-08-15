#[derive(Clone, Debug)]
pub enum Screen {
    Accounts,
    Account(usize),
    AccountSecondary(usize),
    Monthly(usize),
}
