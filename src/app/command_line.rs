use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    /// Load FILE
    #[arg(long, value_name = "FILE", exclusive = true)]
    pub load: Option<String>,

    /// Create a new FILE
    #[arg(long, value_name = "FILE", exclusive = true)]
    pub new: Option<String>,

    /// Build the manpage
    #[arg(long)]
    pub man: bool,
}

#[derive(Debug)]
pub enum File {
    Load(PathBuf),
    New(PathBuf),
    None,
}

#[must_use]
pub fn get_configuration_file() -> File {
    let args = Args::parse();

    if let Some(arg) = args.load {
        File::Load(PathBuf::from(arg))
    } else if let Some(arg) = args.new {
        File::New(PathBuf::from(arg))
    } else {
        File::None
    }
}
