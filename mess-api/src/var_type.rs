#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Void,
    Int,
    Float,
    Bool,
    Str,
    Named(String),
    Ref(Box<Type>),
}