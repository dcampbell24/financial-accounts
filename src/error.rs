#[derive(Debug)]
pub enum Error {
    Err(String),
    FmtErr(std::fmt::Error),
    IoErr(std::io::Error),
}

impl From<std::fmt::Error> for Error {
    fn from(other: std::fmt::Error) -> Error {
        Error::FmtErr(other)
    }
}

impl From<std::io::Error> for Error {
    fn from(other: std::io::Error) -> Error {
        Error::IoErr(other)
    }
}
