use std::{
    fmt::{Result as FmtResult, Display},
    error::Error as StdError
};

#[derive(Debug)]
pub enum Error {
    Unknown,
    Unimplemented(String)
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> FmtResult {
        write!(f, "{:#?}", self)
    }
}

impl StdError for Error {}