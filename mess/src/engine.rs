use std::{
    fs::File,
    io::Read,
    path::Path,
};

use mess_core::{
    compiler::Compiler as CompilerTrait,
    parser::Parser, codegen::decl::Declarator,
};
#[cfg(feature = "exec-vm")]
use mess_vm::{
    Compiler as VmCompiler,
    Core as VmExec,
};

use crate::{comp_exec::CompExecPair, error::Error};


pub struct Engine {
    comp_exec_pair: CompExecPair,
    declarator: Declarator
}

impl Engine {
    /// Creates a new engine with the bytecode interpreter backend
    #[cfg(feature = "exec-vm")]
    pub fn new_vm(stack_size: usize) -> Engine {
        Engine {
            declarator: Declarator::default(),
            comp_exec_pair: CompExecPair::VM(VmCompiler::new(), VmExec::new(stack_size)),
        }
    }

    /// Creates a new engine with the x64 JIT backend
    #[cfg(feature = "exec-jit")]
    pub fn new_jit() -> Engine {
        unimplemented!("Not implemented yet")
    }

    /// Runs a script file at the given path
    pub fn run_file<P: AsRef<Path>>(&mut self, file_path: P) -> Result<(), Error> {
        let file_path = file_path.as_ref();
        let mut parser = Parser::new_with_path(file_path);
        let decl_list = parser.parse()?;
        self.comp_exec_pair.compile(&decl_list)?;
        Ok(())
    }

    /// Runs a piece of code
    pub fn run_code<S: ToString>(&mut self, code: S) -> Result<(), Error> {
        let mut parser = Parser::new(code);
        let decl_list = parser.parse()?;
        self.comp_exec_pair.compile(&decl_list)?;
        Ok(())
    }

    /// Loads a script file from the given path
    pub fn load_file<P: AsRef<Path>>(&mut self, file_path: P) -> Result<(), Error> {
        let file_path = file_path.as_ref();
        let mut parser = Parser::new_with_path(file_path);
        let decl_list = parser.parse()?;
        self.comp_exec_pair.compile(&decl_list)?;
        Ok(())
    }

    /// Loads a piece of code
    pub fn load_code<S: ToString>(&mut self, code: S) -> Result<(), Error> {
        let mut parser = Parser::new(code);
        let decl_list = parser.parse()?;
        self.comp_exec_pair.compile(&decl_list)?;
        Ok(())
    }

}
