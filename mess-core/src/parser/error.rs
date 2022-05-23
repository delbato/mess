use std::{
    error::Error as StdError,
    fmt::{
        Display,
        Formatter,
        Result as FmtResult,
    },
    result::Result as StdResult,
};

pub type Result<T> = StdResult<T, Error>;

#[derive(Debug)]
pub enum Error {
    Unknown,
    Unimplemented(&'static str),
    ExpectedFn,
    ExpectedEnum,
    ExpectedOpenBlock,
    ExpectedOn,
    ExpectedAssign,
    ExpectedImport,
    ExpectedIdentifier,
    ExpectedColon,
    ExpectedOpenParan,
    ExpectedCloseParan,
    ExpectedType,
    ExpectedWhile,
    ExpectedVar,
    ExpectedSemicolon,
    MalformedExpression,
    MalformedImport,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{:?}", self)
    }
}

impl StdError for Error {}

unsafe impl Send for Error {}
unsafe impl Sync for Error {}
