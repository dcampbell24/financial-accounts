mod accounts;
mod ledger;

use iced::{Sandbox, Settings};

use crate::accounts::Accounts;

fn main() -> std::io::Result<()> {
    Accounts::run(Settings::default()).unwrap();
    // println!("{accounts:#?}");
    Ok(())
}
