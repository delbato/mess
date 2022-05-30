use std::collections::BTreeMap;

use crate::parser::ast::{Type, Declaration, EnumVariant};

pub struct EnumDef {
    pub name: String,
    pub variants: Vec<EnumVariantDef>
}

impl From<Declaration> for EnumDef {
    fn from(enum_decl: Declaration) -> Self {
        if let Declaration::Enum { name, variants } = enum_decl {
            Self {
                name,
                variants: variants.into_iter().map(|v| v.into()).collect()
            }
        } else {
            panic!("Not an enum declaration!");
        }
    }
}

pub enum EnumVariantDef {
    Empty {
        name: String
    },
    Tuple {
        name: String,
        types: Vec<Type>
    },
    Cont {
        name: String,
        members: BTreeMap<String, Type>
    }
}

impl From<EnumVariant> for EnumVariantDef {
    fn from(enum_variant: EnumVariant) -> Self {
        match enum_variant {
            EnumVariant::Empty(name) => Self::Empty { name },
            EnumVariant::Cont(name, members) => Self::Cont { name, members },
            EnumVariant::Tuple(name, types) => Self::Tuple { name, types }
        }
    }
}