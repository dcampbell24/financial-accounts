#[derive(Clone, Debug)]
pub enum Screen {
    NewOrLoadFile,
    Accounts,
    Account(usize),
    ImportBoa(usize),
    Monthly(usize),
}
