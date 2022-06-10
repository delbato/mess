use std::{path::PathBuf, process::exit, collections::HashMap, error::Error as StdError, hash::Hash};

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
    #[clap(help = "Supplemental options", short, long, parse(from_str = parse_options), number_of_values=1)]
    options: HashMap<String, String>,
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

fn parse_options(s: &str) -> HashMap<String, String> {
    let mut ret = HashMap::new();
    if s.trim().is_empty() {
        return ret;
    }
    if s.contains(',') {
        let options_kv: Vec<&str> = s.split(',').collect();
        for option_kv in options_kv {
            if option_kv.contains('=') {
                let option: Vec<&str> = option_kv.split('=').collect();
                let option_key = String::from(option[0]);
                let option_val = String::from(option[1]);
                ret.insert(option_key, option_val);
            } else {
                let option_key = String::from(option_kv);
                ret.insert(option_key, String::new());
            }
        }
    } else if s.contains('=') {
        let option: Vec<&str> = s.split('=').collect();
        let option_key = String::from(option[0]);
        let option_val = String::from(option[1]);
        ret.insert(option_key, option_val);
    } else {
        let option_key = String::from(s);
        ret.insert(option_key, String::new());
    }
    ret
}

fn main() -> Result<(), Error> {
    let run_args = RunArgs::parse();
    println!("Options: {:#?}", run_args.options);
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
