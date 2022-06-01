use std::{path::PathBuf, process::exit};

use clap::{Parser, ArgEnum};
use mess::{engine::Engine, error::Error};

#[derive(Parser)]
#[clap(name = "mess")]
#[clap(author, version, about)]
struct RunArgs {
    #[clap(help = "Execution mode", short, long, arg_enum, default_value_t=ExecMode::Vm)]
    exec: ExecMode,
    #[clap(help = "Stack size of the interpeter. Unused with JIT backend.", long, default_value_t=10*1024*1024)]
    stack_size: usize,
    #[clap(help = "Path to the script file to execute", index = 1)]
    script_file: PathBuf
}

#[derive(Clone, ArgEnum)]
enum ExecMode {
    #[clap(help = "Run with the bytecode interpeter")]
    Vm,
    #[clap(help = "Run with the AMD64 JIT compiler")]
    Jit
}

fn main() -> Result<(), Error> {
    let run_args = RunArgs::parse();
    let mut engine = match run_args.exec {
        ExecMode::Jit => {
            println!("JIT not implemented yet!");
            return Ok(());
        },
        ExecMode::Vm => {
            Engine::new_vm(run_args.stack_size)
        }
    };
    engine.run_file(&run_args.script_file)
}
