use std::{
    fs::File,
    io::Read,
    path::Path,
};

use mess_core::{
    compiler::Compiler as CompilerTrait,
    parser::Parser,
};
#[cfg(feature = "exec-vm")]
use mess_vm::{
    Compiler as VmCompiler,
    Core as VmExec,
};

use crate::comp_exec::CompExecPair;


pub struct Engine {
    comp_exec_pair: CompExecPair,
}

impl Engine {
    /// Creates a new engine with the bytecode interpreter backend
    #[cfg(feature = "exec-vm")]
    pub fn new_vm(stack_size: usize) -> Engine {
        Engine {
            comp_exec_pair: CompExecPair::VM(VmCompiler::new(), VmExec::new(stack_size)),
        }
    }

    /// Creates a new engine with the x64 JIT backend
    #[cfg(feature = "exec-jit")]
    pub fn new_jit() -> Engine {
        unimplemented!("Not implemented yet")
    }

    /// Runs a script file at the given path
    pub fn run_file<'p, P: Into<&'p Path>>(&mut self, file_path: P) -> Result<(), ()> {
        let file_path = file_path.into();
        let mut file_content = String::new();
        let mut file = File::open(file_path).map_err(|_| ())?;
        file.read_to_string(&mut file_content).map_err(|_| ())?;
        let mut parser = Parser::new(file_content);
        let decl_list = parser.parse().map_err(|_| ())?;
        self.comp_exec_pair.compile(&decl_list)?;
        Ok(())
    }
}
