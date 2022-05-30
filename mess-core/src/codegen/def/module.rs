use std::collections::HashMap;

use super::FunctionDef;

#[derive(Clone)]
pub struct ModuleDef {
    pub name: String,
    pub canon_name: String,
    pub functions: HashMap<String, FunctionDef>,
    pub modules: HashMap<String, ModuleDef>,
}

impl ModuleDef {
    pub fn new<N: Into<String>>(module_path: &str, name: N) -> Self {
        let name = name.into();
        Self {
            canon_name: format!("{}{}", module_path, name),
            name,
            functions: HashMap::new(),
            modules: HashMap::new(),
        }
    }

    pub fn add_module(&mut self, module_def: ModuleDef) {
        let name = module_def.name.clone();
        self.modules.insert(name, module_def);
    }

    pub fn add_function(&mut self, fn_def: FunctionDef) {
        let name = fn_def.name.clone();
        self.functions.insert(name, fn_def);
    }

    pub fn has_function(&self, fn_name: &str) -> bool {
        self.functions.contains_key(fn_name)
    }

    pub fn has_module(&self, mod_name: &str) -> bool {
        self.modules.contains_key(mod_name)
    }

    pub fn get_function(&self, fn_name: &str) -> Result<&FunctionDef, ()> {
        self.functions.get(fn_name).ok_or(())
    }

    pub fn get_function_list(&self) -> Result<Vec<(u64, String)>, ()> {
        let ret = self
            .functions
            .iter()
            .map(|(_, fn_def)| (fn_def.label_uid, fn_def.canon_name.clone()))
            .collect();
        Ok(ret)
    }

    pub fn get_module(&self, mod_name: &str) -> Result<&ModuleDef, ()> {
        self.modules.get(mod_name).ok_or(())
    }
}