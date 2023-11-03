mod accounts;
mod ledger;

use iced::{Settings, Sandbox};

use crate::accounts::Accounts;

fn main() -> std::io::Result<()> {
    let mut accounts = Accounts::new();

    Accounts::run(Settings::default()).unwrap();

    println!("{accounts:#?}");
    accounts.save();
    Ok(())
}
