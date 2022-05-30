use std::collections::HashMap;

use crate::parser::ast::Type;

#[derive(Clone)]
pub struct ContDef {
    pub name: String,
    pub canon_name: String,
    pub members: HashMap<String, (u64, Type)>,
}
