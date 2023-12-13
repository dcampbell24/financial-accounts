#[derive(Clone, Debug)]
pub enum Screen {
    NewOrLoadFile,
    Accounts,
    Account(usize),
    Monthly(usize),
}
