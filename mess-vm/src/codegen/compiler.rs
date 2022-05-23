use std::{
    collections::{
        HashMap,
        VecDeque,
    },
    result::Result as StdResult,
    error::Error as StdError
};

use mess_core::{
    codegen::{
        ctx::{
            FnContext,
            StackContext,
        },
        data::Data,
        decl::Declarator,
        def::{
            ContDef,
            FunctionDef,
            ModuleDef,
        },
    },
    compiler::Compiler as CompilerTrait,
    parser::ast::{
        Declaration,
        Expression,
        Operator,
        Statement,
        Type,
    },
    util::uid::UIDGenerator,
};

use crate::{
    codegen::{
        assembler::Assembler,
        error::{
            Error,
            Result,
        },
        instruction::Instruction,
        output::Output as OutputVM,
        register::Register,
    },
    exec::{
        core::CoreError,
        is::Opcode,
    },
};

pub struct Compiler {
    mod_def_stack: VecDeque<ModuleDef>,
    stack_ctx_stack: VecDeque<StackContext>,
    fn_ctx_stack: VecDeque<FnContext>,
    uid_gen: UIDGenerator,
    assembler: Assembler,
    declarator: Declarator,
}

impl CompilerTrait for Compiler {
    type Output = OutputVM;

    fn get_output(&mut self) -> Self::Output {
        let mut assembler = Assembler::new();
        std::mem::swap(&mut assembler, &mut self.assembler);
        let code = assembler.build();
        OutputVM::new()
            .with_code(code)
    }

    fn compile(&mut self, decl_list: &[Declaration]) -> StdResult<(), ()> {
        self.declarator.declare(decl_list)?;
        let (root_mod_def, _) = self.declarator.get_result()?;
        self.set_root_module(root_mod_def);
        self.compile_decl_list(decl_list).map_err(|_| ())
    }
}

impl Compiler {
    pub fn new() -> Self {
        let mut mod_def_stack = VecDeque::new();
        Self {
            mod_def_stack,
            uid_gen: UIDGenerator::new(),
            stack_ctx_stack: VecDeque::new(),
            fn_ctx_stack: VecDeque::new(),
            assembler: Assembler::new(),
            declarator: Declarator::default(),
        }
    }

    pub fn set_root_module(&mut self, module_def: ModuleDef) {
        self.mod_def_stack.clear();
        self.mod_def_stack.push_front(module_def);
    }

    fn get_module_path(&self) -> Result<String> {
        let mut path = String::new();
        for mod_def in self.mod_def_stack.iter() {
            path += &mod_def.name;
            path += "::";
        }
        if path.is_empty() {
            return Err(Error::Unknown);
        }
        Ok(path)
    }

    fn get_stack_pos(&self) -> Result<i32> {
        let stack_ctx = self.stack_ctx_stack.get(0).ok_or(Error::Unknown)?;
        Ok(stack_ctx.stack_pos)
    }

    fn get_stack_extent(&self) -> Result<i32> {
        let stack_ctx = self.stack_ctx_stack.get(0).ok_or(Error::Unknown)?;
        Ok(stack_ctx.stack_extent)
    }

    fn get_stack_offset(&self) -> Result<i32> {
        Ok(-(self.get_stack_pos()? as i32))
    }

    fn inc_stack(&mut self, inc: isize) -> Result<()> {
        let stack_ctx = self.get_stack_ctx()?;
        stack_ctx.inc_stack(inc);
        Ok(())
    }

    fn dec_stack(&mut self, dec: isize) -> Result<()> {
        let stack_ctx = self.get_stack_ctx()?;
        stack_ctx.dec_stack(dec);
        Ok(())
    }

    fn get_stack_ctx(&mut self) -> Result<&mut StackContext> {
        self.stack_ctx_stack.get_mut(0).ok_or(Error::Unknown)
    }

    fn get_var_position(&self, name: &str) -> Result<i32> {
        let stack_ctx = self.stack_ctx_stack.get(0).ok_or(Error::Unknown)?;
        let raw = stack_ctx.get_var(name);
        Ok(raw.0)
    }

    fn get_var_type(&self, name: &str) -> Result<Type> {
        let stack_ctx = self.stack_ctx_stack.get(0).ok_or(Error::Unknown)?;
        let raw = stack_ctx.get_var(name);
        Ok(raw.1.clone())
    }

