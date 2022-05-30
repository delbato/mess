use mess_core::{compiler::Compiler, parser::ast::Declaration};
#[cfg(feature = "exec-vm")]
use mess_vm::{
    Compiler as VmCompiler,
    Core as VmCore
};

use crate::error::Error;

pub enum CompExecPair {
    #[cfg(feature = "exec-vm")]
    VM(VmCompiler, VmCore),
}

impl CompExecPair {
    /// Compiles a declaration list according to the chosen backend
    pub fn compile(&mut self, decl_list: &[Declaration]) -> Result<(), Error> {
        match self {
            #[cfg(feature = "exec-vm")]
            CompExecPair::VM(compiler, _) => compiler.compile(decl_list)?
        };
        Ok(())
    }
}