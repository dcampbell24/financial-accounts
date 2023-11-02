mod accounts;
mod ledger;

use iced::{Settings, Sandbox};
use ledger::Ledger;

use crate::accounts::Accounts;

fn main() -> std::io::Result<()> {
    // let mut accounts = Accounts::new();
    // let mut accounts = Accounts::load();
    // accounts.prompt();

    Accounts::run(Settings::default()).unwrap();

    // println!("{accounts:#?}");
    // accounts.save();
    Ok(())
}