    fn get_current_module(&self) -> Result<&ModuleDef> {
        self.mod_def_stack.get(0).ok_or(Error::Unknown)
    }

    fn get_current_fn_ctx(&self) -> Result<&FnContext> {
        self.fn_ctx_stack.get(0).ok_or(Error::Unknown)
    }

    fn resolve_fn(&self, fn_name: &str) -> Result<&FunctionDef> {
        if fn_name.contains("::") {
            let mod_path: Vec<&str> = fn_name.split("::").collect();
            let mut mod_def = {
                let first_part = mod_path[0];
                let front_mod_def = &self.mod_def_stack[0];
                if front_mod_def.has_module(first_part) {
                    front_mod_def
                } else {
                    &self.mod_def_stack[self.mod_def_stack.len() - 1]
                }
            };

            for i in 1..mod_path.len() - 1 {
                let mod_name = mod_path[i];
                if mod_def.has_module(mod_name) {
                    mod_def = mod_def.get_module(mod_name).map_err(|_| Error::Unknown)?;
                }
            }

            let last_part = mod_path[mod_path.len() - 1];
            mod_def.get_function(last_part).map_err(|_| Error::Unknown)
        } else {
            let mod_def = &self.mod_def_stack[0];
            if mod_def.has_function(fn_name) {
                return mod_def.get_function(fn_name).map_err(|_| Error::Unknown);
            }
            Err(Error::Unknown)
        }
    }

    fn resolve_mod(&self, name: &str) -> Result<&ModuleDef> {
        Err(Error::Unknown)
    }

    fn resolve_cont(&self, name: &str) -> Result<&ContDef> {
        Err(Error::Unknown)
    }

    fn get_next_uid(&mut self) -> u64 {
        self.uid_gen.generate()
    }

    fn get_size_of_type(&self, var_type: &Type) -> Result<usize> {
        match var_type {
            Type::Int => Ok(8),
            Type::Float => Ok(4),
            Type::Ref(_) => Ok(8),
            Type::Bool => Ok(1),
            _ => return Err(Error::UnknownType(var_type.clone())),
        }
    }

    pub fn compile_decl_list(&mut self, decl_list: &[Declaration]) -> Result<()> {
        for decl in decl_list {
            self.compile_decl(decl)?;
        }
        Ok(())
    }

    pub fn compile_decl(&mut self, decl: &Declaration) -> Result<()> {
        match decl {
            Declaration::Function { .. } => self.compile_decl_fn(decl),
            Declaration::Container { .. } => self.compile_decl_cont(decl),
            Declaration::Module { .. } => self.compile_decl_mod(decl),
            Declaration::Interface { .. } => self.compile_decl_intf(decl),
            Declaration::Import(..) => self.compile_decl_import(decl),
            _ => Err(Error::Unknown),
        }
    }

    pub fn compile_decl_import(&mut self, decl: &Declaration) -> Result<()> {
        Err(Error::Unimplemented("Import declaration"))
    }

    pub fn compile_decl_mod(&mut self, decl: &Declaration) -> Result<()> {
        Err(Error::Unimplemented("Module declaration"))
    }

    pub fn compile_decl_intf(&mut self, decl: &Declaration) -> Result<()> {
        Err(Error::Unimplemented("Interface declaration"))
    }

    pub fn compile_decl_cont(&mut self, decl: &Declaration) -> Result<()> {
        Err(Error::Unimplemented("Container declaration"))
    }

