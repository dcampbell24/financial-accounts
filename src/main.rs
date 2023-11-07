mod accounts;
mod ledger;

use iced::{Sandbox, Settings};

use crate::accounts::Accounts;

const TEXT_SIZE: u16 = 25;

fn main() -> std::io::Result<()> {
    Accounts::run(Settings::default()).unwrap();
    // println!("{accounts:#?}");
    Ok(())
}
