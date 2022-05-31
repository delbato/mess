use std::collections::BTreeMap;

use crate::{function::Function, container::Container, interface::Interface};

pub struct Module {
    pub name: String,
    pub functions: BTreeMap<String, Function>,
    containers: BTreeMap<String, Container>,
    interfaces: BTreeMap<String, Interface>
}

impl Module {
    pub fn new(name: String) -> Self {
        Self {
            name,
            functions: BTreeMap::new(),
            containers: BTreeMap::new(),
            interfaces: BTreeMap::new()
        }
    }

    pub fn add_function(&mut self, function: Function) {
        self.functions.insert(function.name.clone(), function);
    }
}