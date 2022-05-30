use std::fmt::Display;
use std::error::Error as StdError;

use mess_vm::codegen::error::Error as VmCompileError;
use mess_vm::exec::core::CoreError as VmCoreError;
use mess_core::parser::error::Error as ParseError;

#[derive(Debug)]
pub enum Error {
    #[cfg(feature = "exec-vm")]
    VmCompileError(VmCompileError),
    #[cfg(feature = "exec-vm")]
    VmCoreError(VmCoreError),
    ParseError(ParseError)
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl StdError for Error {}

impl From<ParseError> for Error {
    fn from(p: ParseError) -> Self {
        Self::ParseError(p)
    }
}

#[cfg(feature = "exec-vm")]
impl From<VmCoreError> for Error {
    fn from(e: VmCoreError) -> Self {
        Self::VmCoreError(e)
    }
}

#[cfg(feature = "exec-vm")]
impl From<VmCompileError> for Error {
    fn from(e: VmCompileError) -> Self {
        Self::VmCompileError(e)
    }
}