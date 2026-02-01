#[derive(Clone, Debug, Default)]
pub enum Screen {
    #[default]
    Accounts,
    Account(usize),
    AccountSecondary(usize),
    Configuration,
}