    pub fn compile_decl_fn(&mut self, decl: &Declaration) -> Result<()> {
        let (name, ret_type, args, stmt_list) = match decl {
            Declaration::Function {
                external,
                name,
                arguments,
                returns,
                body,
            } => {
                if body.is_none() {
                    return Ok(());
                }
                (name, returns, arguments, body.as_ref().unwrap())
            }
            _ => return Err(Error::Unknown),
        };
        let full_fn_name = self.get_module_path()? + "::" + name;
        self.assembler.push_label(full_fn_name);
        // MOVA sp, bp
        let mov_rbp_rsp = Instruction::new(Opcode::MOVA)
            .with_operand::<u8>(Register::SP.into())
            .with_operand::<u8>(Register::BP.into());
        self.assembler.push_instr(mov_rbp_rsp);
        // Tag for referencing the stack inc instruction
        let stack_inc_tag = self.assembler.new_tag();
        let mut stack_inc_instr = Instruction::new_inc_stack(0);
        self.assembler.push_instr(stack_inc_instr);

        // Create a new stack context
        let stack_ctx_uid = self.uid_gen.generate();
        let mut stack_ctx = StackContext::new(stack_ctx_uid);

        // Insert function arguments into this context
        for (arg_name, arg_type) in args {
            let mut offset = 0;
            for (other_arg_name, other_arg_type) in args.iter().rev() {
                if arg_name == other_arg_name {
                    break;
                }
                let size = self.get_size_of_type(other_arg_type)?;
                offset -= size as i32;
            }
            stack_ctx.set_var(offset, arg_name, arg_type);
        }

        // Push a new function context
        let fn_ctx = FnContext::new(ret_type.clone(), stack_ctx_uid);
        self.stack_ctx_stack.push_front(stack_ctx);
        self.fn_ctx_stack.push_front(fn_ctx);

        // Compile the functions statement list
        self.compile_stmt_list(stmt_list)?;

        // Retrieve the stack context
        stack_ctx = self.stack_ctx_stack.pop_front().ok_or(Error::Unknown)?;

        // Get the biggest stack size
        let stack_size = stack_ctx.stack_extent;
        // Create new stack inc instruction with this size
        stack_inc_instr = Instruction::new_inc_stack(stack_size as usize);
        let stack_inc_instr_pos = self
            .assembler
            .get_tag(&stack_ctx_uid)
            .ok_or(Error::Unknown)?
            .pop()
            .ok_or(Error::Unknown)?;
        // Replace the tagged instruction with this one
        let stack_inc_instr_ref = self
            .assembler
            .get_instr(&stack_inc_instr_pos)
            .ok_or(Error::Unknown)?;
        *stack_inc_instr_ref = stack_inc_instr;

        self.assembler.push_instr(Instruction::new(Opcode::HALT));

        unimplemented!("TODO: Implement");
    }

    pub fn compile_stmt_list(&mut self, stmt_list: &[Statement]) -> Result<()> {
        for stmt in stmt_list {
            self.compile_stmt(stmt)?;
        }
        Ok(())
    }

    pub fn compile_stmt(&mut self, stmt: &Statement) -> Result<()> {
        match stmt {
            Statement::VarDeclarationStmt { .. } => self.compile_stmt_var_decl(stmt)?,
            Statement::ExpressionStmt(_) => self.compile_stmt_expr(stmt)?,
            Statement::Return(_) => self.compile_stmt_return(stmt)?,
            _ => return Err(Error::Unknown),
        };
        Ok(())
    }

    pub fn compile_stmt_return(&mut self, stmt: &Statement) -> Result<()> {
        let expr_opt = match stmt {
            Statement::Return(expr_opt) => expr_opt,
            _ => return Err(Error::Unknown),
        };
        let (ret_type, suid) = {
            let fn_ctx = self.get_current_fn_ctx()?;
            (fn_ctx.ret_type.clone(), fn_ctx.stack_ctx_uid)
        };
        unimplemented!("TODO: Implement");
    }

    pub fn compile_stmt_var_decl(&mut self, stmt: &Statement) -> Result<()> {
        let (var_name, mut var_type, var_expr) = match stmt {
            Statement::VarDeclarationStmt {
                name,
                var_type,
                expr,
            } => (name, var_type.clone(), expr),
            _ => return Err(Error::Unknown),
        };
        let expr_type = self.get_expr_type(var_expr)?;
        if var_type == Type::Auto {
            var_type = expr_type;
        }
        let var_size = self.get_size_of_type(&var_type)?;
        self.compile_expr(var_expr)?;
        let mut var_pos = self.get_stack_pos()? - var_size as i32;
        let stack_ctx = self.get_stack_ctx()?;
        stack_ctx.set_var(var_pos, var_name, &var_type);
        Ok(())
    }

    pub fn compile_stmt_expr(&mut self, stmt: &Statement) -> Result<()> {
        let expr = match stmt {
            Statement::ExpressionStmt(expr) => expr,
            _ => return Err(Error::Unknown),
        };
        self.compile_expr(expr)
    }

