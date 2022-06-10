use std::{
    collections::{
        BTreeMap,
        HashMap,
    },
    ops::Range,
};

use mess_api::prelude::Function;
use mess_core::artifact::Artifact;

#[derive(PartialEq, Debug)]
pub struct Output {
    pub code: Vec<u8>,
    pub function_name_map: HashMap<String, u64>,
    pub functions: HashMap<u64, usize>,
    pub foreign_functions: HashMap<u64, Function>,
    pub static_pointers: BTreeMap<usize, Range<usize>>,
}

impl Output { 
    pub fn new() -> Output {
        Output {
            code: Vec::new(),
            functions: HashMap::new(),
            function_name_map: HashMap::new(),
            foreign_functions: HashMap::new(),
            static_pointers: BTreeMap::new(),
        }
    }

    pub fn with_code(mut self, code: Vec<u8>) -> Output {
        self.code = code;
        self
    }

    pub fn with_functions(mut self, functions: HashMap<u64, usize>) -> Output {
        self.functions = functions;
        self
    }

    /*pub fn with_foreign_functions(mut self, functions: HashMap<u64, Function>) -> Output {
        self.foreign_functions = functions;
        self
    }*/

    pub fn with_static_pointers(
        mut self,
        static_pointers: BTreeMap<usize, Range<usize>>,
    ) -> Output {
        self.static_pointers = static_pointers;
        self
    }

    pub fn get_size(&self) -> usize {
        self.code.len()
    }
}


impl Artifact for Output {}