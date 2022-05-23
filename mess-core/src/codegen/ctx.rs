use std::collections::HashMap;

use crate::parser::ast::Type;

pub struct StackContext {
    pub stack_extent: i32,
    pub stack_pos: i32,
    pub uid: u64,
    variable_positions: HashMap<String, (i32, Type)>,
}

impl StackContext {
    pub fn new(uid: u64) -> Self {
        Self {
            uid,
            stack_extent: 0,
            stack_pos: 0,
            variable_positions: HashMap::new(),
        }
    }

    pub fn extend_from(uid: u64, context: &StackContext) -> Self {
        let mut variables = HashMap::new();

        let context_size = context.stack_extent;
        for (var_name, (var_offset, var_type)) in context.variable_positions.iter() {
            let new_offset = context_size - var_offset;
            variables.insert(var_name.clone(), (new_offset as i32, var_type.clone()));
        }

        Self {
            uid,
            stack_pos: 0,
            stack_extent: 0,
            variable_positions: variables,
        }
    }

    pub fn inc_stack(&mut self, inc: isize) {
        self.stack_pos += inc as i32;
        if self.stack_pos > self.stack_extent {
            self.stack_extent = self.stack_pos as i32;
        }
    }

    pub fn dec_stack(&mut self, dec: isize) {
        self.stack_pos -= dec as i32;
    }

    pub fn set_var(&mut self, stack_pos: i32, name: &str, var_type: &Type) {
        self.variable_positions
            .insert(String::from(name), (stack_pos, var_type.clone()));
    }

    pub fn get_var(&self, name: &str) -> &(i32, Type) {
        self.variable_positions.get(name).unwrap()
    }
}

pub struct FnContext {
    pub ret_type: Type,
    pub stack_ctx_uid: u64,
}

impl FnContext {
    pub fn new(ret_type: Type, stack_ctx_uid: u64) -> Self {
        Self {
            ret_type,
            stack_ctx_uid,
        }
    }
}
