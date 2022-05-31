use std::{sync::Arc, any::Any};

use mess_api::prelude::{AdapterImpl, Function};

use crate::{Core, codegen::register::Register, exec::register::RegisterAccess};

pub struct Adapter<'c> {
    function: Function,
    core: &'c mut Core
}

impl<'c> Adapter<'c> {
    pub fn new(func: &Function, core: &'c mut Core) -> Self {
        Self {
            function: func.clone(),
            core
        }
    }
}

impl<'c> AdapterImpl for Adapter<'c> {
    fn ret(&mut self, bytes: &[u8]) {
        if bytes.len() > 8 {
            panic!("Return value too big");
        }
        let mut actual_bytes = [0u8; 8];
        actual_bytes[0..8].copy_from_slice(bytes);
        let reg0 = self.core.reg(Register::R0.into()).unwrap();
        reg0.set(actual_bytes);
    }

    fn get_arg_bytes(&self, arg_index: usize) -> Vec<u8> {
        unimplemented!("Not implemented yet");
    }

    fn get_foreign_ptr(&self, ptr: u64) -> Box<dyn Any> {
        unimplemented!("Not implemented yet");
    }

    fn insert_foreign_ptr(&mut self, object: Box<dyn Any>) -> u64 {
        unimplemented!("Not implemented yet");
    }
}