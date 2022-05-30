/// AST definitions
pub mod ast;

/// Token definitions
pub mod token;

pub mod error;

#[cfg(test)]
mod tests;

use std::{
    collections::{
        HashMap,
        VecDeque, BTreeMap,
    },
    convert::AsRef,
    ops::Range, path::{Path, PathBuf},
};

use ast::{
    Declaration,
    EnumVariant,
    Expression,
    Operator,
    Statement,
    Type,
};
use error::{
    Error,
    Result,
};
use logos::{
    Logos,
};
use token::Token;

use self::ast::{InterfaceFunction, ContainerFunction};

/// The Parser
pub struct Parser {
    tokens: Vec<(Token, Range<usize>)>,
    source: String,
    token_pos: usize,
    yield_stack: VecDeque<Option<Expression>>,
    current_path: Option<PathBuf>
}

impl Parser {
    /// Creates a new parser, wrapping the given source
    pub fn new<S: ToString>(source: S) -> Self {
        let source = source.to_string();
        let lexer = Token::lexer(source.as_ref());
        let tokens: Vec<(Token, Range<usize>)> =
            lexer.spanned().collect();
        Self {
            tokens,
            source,
            token_pos: 0,
            current_path: None,
            yield_stack: VecDeque::new(),
        }
    }