    pub fn compile_expr(&mut self, expr: &Expression) -> Result<()> {
        let start_pos = self.get_stack_pos()?;
        let expr_type = self.get_expr_type(expr)?;
        let expr_size = self.get_size_of_type(&expr_type)?;
        match expr {
            Expression::Variable(var_name) => {
                let mut var_pos = self.get_var_position(var_name)?;
                let var_type = self.get_var_type(var_name)?;
                let var_size = self.get_size_of_type(&var_type)?;
                let target_pos = self.get_stack_pos()?;
                self.inc_stack(var_size as isize)?;
                self.asm_stack_copy(var_pos, target_pos, var_size)?;
            }
            Expression::IntLiteral(int_val) => {
                let pos = self.get_stack_pos()?;
                self.inc_stack(8)?;
                unimplemented!("TODO: Implement");
            }
            Expression::FloatLiteral(float_val) => {
                let float_int: i32 = unsafe { std::mem::transmute(*float_val) };
                let pos = self.get_stack_pos()?;
                self.inc_stack(4)?;
                unimplemented!("TODO: Implement");
            }
            Expression::Condition { .. } => {
                self.compile_expr_cond(expr)?;
            }
            Expression::Call(fn_name, fn_args) => self.compile_expr_call(fn_name, fn_args)?,
            Expression::Unary(op, op_expr) => {
                match op {
                    Operator::Ref => self.compile_expr_ref(op_expr)?,
                    Operator::Deref => self.compile_expr_deref(op_expr)?,
                    _ => return Err(Error::Unknown),
                };
            }
            Expression::Binary(lhs_expr, op, rhs_expr) => {
                match op {
                    Operator::Plus => self.compile_expr_add(lhs_expr, rhs_expr)?,
                    Operator::Minus => self.compile_expr_sub(lhs_expr, rhs_expr)?,
                    Operator::Times => self.compile_expr_mul(lhs_expr, rhs_expr)?,
                    Operator::Divide => self.compile_expr_div(lhs_expr, rhs_expr)?,
                    Operator::Assign => self.compile_expr_assign(lhs_expr, rhs_expr)?,
                    Operator::AddAssign => self.compile_expr_add_assign(lhs_expr, rhs_expr)?,
                    Operator::SubAssign => self.compile_expr_sub_assign(lhs_expr, rhs_expr)?,
                    Operator::MulAssign => self.compile_expr_mul_assign(lhs_expr, rhs_expr)?,
                    Operator::DivAssign => self.compile_expr_div_assign(lhs_expr, rhs_expr)?,
                    _ => return Err(Error::Unknown),
                };
            }
            _ => return Err(Error::Unknown),
        };
        let end_pos = self.get_stack_pos()?;
        if end_pos - start_pos > (expr_size as i32) {
            let diff = ((end_pos - start_pos) as usize) - expr_size;
            let to_pos = start_pos;
            let from_pos = end_pos - expr_size as i32;
            self.asm_stack_copy(from_pos, to_pos, expr_size)?;
            self.dec_stack(diff as isize)?;
        }
        Ok(())
    }

    fn compile_expr_cond(&mut self, expr: &Expression) -> Result<()> {
        Err(Error::Unimplemented("Condition expr compilation"))
    }

    fn compile_expr_call(&mut self, fn_name: &String, fn_args: &[Expression]) -> Result<()> {
        Err(Error::Unimplemented("Call expression"))
    }

    fn compile_expr_deref(&mut self, expr: &Expression) -> Result<()> {
        Err(Error::Unimplemented("Deref expression"))
    }

    fn compile_expr_ref(&mut self, expr: &Expression) -> Result<()> {
        Err(Error::Unimplemented("Ref expression"))
    }

