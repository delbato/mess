use std::collections::HashMap;

use super::Token;

#[derive(Debug, Clone)]
pub enum Declaration {
    Function {
        external: bool,
        name: String,
        returns: Type,
        arguments: Vec<(String, Type)>,
        body: Option<Vec<Statement>>,
    },
    Module {
        name: String,
        decl_list: Vec<Declaration>,
    },
    Container {
        name: String,
        members: HashMap<String, Type>,
    },
    Interface {
        name: String,
        functions: HashMap<String, (Vec<(String, Type)>, Option<Vec<Statement>>, Type)>,
    },
    Enum {
        name: String,
        variants: Vec<EnumVariant>,
    },
    Import(Vec<(String, String)>),
}

#[derive(Debug, Clone)]
pub enum EnumVariant {
    Empty(String),
    Tuple(String, Vec<Type>),
    Cont(String, HashMap<String, Type>),
}

#[derive(Debug, Clone)]
pub struct ContMember {
    pub public: bool,
    pub name: String,
    pub var_type: Type,
}

#[derive(Debug, Clone)]
pub enum Statement {
    VarDeclarationStmt {
        name: String,
        var_type: Type,
        expr: Expression,
    },
    Import(Vec<(String, String)>),
    Return(Option<Expression>),
    Yield(Option<Expression>),
    Break,
    Continue,
    While(Expression, Vec<Statement>),
    Condition {
        expr: Expression,
        cond_body: Vec<Statement>,
        cond_chain: Vec<(Expression, Vec<Statement>)>,
        else_body: Vec<Statement>,
    },
    ExpressionStmt(Expression),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    Plus,
    Minus,
    Neg,
    Pos,
    Times,
    Divide,
    Not,
    Ref,
    Deref,
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    LessThan,
    GreaterThan,
    LessThanEquals,
    GreaterThanEquals,
    Equals,
    NotEquals,
    OpenParan,
    CloseParan,
}

impl Operator {
    pub fn from(token: Token) -> Option<Self> {
        match token {
            Token::Plus => Some(Operator::Plus),
            Token::Minus => Some(Operator::Minus),
            Token::Times => Some(Operator::Times),
            Token::Divide => Some(Operator::Divide),
            Token::Assign => Some(Operator::Assign),
            Token::LessThan => Some(Operator::LessThan),
            Token::GreaterThan => Some(Operator::GreaterThan),
            Token::LessThanEquals => Some(Operator::LessThanEquals),
            Token::GreaterThanEquals => Some(Operator::GreaterThanEquals),
            Token::Ref => Some(Operator::Ref),
            Token::Tilde => Some(Operator::Deref),
            Token::AddAssign => Some(Operator::AddAssign),
            Token::SubAssign => Some(Operator::SubAssign),
            Token::MulAssign => Some(Operator::MulAssign),
            Token::DivAssign => Some(Operator::DivAssign),
            Token::OpenParan => Some(Operator::OpenParan),
            Token::CloseParan => Some(Operator::CloseParan),
            _ => None,
        }
    }

    pub fn prec(&self) -> i8 {
        match self {
            Operator::Plus => 2,
            Operator::Minus => 1,
            Operator::LessThan => 0,
            Operator::LessThanEquals => 0,
            Operator::GreaterThan => 0,
            Operator::GreaterThanEquals => 0,
            Operator::Times => 3,
            Operator::Divide => 3,
            Operator::Ref => 4,
            Operator::Deref => 4,
            Operator::Pos => 5,
            Operator::Neg => 5,
            Operator::Not => 5,
            Operator::Assign => -1,
            Operator::AddAssign => -1,
            Operator::SubAssign => -1,
            Operator::MulAssign => -1,
            Operator::DivAssign => -1,
            _ => 0,
        }
    }

    pub fn unary(&self) -> bool {
        match self {
            Operator::Pos => true,
            Operator::Neg => true,
            Operator::Ref => true,
            Operator::Deref => true,
            Operator::Not => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expression {
    Call(String, Vec<Expression>),
    IntLiteral(i64),
    FloatLiteral(f32),
    BoolLiteral(bool),
    StringLiteral(String),
    Variable(String),
    Unary(Operator, Box<Expression>),
    Binary(Box<Expression>, Operator, Box<Expression>),
    Condition {
        expr: Box<Expression>,
        cond_body: Vec<Statement>,
        cond_chain: Vec<(Expression, Vec<Statement>)>,
        else_body: Vec<Statement>,
        yield_expr: Option<Box<Expression>>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Auto,
    Void,
    Int,
    Float,
    Bool,
    String,
    Tuple(Vec<Type>),
    Named(String),
    Ref(Box<Type>),
    Deref(Box<Type>),
    UnsizedArray(Box<Type>),
    SizedArray(Box<Type>),
}
