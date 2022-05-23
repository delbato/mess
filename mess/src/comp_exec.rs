use mess_core::{compiler::Compiler, parser::ast::Declaration};
#[cfg(feature = "exec-vm")]
use mess_vm::{
    Compiler as VmCompiler,
    Core as VmCore
};

pub enum CompExecPair {
    #[cfg(feature = "exec-vm")]
    VM(VmCompiler, VmCore),
}

impl CompExecPair {
    /// Compiles a declaration list according to the chosen backend
    pub fn compile(&mut self, decl_list: &[Declaration]) -> Result<(), ()> {
        match self {
            #[cfg(feature = "exec-vm")]
            CompExecPair::VM(compiler, _) => compiler.compile(decl_list)
        }
    }
}