    fn compile_expr_add(&mut self, lhs_expr: &Expression, rhs_expr: &Expression) -> Result<()> {
        let expr_type = self.get_expr_type(lhs_expr)?;
        match expr_type {
            Type::Int => {
                let lhs_pos = self.get_stack_pos()?;
                self.compile_expr(lhs_expr)?;
                let rhs_pos = self.get_stack_pos()?;
                self.compile_expr(rhs_expr)?;
                let pos = self.get_stack_pos()?;
                unimplemented!("TODO: Implement");
                self.inc_stack(8)?;
                unimplemented!("TODO: Implement");
            }
            Type::Float => {
                let lhs_pos = self.get_stack_pos()?;
                self.compile_expr(lhs_expr)?;
                let rhs_pos = self.get_stack_pos()?;
                self.compile_expr(rhs_expr)?;
                let pos = self.get_stack_pos()?;
                unimplemented!("TODO: Implement");
                self.inc_stack(4)?;
                unimplemented!("TODO: Implement");
            }
            _ => return Err(Error::Unknown),
        };
        Ok(())
    }

    fn compile_expr_sub(&mut self, lhs_expr: &Expression, rhs_expr: &Expression) -> Result<()> {
        Err(Error::Unimplemented("Sub expression"))
    }

    fn compile_expr_mul(&mut self, lhs_expr: &Expression, rhs_expr: &Expression) -> Result<()> {
        let expr_type = self.get_expr_type(lhs_expr)?;
        match expr_type {
            Type::Int => {
                let lhs_pos = self.get_stack_pos()?;
                self.compile_expr(lhs_expr)?;
                let rhs_pos = self.get_stack_pos()?;
                self.compile_expr(rhs_expr)?;
                let pos = self.get_stack_pos()?;
                unimplemented!("TODO: Implement");
                self.inc_stack(8)?;
                unimplemented!("TODO: Implement");
            }
            Type::Float => {
                let lhs_pos = self.get_stack_pos()?;
                self.compile_expr(lhs_expr)?;
                let rhs_pos = self.get_stack_pos()?;
                self.compile_expr(rhs_expr)?;
                let pos = self.get_stack_pos()?;
                unimplemented!("TODO: Implement");
                self.inc_stack(4)?;
                unimplemented!("TODO: Implement");
            }
            _ => return Err(Error::Unknown),
        };
    }

    fn compile_expr_div(&mut self, lhs_expr: &Expression, rhs_expr: &Expression) -> Result<()> {
        Err(Error::Unimplemented("Division expression"))
    }

    fn compile_expr_assign(&mut self, lhs_expr: &Expression, rhs_expr: &Expression) -> Result<()> {
        Err(Error::Unimplemented("Assign expression"))
    }

    fn compile_expr_add_assign(
        &mut self,
        lhs_expr: &Expression,
        rhs_expr: &Expression,
    ) -> Result<()> {
        Err(Error::Unimplemented("Add assign expression"))
    }

    fn compile_expr_sub_assign(
        &mut self,
        lhs_expr: &Expression,
        rhs_expr: &Expression,
    ) -> Result<()> {
        Err(Error::Unimplemented("Sub assign expression"))
    }

    fn compile_expr_mul_assign(
        &mut self,
        lhs_expr: &Expression,
        rhs_expr: &Expression,
    ) -> Result<()> {
        Err(Error::Unimplemented("Mul assign expression"))
    }

    fn compile_expr_div_assign(
        &mut self,
        lhs_expr: &Expression,
        rhs_expr: &Expression,
    ) -> Result<()> {
        Err(Error::Unimplemented("Div assign expression"))
    }

    fn asm_stack_ret(&mut self, stack_ctx_uid: u64) -> Result<()> {
        let mut pop_size = 0;
        for stack_ctx in self.stack_ctx_stack.iter() {
            pop_size += stack_ctx.stack_extent;
            if stack_ctx.uid == stack_ctx_uid {
                break;
            }
        }
        self.dec_stack(pop_size as isize)?;
        unimplemented!("TODO: Implement");
        Ok(())
    }

