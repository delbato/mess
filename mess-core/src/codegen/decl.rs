use std::collections::VecDeque;

use crate::{
    codegen::def::{
        FunctionDef,
        ModuleDef,
    },
    parser::ast::Declaration,
};

pub struct Declarator {
    mod_def_stack: VecDeque<ModuleDef>,
    label_uid_ctr: u64,
}

impl Default for Declarator {
    fn default() -> Self {
        let mut mod_def_stack = VecDeque::new();
        mod_def_stack.push_front(ModuleDef::new("", "root"));
        Self {
            mod_def_stack,
            label_uid_ctr: 0,
        }
    }
}

impl Declarator {
    pub fn new(mod_def: ModuleDef) -> Self {
        let mut mod_def_stack = VecDeque::new();
        mod_def_stack.push_front(mod_def);
        Self {
            mod_def_stack,
            label_uid_ctr: 0,
        }
    }

    fn get_next_label_uid(&mut self) -> u64 {
        let ret = self.label_uid_ctr;
        self.label_uid_ctr += 1;
        ret
    }

    fn build_mod_path(&mut self) -> String {
        let mut i = self.mod_def_stack.len();
        let mut mod_path = String::new();
        while i > 0 {
            let mod_def = &self.mod_def_stack[i - 1];
            mod_path += &mod_def.name;
            mod_path += "::";
            i -= 1;
        }
        mod_path
    }

    pub fn get_result(&mut self) -> Result<(ModuleDef, u64), ()> {
        let mod_def = self.mod_def_stack.get(0).cloned().ok_or(())?;
        if self.mod_def_stack.len() > 0 {
            return Err(());
        }
        Ok((mod_def, self.label_uid_ctr))
    }

    pub fn declare(&mut self, decl_list: &[Declaration]) -> Result<(), ()> {
        for decl in decl_list {
            match decl {
                Declaration::Module { .. } => self.declare_mod(decl)?,
                Declaration::Function { .. } => self.declare_fn(decl)?,
                Declaration::Container { .. } => self.declare_cont(decl)?,
                _ => return Err(()),
            };
        }
        Ok(())
    }

    fn declare_mod(&mut self, mod_decl: &Declaration) -> Result<(), ()> {
        let (name, decl_list) = match mod_decl {
            Declaration::Module { decl_list, name } => (name, decl_list),
            _ => return Err(()),
        };
        let module_path = self.build_mod_path();
        let mut mod_def = ModuleDef::new(&module_path, name);
        self.mod_def_stack.push_front(mod_def);
        self.declare(decl_list)?;
        mod_def = self.mod_def_stack.pop_front().ok_or(())?;
        let front_mod = self.mod_def_stack.get_mut(0).ok_or(())?;
        front_mod.add_module(mod_def);
        Ok(())
    }

    fn declare_cont(&mut self, _cont_decl: &Declaration) -> Result<(), ()> {
        Err(())
    }

    fn declare_fn(&mut self, fn_decl: &Declaration) -> Result<(), ()> {
        let module_path = self.build_mod_path();
        let label_uid = self.get_next_label_uid();
        let fn_def = FunctionDef::from_decl(label_uid, &module_path, fn_decl)?;
        let front_mod = &mut self.mod_def_stack[0];
        front_mod.add_function(fn_def);
        Ok(())
    }
}
