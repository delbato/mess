use std::any::Any;
use std::error::Error;

use crate::parser::ast::Declaration;

pub trait Compiler {
    type Output: Any;

    fn compile(&mut self, decl_list: &[Declaration]) -> Result<(), ()>;

    fn get_output(&mut self) -> Self::Output;
}