use mess_core::compiler::Compiler as CompilerTrait;

use crate::artifact::Artifact;

use super::error::Error;

pub struct Compiler {}

impl Default for Compiler {
    fn default() -> Self {
        Self {}
    }
}

impl CompilerTrait for Compiler {
    type Output = Artifact;
    type Error = Error;

    fn compile(&mut self, decl_list: &[mess_core::parser::ast::Declaration]) -> Result<(), Self::Error> {
        Err(Error::Unimplemented(String::from("Not implemented yet")))
    }

    fn register_module(&mut self, module: Module) -> Result<(), Self::Error> {
        Err(Error::Unknown)
    }
}