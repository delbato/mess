use mess_api::prelude::Function;

use crate::parser::ast::{Type, Declaration};

#[derive(Clone)]
pub struct FunctionDef {
    pub label_uid: u64,
    pub name: String,
    pub canon_name: String,
    pub returns: Type,
    pub arguments: Vec<(String, Type)>,
}

impl FunctionDef {
    pub fn from_decl(
        label_uid: u64,
        module_path: &str,
        decl: &Declaration,
    ) -> Result<FunctionDef, ()> {
        match decl {
            Declaration::Function {
                name,
                returns,
                arguments,
                ..
            } => Ok(Self {
                label_uid,
                name: name.clone(),
                returns: returns.clone(),
                canon_name: format!("{}{}", module_path, name),
                arguments: arguments.clone(),
            }),
            _ => Err(()),
        }
    }

    pub fn from_api(
        label_uid: u64,
        module_path: &str,
        api_fun: Function
    ) -> Self {
        Self {
            label_uid,
            name: api_fun.name.clone(),
            canon_name: format!("{}{}", module_path, api_fun.name),
            returns: api_fun.returns.into(),
            arguments: api_fun.args.into_iter()
                .map(|fn_arg| (String::new(), fn_arg.into()))
                .collect()
        }
    }
}