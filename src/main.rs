mod accounts;
mod ledger;

use crate::accounts::Accounts;

fn main() -> std::io::Result<()> {
    // let mut accounts = Accounts::new();
    let mut accounts = Accounts::load();
    accounts.prompt();

    // println!("{accounts:#?}");
    accounts.save()
}
