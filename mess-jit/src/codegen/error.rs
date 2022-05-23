use std::{
    error::Error as StdError,
    fmt::{
        Display,
        Formatter,
        Result as FmtResult,
    },
    result::Result as StdResult,
};

use mess_core::parser::ast::Type;

pub type Result<T> = StdResult<T, Error>;

#[derive(Debug)]
pub enum Error {
    Unknown,
    TypeMismatch(Type, Type),
    UnknownType(Type),
    Unimplemented(&'static str),
    UnsupportedDeclaration,
    ExpectedReturnExpression,
    RegisterMapping,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{:?}", self)
    }
}

impl StdError for Error {}

unsafe impl Send for Error {}
unsafe impl Sync for Error {}

impl From<()> for Error {
    fn from(_: ()) -> Self {
        Self::Unknown
    }
}