    fn asm_args_to_stack(&mut self, fn_args: &[(String, Type)]) -> Result<()> {
        let arg_size: isize = fn_args
            .iter()
            .map(|(_, arg_type)| self.get_size_of_type(arg_type).unwrap() as isize)
            .sum();
        self.inc_stack(arg_size)?;
        let mut offset = 0;
        let mut int_reg_ctr = 0;
        let mut float_reg_ctr = 0;
        for (arg_name, arg_type) in fn_args.iter() {
            let arg_size = self.get_size_of_type(arg_type)?;
            match arg_type {
                Type::Int => {
                    match int_reg_ctr {
                        0 => unimplemented!("TODO: Implement"),
                        1 => unimplemented!("TODO: Implement"),
                        2 => unimplemented!("TODO: Implement"),
                        3 => unimplemented!("TODO: Implement"),
                        _ => return Err(Error::Unknown),
                    };
                    int_reg_ctr += 1;
                    self.get_stack_ctx()?.set_var(offset, arg_name, arg_type);
                    offset += 8;
                }
                Type::Float => {
                    match float_reg_ctr {
                        0 => unimplemented!("TODO: Implement"),
                        1 => unimplemented!("TODO: Implement"),
                        2 => unimplemented!("TODO: Implement"),
                        3 => unimplemented!("TODO: Implement"),
                        _ => return Err(Error::Unknown),
                    };
                    float_reg_ctr += 1;
                    self.get_stack_ctx()?.set_var(offset, arg_name, arg_type);
                    offset += 4;
                }
                _ => return Err(Error::Unimplemented("NOT IMPLEMENTED")),
            };
        }
        Ok(())
    }

    fn asm_stack_copy(&mut self, from_offset: i32, to_offset: i32, n: usize) -> Result<()> {
        let qword_copies = n / 8;
        let qword_rest = n % 8;
        let dword_copies = qword_rest / 4;
        let dword_rest = qword_rest % 4;
        let word_copies = dword_rest / 2;
        let word_rest = dword_rest % 2;
        let byte_copies = word_rest;
        let mut offset = 0;
        for _ in 0..qword_copies {
            let pos = from_offset + offset;
            let target_pos = to_offset + offset;
            unimplemented!("TODO: Implement");
            offset += 8;
        }

        if offset > to_offset {
            return Err(Error::Unknown);
        }

        for _ in 0..dword_copies {
            let pos = from_offset + offset;
            let target_pos = to_offset + offset;
            unimplemented!("TODO: Implement");
            offset += 4;
        }

        for _ in 0..word_copies {
            let pos = from_offset + offset;
            let target_pos = to_offset + offset;
            unimplemented!("TODO: Implement");
            offset += 2;
        }

        for _ in 0..byte_copies {
            let pos = from_offset + offset;
            let target_pos = to_offset + offset;
            unimplemented!("TODO: Implement");
            offset += 1;
        }

        Ok(())
    }

    fn get_expr_type(&self, expr: &Expression) -> Result<Type> {
        let ret = match expr {
            Expression::IntLiteral(_) => Type::Int,
            Expression::BoolLiteral(_) => Type::Bool,
            Expression::FloatLiteral(_) => Type::Float,
            Expression::Variable(var_name) => self.get_var_type(var_name)?,
            Expression::StringLiteral(_) => Type::Ref(Box::new(Type::String)),
            Expression::Condition { .. } => self.get_expr_type_cond(expr)?,
            Expression::Unary(op, op_expr) => match op {
                Operator::Not => Type::Bool,
                Operator::Ref => {
                    let expr_type = self.get_expr_type(op_expr)?;
                    Type::Ref(Box::new(expr_type))
                }
                Operator::Deref => {
                    let expr_type = self.get_expr_type(op_expr)?;
                    match expr_type {
                        Type::Ref(ret) => *ret,
                        _ => return Err(Error::Unknown),
                    }
                }
                _ => {
                    let op_type = self.get_expr_type(op_expr)?;
                    op_type
                }
            },
            Expression::Binary(lhs_expr, op, rhs_expr) => {
                let lhs_type = self.get_expr_type(lhs_expr)?;
                let rhs_type = self.get_expr_type(rhs_expr)?;
                if rhs_type != lhs_type {
                    return Err(Error::TypeMismatch(lhs_type, rhs_type));
                }
                lhs_type
            }
            _ => return Err(Error::Unknown),
        };
        Ok(ret)
    }

    fn get_expr_type_cond(&self, expr: &Expression) -> Result<Type> {
        match expr {
            Expression::Condition {
                yield_expr: yield_expr_opt,
                ..
            } => match yield_expr_opt {
                Some(yield_expr) => self.get_expr_type(yield_expr),
                None => Ok(Type::Void),
            },
            _ => Err(Error::Unknown),
        }
    }
}