    /// Creates a new parser, using the given file path
    pub fn new_with_path<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();
        if !path.is_file() {
            panic!("Given path is not a file!");
        }
        let path = std::fs::canonicalize(path).unwrap();
        let source = std::fs::read_to_string(&path).expect("Could not read source file!");
        let lexer = Token::lexer(source.as_ref());
        let tokens: Vec<(Token, Range<usize>)> =
            lexer.spanned().collect();
        Self {
            source,
            tokens,
            token_pos: 0,
            current_path: Some(path),
            yield_stack: VecDeque::new()
        }
    }

    fn get_token(&self) -> Result<Token> {
        self.tokens
            .get(self.token_pos)
            .map(|(token, _range)| token.clone())
            .ok_or(Error::Unknown)
    }

    fn peek_token(&self, offset: isize) -> Result<Token> {
        let pos = (self.token_pos as isize) + offset;
        self.tokens
            .get(pos as usize)
            .map(|(token, _range)| token.clone())
            .ok_or(Error::Unknown)
    }

    fn advance(&mut self) {
        self.token_pos += 1;
    }

    fn get_value(&self) -> Result<String> {
        let range = self.get_range()?;
        let ret = String::from(&self.source[range]);
        Ok(ret)
    }

    fn get_range(&self) -> Result<Range<usize>> {
        self.tokens
            .get(self.token_pos)
            .map(|(_token, range)| range.clone())
            .ok_or(Error::Unknown)
    }

    /// Parses the source into a root decl list
    pub fn parse(&mut self) -> Result<Vec<Declaration>> {
        self.parse_decl_list(&[])
    }

    /// Parses a declaration list
    pub fn parse_decl_list(&mut self, delims: &[Token]) -> Result<Vec<Declaration>> {
        let mut ret = vec![];
        while self.token_pos < self.tokens.len() {
            let mut token = self.get_token()?;
            if delims.contains(&token) {
                break;
            }

            while [Token::Pub, Token::Ext].contains(&token) {
                token = self.peek_token(1)?;
            }
            
            let decl = match token {
                Token::Fun => self.parse_decl_fn()?,
                Token::Mod => self.parse_decl_mod()?,
                Token::Cont => self.parse_decl_cont()?,
                Token::Import => self.parse_decl_import()?,
                Token::Intf => self.parse_decl_intf()?,
                Token::Enum => self.parse_decl_enum()?,
                _ => return Err(Error::Unknown),
            };
            ret.push(decl);
        }
        Ok(ret)
    }

    /// Parses an interface declaration
    pub fn parse_decl_intf(&mut self) -> Result<Declaration> {
        let mut token = self.get_token()?;
        if token != Token::Intf {
            return Err(Error::ExpectedIntf);
        }
        self.advance();

        token = self.get_token()?;
        if token != Token::Identifier {
            return Err(Error::ExpectedIdentifier);
        }
        let intf_name = self.get_value()?;
        self.advance();

        token = self.get_token()?;
        if token != Token::OpenBlock {
            return Err(Error::ExpectedOpenBlock);
        }
        self.advance();

        let intf_functions = self.parse_intf_functions()?;

        Ok(Declaration::Interface {
            name: intf_name,
            functions: intf_functions
        })
    }

    /// Parses interface functions
    pub fn parse_intf_functions(&mut self) -> Result<Vec<InterfaceFunction>> {
        let mut ret = vec![];
        while self.get_token()? != Token::CloseBlock {
            let intf_fn = self.parse_intf_function()?;
            ret.push(intf_fn);
        }
        self.advance();
        Ok(ret)
    }

    /// Parses a single interface function
    pub fn parse_intf_function(&mut self) -> Result<InterfaceFunction> {
        let mut token = self.get_token()?;
        if token != Token::Fun {
            return Err(Error::ExpectedFun);
        }
        self.advance();

        token = self.get_token()?;
        if token != Token::Identifier {
            return Err(Error::ExpectedIdentifier);
        }
        let fn_name = self.get_value()?;
        self.advance();

        token = self.get_token()?;
        if token != Token::OpenParan {
            return Err(Error::ExpectedOpenParan);
        }
        self.advance();

        let fn_args = self.parse_fn_args()?;

        token = self.get_token()?;
        let mut ret_type = Type::Void;
        if token == Token::Tilde {
            self.advance();
            ret_type = self.parse_type()?;
        }

        token = self.get_token()?;
        let mut stmt_list = None;
        if token == Token::OpenBlock {
            self.advance();
            stmt_list = Some(self.parse_stmt_list(&[ Token::CloseBlock ])?);
        } else if token == Token::Semicolon {
            self.advance();
        } else {
            return Err(Error::ExpectedSemicolon);
        }

        Ok(InterfaceFunction {
            name: fn_name,
            returns: ret_type,
            body: stmt_list,
            arguments: fn_args
        })
    }

    /// Parses an import declaration
    pub fn parse_decl_import(&mut self) -> Result<Declaration> {
        let token = self.get_token()?;
        if token != Token::Import {
            return Err(Error::ExpectedImport);
        }
        self.advance();
    
        let import_list = self.parse_multi_import(&[Token::Semicolon])?;
        Ok(Declaration::Import(import_list))
    }

    /// Parses an enum declaration
    pub fn parse_decl_enum(&mut self) -> Result<Declaration> {
        let mut token = self.get_token()?;
        if token != Token::Enum {
            return Err(Error::ExpectedEnum);
        }
        self.advance();

        token = self.get_token()?;
        if token != Token::Identifier {
            return Err(Error::ExpectedIdentifier);
        }
        let ident_string = self.get_value()?;
        self.advance();

        token = self.get_token()?;
        if token != Token::OpenBlock {
            return Err(Error::ExpectedOpenBlock);
        }
        self.advance();

        let variants = self.parse_enum_variants()?;

        Ok(Declaration::Enum {
            name: ident_string,
            variants,
        })
    }

    fn parse_enum_variants(&mut self) -> Result<Vec<EnumVariant>> {
        let mut enum_variants = vec![];
        while self.get_token()? != Token::CloseBlock {
            if self.get_token()? == Token::Comma && !enum_variants.is_empty() {
                self.advance();
            } else if !enum_variants.is_empty() {
                return Err(Error::ExpectedComma);
            }
            let enum_variant = self.parse_enum_variant()?;
            enum_variants.push(enum_variant);
        }
        self.advance();
        Ok(enum_variants)
    }

    #[allow(dead_code)]
    fn parse_enum_variant(&mut self) -> Result<EnumVariant> {
        let mut token = self.get_token()?;
        if token != Token::Identifier {
            return Err(Error::ExpectedIdentifier);
        }
        let ident_string = self.get_value()?;
        self.advance();

        token = self.get_token()?;

        let ret = match token {
            Token::OpenBlock => {
                self.advance();
                token = self.get_token()?;
                let mut members = BTreeMap::new();
                while token != Token::CloseBlock {
                    if token != Token::Identifier {
                        return Err(Error::ExpectedIdentifier);
                    }
                    let member_name = self.get_value()?;
                    self.advance();

                    token = self.get_token()?;
                    if token != Token::Colon {
                        return Err(Error::ExpectedColon);
                    }
                    self.advance();

                    let member_type = self.parse_type()?;
                    members.insert(member_name, member_type);
                    token = self.get_token()?;
                    if token == Token::Comma {
                        self.advance();
                        token = self.get_token()?;
                    }
                }

                self.advance();

                EnumVariant::Cont(ident_string, members)
            }
            Token::OpenParan => {
                if let Type::Tuple(types) = self.parse_type()? {
                    token = self.get_token()?;
                    EnumVariant::Tuple(ident_string, types)
                } else {
                    return Err(Error::Unknown);
                }
            }
            Token::Comma => {
                EnumVariant::Empty(ident_string)
            }
            _ => {
                if token != Token::Comma && token != Token::CloseBlock {
                    return Err(Error::Unknown);
                }
                EnumVariant::Empty(ident_string)
            }
        };

        Ok(ret)
    }

    /// Parses a function declaration
    pub fn parse_decl_fn(&mut self) -> Result<Declaration> {
        let mut token = self.get_token()?;

        let mut external = false;
        let mut public = false;
        while [Token::Pub, Token::Ext].contains(&token) {
            external = token == Token::Ext;
            public = token == Token::Pub;
            self.advance();
        }

        if token != Token::Fun {
            return Err(Error::ExpectedFun);
        }
        self.advance();

        token = self.get_token()?;
        if token != Token::Identifier {
            return Err(Error::ExpectedIdentifier);
        }
        let ident_string = self.get_value()?;
        self.advance();

        token = self.get_token()?;
        if token != Token::OpenParan {
            return Err(Error::ExpectedOpenParan);
        }
        self.advance();

        let fn_args = self.parse_fn_args()?;
        let mut ret_type = Type::Void;

        token = self.get_token()?;
        if token == Token::Tilde {
            self.advance();
            ret_type = self.parse_type()?;
        }

        token = self.get_token()?;
        let stmt_list: Option<Vec<Statement>> = match token {
            Token::Semicolon => {
                self.advance();
                None
            }
            Token::OpenBlock => {
                self.advance();
                let stmt_list = self.parse_stmt_list(&[Token::CloseBlock])?;
                Some(stmt_list)
            }
            _ => return Err(Error::Unknown),
        };

        Ok(Declaration::Function {
            public,
            external,
            name: ident_string,
            arguments: fn_args,
            returns: ret_type,
            body: stmt_list,
        })
    }

    fn parse_multi_import(&mut self, delims: &[Token]) -> Result<Vec<(String, String)>> {
        let mut token = self.get_token()?;

        let mut ret = Vec::new();

        while !delims.contains(&token) {
            let (import_path, mut import_as) = self.parse_import_string(&[
                Token::Semicolon,
                Token::Comma,
                Token::OpenBlock,
                Token::CloseBlock,
            ])?;
            token = self.get_token()?;
            if token == Token::Comma {
                self.advance();
                token = self.get_token()?;
            }
            match token {
                Token::Assign => {
                    self.advance();
                    token = self.get_token()?;
                    if token != Token::Identifier {
                        return Err(Error::ExpectedIdentifier);
                    }
                    import_as = self.get_value()?;
                    self.advance();

                    ret.push((import_path, import_as));

                    token = self.get_token()?;
                    if token == Token::Comma {
                        self.advance();
                        token = self.get_token()?;
                    }
                }
                Token::OpenBlock => {
                    self.advance();
                    if !import_path.ends_with("::") {
                        return Err(Error::MalformedImport);
                    }

                    let mut nested_imports = self.parse_multi_import(&[Token::CloseBlock])?;

                    for (imp_path, _) in nested_imports.iter_mut() {
                        *imp_path = format!("{}{}", import_path, imp_path);
                    }
                    self.advance();

                    token = self.get_token()?;
                    if token == Token::Comma {
                        self.advance();
                        token = self.get_token()?;
                    }

                    ret.append(&mut nested_imports);
                }
                _ => {
                    ret.push((import_path, import_as));
                }
            };
        }

        Ok(ret)
    }

    fn parse_import_string(&mut self, delims: &[Token]) -> Result<(String, String)> {
        let mut import_path = String::new();
        let mut import_as = String::new();
        let mut token = self.get_token()?;

        while !delims.contains(&token) {
            match token {
                Token::Identifier => {
                    import_path += &self.get_value()?;
                    self.advance();
                }
                Token::DoubleColon => {
                    import_path += "::";
                    self.advance();
                }
                Token::Assign => {
                    self.advance();
                    token = self.get_token()?;
                    if token != Token::Identifier {
                        return Err(Error::ExpectedIdentifier);
                    }
                    import_as = self.get_value()?;
                    self.advance();
                }
                _ => return Err(Error::MalformedImport),
            };
            token = self.get_token()?;
        }

        if import_path.is_empty() {
            return Err(Error::MalformedImport);
        }

        if import_as.is_empty() && !import_path.ends_with("::") {
            let last_opt = import_path.split("::").last();
            if let Some(last) = last_opt {
                import_as += last;
            } else {
                import_as = import_path.clone();
            }
        }

        Ok((import_path, import_as))
    }

    /// Parses a function declarations arguments
    fn parse_fn_args(&mut self) -> Result<Vec<(String, Type)>> {
        let mut args = vec![];
        let mut token = self.get_token()?;
        while token != Token::CloseParan {
            let fn_arg = self.parse_fn_arg()?;
            args.push(fn_arg);
            token = self.get_token()?;
            if token == Token::Comma {
                self.advance();
            } else if token != Token::CloseParan {
                return Err(Error::ExpectedCloseParan);
            }
        }
        self.advance();
        Ok(args)
    }

    /// Parses a function declarations argument
    fn parse_fn_arg(&mut self) -> Result<(String, Type)> {
        let mut token = self.get_token()?;
        if [Token::This, Token::ThisRef].contains(&token) {
            self.advance();
            return match token {
                Token::This => Ok((String::from("this"), Type::This)),
                Token::ThisRef => Ok((String::from("this"), Type::Ref(Box::new(Type::This)))),
                _ => Err(Error::Unknown)
            };
        }
        if token != Token::Identifier {
            return Err(Error::ExpectedIdentifier);
        }
        let ident_string = self.get_value()?;
        self.advance();

        if &ident_string == "this" {

        } else {
            token = self.get_token()?;
            if token != Token::Colon {
                return Err(Error::ExpectedColon);
            }
            self.advance();
        }

        

        let var_type = self.parse_type()?;
        Ok((ident_string, var_type))
    }

    fn parse_type(&mut self) -> Result<Type> {
        let mut token = self.get_token()?;
        let ret = match token {
            Token::OpenParan => {
                self.advance();
                let mut types = vec![];
                token = self.get_token()?;
                while token != Token::CloseParan {
                    let var_type = self.parse_type()?;
                    token = self.get_token()?;
                    if token == Token::Comma {
                        self.advance();
                    } else if token != Token::CloseParan {
                        return Err(Error::ExpectedType);
                    }
                    types.push(var_type);
                }
                self.advance();
                Type::Tuple(types)
            }
            Token::Ref => {
                self.advance();
                let inner_type = self.parse_type()?;
                Type::Ref(Box::new(inner_type))
            }
            Token::Identifier => {
                let ident_value = self.get_value()?;
                self.advance();
                Type::Named(ident_value)
            }
            Token::PrimitiveType => {
                let token_val = self.get_value()?;
                self.advance();
                match token_val.as_str() {
                    "int" => Type::Int,
                    "float" => Type::Float,
                    "bool" => Type::Bool,
                    "string" => Type::String,
                    _ => return Err(Error::ExpectedType),
                }
            }
            _ => return Err(Error::ExpectedType),
        };
        Ok(ret)
    }

    /// Parses a module declaration
    pub fn parse_decl_mod(&mut self) -> Result<Declaration> {
        Err(Error::Unimplemented("mod decl"))
    }

    /// Parses a container declaration
    pub fn parse_decl_cont(&mut self) -> Result<Declaration> {
        let mut token = self.get_token()?;

        let mut public = false;
        if token == Token::Pub {
            public = true;
            self.advance();
            token = self.get_token()?;
        }

        if token != Token::Cont {
            return Err(Error::ExpectedCont);
        }
        self.advance();
        
        token = self.get_token()?;
        if token != Token::Identifier {
            return Err(Error::ExpectedIdentifier);
        }
        let cont_name = self.get_value()?;
        self.advance();

        token = self.get_token()?;
        if token != Token::OpenBlock {
            return Err(Error::ExpectedOpenBlock);
        }
        self.advance();

        token = self.get_token()?;

        let mut member_vars = BTreeMap::new();
        let mut member_fns = vec![];

        while token != Token::CloseBlock {
            let mut public = false;
            if token == Token::Pub {
                public = true;
                self.advance();
                token = self.get_token()?;
            }
            match token {
                Token::Identifier => {
                    let member_var = self.parse_cont_member_var(public)?;
                    member_vars.insert(member_var.0, member_var.1);
                }
                Token::Fun => {
                    let member_fn = self.parse_cont_member_fun(public)?;
                    member_fns.push(member_fn);
                },
                _ => return Err(Error::Unknown)
            }
            token = self.get_token()?;
        }

        self.advance();

        Ok(Declaration::Container {
            public,
            name: cont_name,
            member_functions: member_fns,
            member_variables: member_vars
        })

    }

    /// Parses a container member function
    pub fn parse_cont_member_fun(&mut self, public: bool) -> Result<ContainerFunction> {
        let mut token = self.get_token()?;
        if token != Token::Fun {
            return Err(Error::ExpectedFun);
        }
        self.advance();

        token = self.get_token()?;
        if token != Token::Identifier {
            return Err(Error::ExpectedIdentifier);
        }
        let fn_name = self.get_value()?;
        self.advance();

        token = self.get_token()?;
        if token != Token::OpenParan {
            return Err(Error::ExpectedOpenParan);
        }
        self.advance();

        let fn_args = self.parse_fn_args()?;

        token = self.get_token()?;
        let ret_type = if token == Token::Tilde {
            self.advance();
            token = self.get_token()?;
            self.parse_type()?
        } else {
            Type::Void
        };

        token = self.get_token()?;
        if token != Token::OpenBlock {
            return Err(Error::ExpectedOpenBlock);
        }
        self.advance();

        let stmt_list = self.parse_stmt_list(&[Token::CloseBlock])?;

        Ok(ContainerFunction {
            name: fn_name,
            arguments: fn_args,
            public,
            body: stmt_list,
            returns: ret_type
        })
    }

    /// Parses a container member variable
    pub fn parse_cont_member_var(&mut self, _public: bool) -> Result<(String, Type)> {
        let mut token = self.get_token()?;
        if token != Token::Identifier {
            return Err(Error::ExpectedIdentifier);
        }
        let var_name = self.get_value()?;
        self.advance();

        token = self.get_token()?;
        if token != Token::Colon {
            return Err(Error::ExpectedColon);
        }
        self.advance();

        let var_type = self.parse_type()?;
        
        token = self.get_token()?;
        if token != Token::Semicolon {
            return Err(Error::ExpectedSemicolon);
        }
        self.advance();

        Ok((var_name, var_type))
    }

    /// Parses a statement list, breaking on a set of given delimiters
    pub fn parse_stmt_list(&mut self, delims: &[Token]) -> Result<Vec<Statement>> {
        let mut statements = vec![];
        while self.token_pos < self.tokens.len() {
            let token = self.get_token()?;
            if delims.contains(&token) {
                self.advance();
                break;
            }
            let stmt = self.parse_stmt()?;
            statements.push(stmt);
        }
        Ok(statements)
    }

    /// Parses a single statement
    pub fn parse_stmt(&mut self) -> Result<Statement> {
        let token = self.get_token()?;
        match token {
            Token::Var => self.parse_stmt_var_decl(),
            Token::On => self.parse_stmt_on(),
            Token::While => self.parse_stmt_while(),
            Token::Yield => {
                self.advance();
                let _next_token = self.get_token()?;
                let expr_opt = if token != Token::Semicolon {
                    let expr = self.parse_expr(&[Token::Semicolon])?;
                    Some(expr)
                } else {
                    None
                };
                if !self.yield_stack.is_empty() {
                    if let Some(expr) = expr_opt.as_ref().cloned() {
                        let expr_ref = self.yield_stack.get_mut(0).ok_or(Error::Unknown)?;
                        *expr_ref = Some(expr);
                    }
                }
                Ok(Statement::Yield(expr_opt))
            }
            Token::Return => {
                self.advance();
                let _next_token = self.get_token()?;
                let expr_opt = if token != Token::Semicolon {
                    let expr = self.parse_expr(&[Token::Semicolon])?;
                    Some(expr)
                } else {
                    None
                };
                Ok(Statement::Return(expr_opt))
            }
            Token::Continue => {
                self.advance();
                let next_token = self.get_token()?;
                if next_token != Token::Semicolon {
                    Err(Error::ExpectedSemicolon)
                } else {
                    Ok(Statement::Continue)
                }
            }
            Token::Break => {
                self.advance();
                let next_token = self.get_token()?;
                if next_token != Token::Semicolon {
                    Err(Error::ExpectedSemicolon)
                } else {
                    Ok(Statement::Break)
                }
            }
            _ => {
                let expr = self.parse_expr(&[Token::Semicolon])?;
                Ok(Statement::ExpressionStmt(expr))
            }
        }
    }

    /// Parses a variable declaration statement
    pub fn parse_stmt_var_decl(&mut self) -> Result<Statement> {
        let mut token = self.get_token()?;
        if token != Token::Var {
            return Err(Error::ExpectedVar);
        }
        self.advance();

        token = self.get_token()?;
        if token != Token::Identifier {
            return Err(Error::ExpectedIdentifier);
        }
        let var_name = self.get_value()?;
        self.advance();

        token = self.get_token()?;
        let var_type = match token {
            Token::Colon => {
                self.advance();
                self.parse_type()?
            }
            _ => Type::Auto,
        };

        token = self.get_token()?;
        if token != Token::Assign {
            return Err(Error::ExpectedAssign);
        }
        self.advance();

        let var_expr = self.parse_expr(&[Token::Semicolon])?;

        token = self.get_token()?;

        Ok(Statement::VarDeclaration {
            name: var_name,
            var_type,
            expr: var_expr,
        })
    }

    /// Parses a conditional statement
    pub fn parse_stmt_on(&mut self) -> Result<Statement> {
        let mut token = self.get_token()?;
        if token != Token::On {
            return Err(Error::ExpectedOn);
        }
        self.advance();

        let cond_expr = self.parse_expr(&[Token::OpenBlock])?;
        let cond_body = self.parse_stmt_list(&[Token::CloseBlock])?;
        let mut else_body: Vec<Statement> = vec![];
        let mut cond_chain: Vec<(Expression, Vec<Statement>)> = vec![];
        if self.token_pos < self.tokens.len() {
            token = self.get_token()?;
            while token == Token::Else {
                let next_token = self.peek_token(1)?;
                if next_token == Token::On {
                    self.token_pos += 2;
                    let else_if_expr = self.parse_expr(&[Token::OpenBlock])?;
                    let else_if_body = self.parse_stmt_list(&[Token::CloseBlock])?;
                    cond_chain.push((else_if_expr, else_if_body));
                    token = self.get_token()?;
                } else if next_token == Token::OpenBlock {
                    self.token_pos += 2;
                    else_body = self.parse_stmt_list(&[Token::CloseBlock])?;
                    break;
                }
            }
        }

        Ok(Statement::Condition {
            expr: cond_expr,
            cond_body,
            else_body,
            cond_chain,
        })
    }

    /// Parses a conditional expression
    pub fn parse_expr_on(&mut self) -> Result<Expression> {
        let mut token = self.get_token()?;
        if token != Token::On {
            return Err(Error::ExpectedOn);
        }
        self.advance();
        self.yield_stack.push_front(None);
        let cond_expr = self.parse_expr(&[Token::OpenBlock])?;
        let cond_body = self.parse_stmt_list(&[Token::CloseBlock])?;
        let mut else_body: Vec<Statement> = vec![];
        let mut cond_chain: Vec<(Expression, Vec<Statement>)> = vec![];
        if self.token_pos < self.tokens.len() {
            token = self.get_token()?;
            while token == Token::Else {
                let next_token = self.peek_token(1)?;
                if next_token == Token::On {
                    self.token_pos += 2;
                    let else_if_expr = self.parse_expr(&[Token::OpenBlock])?;
                    let else_if_body = self.parse_stmt_list(&[Token::CloseBlock])?;
                    cond_chain.push((else_if_expr, else_if_body));
                    token = self.get_token()?;
                } else if next_token == Token::OpenBlock {
                    self.token_pos += 2;
                    else_body = self.parse_stmt_list(&[Token::CloseBlock])?;
                    break;
                }
            }
        }

        let yield_expr = self.yield_stack.pop_front().ok_or(Error::Unknown)?;

        Ok(Expression::Condition {
            expr: Box::new(cond_expr),
            cond_body,
            else_body,
            cond_chain,
            yield_expr: yield_expr.map(|expr| Box::new(expr)),
        })
    }

    /// Parses a while statement
    pub fn parse_stmt_while(&mut self) -> Result<Statement> {
        let token = self.get_token()?;
        if token != Token::While {
            return Err(Error::ExpectedWhile);
        }
        self.advance();

        let while_expr = self.parse_expr(&[Token::OpenBlock])?;
        let while_body = self.parse_stmt_list(&[Token::CloseBlock])?;

        Ok(Statement::While(while_expr, while_body))
    }

    /// Parses an expression
    pub fn parse_expr(&mut self, delims: &[Token]) -> Result<Expression> {
        if let Token::On = self.get_token()? {
            return self.parse_expr_on();
        }

        let mut op_stack: VecDeque<Operator> = VecDeque::new();
        let mut out_queue: VecDeque<ExprOutput> = VecDeque::new();

        let _last_tokens: VecDeque<Token> = VecDeque::new();

        let mut paran_count = 0;

        let mut last_token = Token::Error;

        while self.token_pos < self.tokens.len() {
            // Read a token
            let token = self.get_token()?;
            if delims.contains(&token) || (token == Token::CloseParan && paran_count == 0) {
                self.advance();
                break;
            }
            // If it is an operator
            if let Some(mut op) = Operator::from(token.clone()) {
                match op {
                    // If its an "(", push it onto the operator stack
                    Operator::OpenParan => {
                        paran_count += 1;
                        op_stack.push_front(op);
                    }
                    // If its an ")"
                    Operator::CloseParan => {
                        paran_count -= 1;
                        let mut op = op_stack.pop_front().ok_or(Error::Unknown)?;
                        while op != Operator::OpenParan {
                            out_queue.push_back(ExprOutput::Operator(op.clone()));
                            if !op_stack.is_empty() {
                                op = op_stack.pop_front().ok_or(Error::Unknown)?;
                            }
                        }
                        if op != Operator::OpenParan {
                            return Err(Error::Unknown);
                        }
                    }
                    // Any other operator
                    _ => {
                        let op = &mut op;
                        while !op_stack.is_empty() {
                            let last_op_opt = Operator::from(last_token.clone());
                            if let Some(last_op) = last_op_opt {
                                if last_op != Operator::Assign {
                                    *op = match op {
                                        Operator::Minus => Operator::Neg,
                                        Operator::Plus => Operator::Pos,
                                        _ => return Err(Error::Unknown),
                                    };
                                }
                            }
                            let op_front = op_stack.pop_front().unwrap();
                            if op_front.prec() > op.prec() {
                                out_queue.push_back(ExprOutput::Operator(op_front));
                            } else {
                                op_stack.push_front(op_front);
                                break;
                            }
                        }
                        match op {
                            Operator::Plus => {
                                if Operator::from(last_token.clone()).is_none() {
                                    op_stack.push_front(op.clone());
                                } else {
                                    let op = Operator::Pos;
                                    op_stack.push_front(op);
                                }
                            }
                            Operator::Minus => {
                                if Operator::from(last_token.clone()).is_none() {
                                    op_stack.push_front(op.clone());
                                } else {
                                    let op = Operator::Neg;
                                    op_stack.push_front(op);
                                }
                            }
                            _ => op_stack.push_front(op.clone()),
                        };
                    }
                };
            }
            // If the token is not an operator it is an operand
            else {
                let expr = self.parse_expr_non_arithmetic(&token)?;
                out_queue.push_back(ExprOutput::Expression(expr));
            }
            self.advance();
            last_token = self.peek_token(-1)?;
        }

        let mut out_stack: VecDeque<Expression> = VecDeque::new();
        for op in op_stack {
            out_queue.push_back(ExprOutput::Operator(op));
        }

        let _expr: Option<Expression> = None;
        while out_queue.len() > 0 {
            let expr_output = out_queue.pop_front().unwrap();
            match expr_output {
                ExprOutput::Expression(expr) => out_stack.push_front(expr),
                ExprOutput::Operator(op) => {
                    if !op.unary() {
                        let rhs_expr = out_stack.pop_front().unwrap();
                        let lhs_expr = out_stack.pop_front().unwrap();
                        let expr = Expression::Binary(Box::new(lhs_expr), op, Box::new(rhs_expr));
                        out_stack.push_front(expr);
                    } else {
                        let op_expr = out_stack.pop_front().unwrap();
                        let expr = Expression::Unary(op, Box::new(op_expr));
                        out_stack.push_front(expr);
                    }
                }
            };
        }

        if out_stack.len() > 1 {
            return Err(Error::MalformedExpression);
        }

        out_stack.pop_front().ok_or(Error::MalformedExpression)
    }

    fn parse_expr_non_arithmetic(&mut self, token: &Token) -> Result<Expression> {
        let expr = match token {
            Token::IntLiteral => {
                let str_val = self.get_value()?;
                let int_val = str_val.parse().map_err(|_| Error::Unknown)?;
                Expression::IntLiteral(int_val)
            }
            Token::FloatLiteral => {
                let str_val = self.get_value()?;
                let float_val = str_val.parse().map_err(|_| Error::Unknown)?;
                Expression::FloatLiteral(float_val)
            }
            Token::BoolLiteral => {
                let str_val = self.get_value()?;
                let bool_val = str_val.parse().map_err(|_| Error::Unknown)?;
                Expression::BoolLiteral(bool_val)
            }
            Token::Identifier => {
                let before_pos = self.token_pos;
                self.parse_expr_call().or_else(|_e| {
                    self.token_pos = before_pos;
                    self.parse_expr_variable()
                })?
            }
            _ => return Err(Error::Unknown),
        };
        Ok(expr)
    }

    fn parse_expr_variable(&mut self) -> Result<Expression> {
        let token = self.get_token()?;
        if token != Token::Identifier {
            return Err(Error::ExpectedIdentifier);
        }
        let ident_string = self.get_value()?;
        Ok(Expression::Variable(ident_string))
    }

    fn parse_expr_call(&mut self) -> Result<Expression> {
        let mut token = self.get_token()?;
        if token != Token::Identifier {
            return Err(Error::ExpectedIdentifier);
        }
        let ident_string = self.get_value()?;
        self.advance();

        token = self.get_token()?;
        if token != Token::OpenParan {
            return Err(Error::ExpectedOpenParan);
        }
        self.advance();
        let call_args = self.parse_expr_call_args()?;
        Ok(Expression::Call(ident_string, call_args))
    }

    fn parse_expr_call_args(&mut self) -> Result<Vec<Expression>> {
        let mut last_token;
        let mut args = Vec::new();
        while self.token_pos < self.tokens.len() {
            let arg_expr = self.parse_expr(&[Token::Comma, Token::CloseParan])?;
            args.push(arg_expr);
            last_token = self.peek_token(-1)?;
            if last_token == Token::Comma {
                continue;
            } else if last_token == Token::CloseParan {
                break;
            } else {
                return Err(Error::Unknown);
            }
        }
        self.token_pos -= 1;
        Ok(args)
    }
}

#[derive(Debug)]
enum ExprOutput {
    Expression(Expression),
    Operator(Operator),
}
