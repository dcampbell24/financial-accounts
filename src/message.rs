#[derive(Clone, Debug)]
pub enum Message {
    Back,
    ChangeAccountName(String),
    ChangeTx(String),
    ChangeDate(String),
    ChangeComment(String),
    ChangeProjectMonths(String),
    ChangeFilterDateYear(String),
    ChangeFilterDateMonth(String),
    Delete(usize),
    NewAccount,
    UpdateAccount(usize),
    ProjectMonths,
    SelectAccount(usize),
    SelectMonthly(usize),
    SubmitTx,
    SubmitFilterDate,
}
