use std::{sync::{Arc, Mutex}, ops::DerefMut};

use crate::{var_type::Type, adapter::Adapter};
use derivative::Derivative;

#[derive(Derivative)]
#[derivative(Clone, Debug, PartialEq)]
pub struct Function {
    pub name: String,
    pub args: Vec<Type>,
    pub returns: Type,
    arg_sizes: Vec<usize>,
    #[derivative(Debug="ignore", PartialEq="ignore")]
    closure: Arc<Mutex<dyn FnMut(&mut Adapter)>>
}

impl Function {
    pub fn new<S: Into<String>>(name: S, args: Vec<Type>, returns: Type, closure: fn(&mut Adapter)) -> Self {
        Self {
            name: name.into(),
            args,
            arg_sizes: vec![],
            returns,
            closure: Arc::new(Mutex::new(closure))
        }
    }

    pub fn run(&self, adapter: &mut Adapter) {
        let mut closure_lock = self.closure.lock().unwrap();
        let closure = closure_lock.deref_mut();
        closure(adapter);
    }

    pub fn set_arg_sizes(&mut self, arg_sizes: Vec<usize>) {
        self.arg_sizes = arg_sizes;
    }

    pub fn get_arg_offset(&self, arg_index: usize) -> i32 {
        let mut offset: i32 = 0;
        let mut index = self.args.len() - 1;
        for var_size in self.arg_sizes.iter().rev() {
            offset -= *var_size as i32;
            if index == arg_index {
                break;
            }
            index -= 1;
        }
        offset
    }

    pub fn get_arg_size(&self, arg_index: usize) -> usize {
        self.arg_sizes[arg_index]
    }
}