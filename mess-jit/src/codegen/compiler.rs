#![allow(unused_must_use)]

use std::collections::{
    HashMap,
    VecDeque,
};

use dynasmrt::{
    dynasm,
    x64::Assembler,
    AssemblyOffset,
    DynamicLabel,
    DynasmApi,
    DynasmLabelApi,
};
use mess_core::{
    codegen::{
        ctx::{
            FnContext,
            StackContext,
        },
        def::{
            ContDef,
            FunctionDef,
            ModuleDef,
        },
    },
    parser::ast::{
        Declaration,
        Expression,
        Operator,
        Statement,
        Type,
    },
};

use super::{
    error::{
        Error,
        Result,
    },
    output::Output,
};

pub struct Compiler {
    mod_def_stack: VecDeque<ModuleDef>,
    stack_ctx_stack: VecDeque<StackContext>,
    fn_ctx_stack: VecDeque<FnContext>,
    label_map: HashMap<u64, DynamicLabel>,
    uid_counter: u64,
    assembler: Assembler,
}

impl Compiler {
    pub fn new() -> Self {
        let mod_def_stack = VecDeque::new();
        Self {
            mod_def_stack,
            uid_counter: 0,
            stack_ctx_stack: VecDeque::new(),
            fn_ctx_stack: VecDeque::new(),
            label_map: HashMap::new(),
            assembler: Assembler::new().expect("Couldnt create x64 JIT assembler!"),
        }
    }

    pub fn set_root_module(&mut self, module_def: ModuleDef) {
        self.mod_def_stack.clear();
        self.mod_def_stack.push_front(module_def);
    }

    pub fn set_uid_counter(&mut self, counter: u64) {
        self.uid_counter = counter;
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
                    mod_def = mod_def.get_module(mod_name)?;
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

    fn resolve_mod(&self, _name: &str) -> Result<&ModuleDef> {
        Err(Error::Unknown)
    }

    fn resolve_cont(&self, _name: &str) -> Result<&ContDef> {
        Err(Error::Unknown)
    }

    fn get_next_uid(&mut self) -> u64 {
        let ret = self.uid_counter;
        self.uid_counter += 1;
        ret
    }

    fn get_dynamic_label(&mut self, uid: &u64) -> Result<DynamicLabel> {
        if let Some(dyn_label) = self.label_map.get(uid) {
            Ok(*dyn_label)
        } else {
            let dyn_label = self.assembler.new_dynamic_label();
            self.label_map.insert(*uid, dyn_label);
            Ok(dyn_label)
        }
    }

    fn get_dynamic_label_raw(&self, uid: &u64) -> Result<DynamicLabel> {
        self.label_map.get(uid).cloned().ok_or(Error::Unknown)
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
            &Declaration::Function { .. } => self.compile_decl_fn(decl),
            _ => Err(Error::Unknown),
        }
    }

    pub fn compile_decl_fn(&mut self, decl: &Declaration) -> Result<()> {
        let (name, ret_type, args, stmt_list) = match decl {
            Declaration::Function {
                external: false,
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
        let fn_uid = self.get_current_module()?.get_function(name)?.label_uid;
        let fn_label = self.get_dynamic_label(&fn_uid)?;
        dynasm!(&mut self.assembler
            ; =>fn_label
            ; push rbp
            ; mov rbp, rsp
        );
        let stack_init_offset = self.assembler.offset();
        dynasm!(self.assembler; sub rsp, DWORD 0);
        let mut stack_ctx = StackContext::new(self.get_next_uid());
        let fn_ctx = FnContext::new(ret_type.clone(), stack_ctx.uid);
        self.fn_ctx_stack.push_front(fn_ctx);
        self.stack_ctx_stack.push_front(stack_ctx);
        self.asm_args_to_stack(args)?;
        self.compile_stmt_list(stmt_list)?;
        stack_ctx = self.stack_ctx_stack.pop_front().ok_or(Error::Unknown)?;
        let mut modifier = self.assembler.alter_uncommitted();
        modifier.goto(stack_init_offset);
        dynasm!(modifier; sub rsp, DWORD stack_ctx.stack_extent);
        self.fn_ctx_stack.pop_front();
        Ok(())
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

        if expr_opt.is_some() {
            let expr = expr_opt.as_ref().unwrap();
            //println!("Ret expr: {:#?}", expr);
            let expr_type = self.get_expr_type(expr)?;
            if ret_type != expr_type {
                return Err(Error::TypeMismatch(ret_type, expr_type));
            }
            let pos = self.get_stack_pos()?;
            self.compile_expr(expr)?;
            match expr_type {
                Type::Int => {
                    dynasm!(&mut self.assembler
                        ; mov rax, [rbp - pos]
                    );
                }
                Type::Float => {
                    dynasm!(&mut self.assembler
                        ; movss xmm0, DWORD [rbp - pos]
                    );
                }
                _ => {
                    return Err(Error::Unimplemented(
                        "Other return types not implemented yet!",
                    ))
                }
            };
        }

        self.asm_stack_ret(suid)?;
        dynasm!(&mut self.assembler
            ; pop rbp
            ; ret
        );

        Ok(())
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
        let var_pos = self.get_stack_pos()? - var_size as i32;
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
                let var_pos = self.get_var_position(var_name)?;
                let var_type = self.get_var_type(var_name)?;
                let var_size = self.get_size_of_type(&var_type)?;
                let target_pos = self.get_stack_pos()?;
                self.inc_stack(var_size as isize)?;
                self.asm_stack_copy(var_pos, target_pos, var_size)?;
            }
            Expression::IntLiteral(int_val) => {
                let pos = self.get_stack_pos()?;
                self.inc_stack(8)?;
                dynasm!(&mut self.assembler
                    ; mov r11, QWORD *int_val
                    ; mov QWORD [rbp - pos], r11
                );
            }
            Expression::FloatLiteral(float_val) => {
                let float_int: i32 = unsafe { std::mem::transmute(*float_val) };
                let pos = self.get_stack_pos()?;
                self.inc_stack(4)?;
                dynasm!(&mut self.assembler
                    ; mov DWORD [rbp - pos], DWORD float_int
                );
            }
            Expression::Call(fn_name, fn_args) => self.compile_expr_call(fn_name, fn_args)?,
            Expression::Unary(op, op_expr) => {
                match op {
                    Operator::Ref => self.compile_expr_ref(op_expr)?,
                    _ => return Err(Error::Unknown),
                };
            }
            Expression::Binary(lhs_expr, op, rhs_expr) => {
                match op {
                    Operator::Plus => self.compile_expr_add(lhs_expr, rhs_expr)?,
                    Operator::Minus => self.compile_expr_sub(lhs_expr, rhs_expr)?,
                    Operator::Times => self.compile_expr_mul(lhs_expr, rhs_expr)?,
                    Operator::Divide => self.compile_expr_div(lhs_expr, rhs_expr)?,
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

    fn compile_expr_call(&mut self, _fn_name: &String, _fn_args: &[Expression]) -> Result<()> {
        Ok(())
    }

    fn compile_expr_ref(&mut self, _expr: &Expression) -> Result<()> {
        Ok(())
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
                dynasm!(&mut self.assembler
                    ; mov rax, QWORD [rbp - lhs_pos]
                    ; mov rbx, QWORD [rbp - rhs_pos]
                    ; add rax, rbx
                );
                self.inc_stack(8)?;
                dynasm!(&mut self.assembler
                    ; mov QWORD [rbp - pos], rax
                );
            }
            Type::Float => {
                let lhs_pos = self.get_stack_pos()?;
                self.compile_expr(lhs_expr)?;
                let rhs_pos = self.get_stack_pos()?;
                self.compile_expr(rhs_expr)?;
                let pos = self.get_stack_pos()?;
                dynasm!(&mut self.assembler
                    ; movss xmm0, DWORD [rbp - lhs_pos]
                    ; movss xmm1, DWORD [rbp - rhs_pos]
                    ; addss xmm0, xmm1
                );
                self.inc_stack(4)?;
                dynasm!(&mut self.assembler
                    ; movss DWORD [rbp - pos], xmm0
                );
            }
            _ => return Err(Error::Unknown),
        };
        Ok(())
    }

    fn compile_expr_sub(&mut self, _lhs_expr: &Expression, _rhs_expr: &Expression) -> Result<()> {
        Ok(())
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
                dynasm!(&mut self.assembler
                    ; mov rax, QWORD [rbp - lhs_pos]
                    ; mov rbx, QWORD [rbp - rhs_pos]
                    ; mul rbx
                );
                self.inc_stack(8)?;
                dynasm!(&mut self.assembler
                    ; mov QWORD [rbp - pos], rdx
                );
            }
            Type::Float => {
                let lhs_pos = self.get_stack_pos()?;
                self.compile_expr(lhs_expr)?;
                let rhs_pos = self.get_stack_pos()?;
                self.compile_expr(rhs_expr)?;
                let pos = self.get_stack_pos()?;
                dynasm!(&mut self.assembler
                    ; movss xmm0, DWORD [rbp - lhs_pos]
                    ; movss xmm1, DWORD [rbp - rhs_pos]
                    ; mulss xmm0, xmm1
                );
                self.inc_stack(4)?;
                dynasm!(&mut self.assembler
                    ; movss DWORD [rbp - pos], xmm0
                );
            }
            _ => return Err(Error::Unknown),
        };
        Ok(())
    }

    fn compile_expr_div(&mut self, _lhs_expr: &Expression, _rhs_expr: &Expression) -> Result<()> {
        Ok(())
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
        dynasm!(self.assembler
            ; add rsp, DWORD pop_size
        );
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
            let _arg_size = self.get_size_of_type(arg_type)?;
            match arg_type {
                Type::Int => {
                    match int_reg_ctr {
                        0 => dynasm!(&mut self.assembler
                            ; mov QWORD [rbp - offset as _], rdi
                        ),
                        1 => dynasm!(&mut self.assembler
                            ; mov QWORD [rbp - offset as _], rsi
                        ),
                        2 => dynasm!(&mut self.assembler
                            ; mov QWORD [rbp - offset as _], rdx
                        ),
                        3 => dynasm!(&mut self.assembler
                            ; mov QWORD [rbp - offset as _], rcx
                        ),
                        _ => return Err(Error::Unknown),
                    };
                    int_reg_ctr += 1;
                    self.get_stack_ctx()?.set_var(offset, arg_name, arg_type);
                    offset += 8;
                }
                Type::Float => {
                    match float_reg_ctr {
                        0 => dynasm!(&mut self.assembler
                            ; movss DWORD [rbp - offset as _], xmm0
                        ),
                        1 => dynasm!(&mut self.assembler
                            ; movss DWORD [rbp - offset as _], xmm1
                        ),
                        2 => dynasm!(&mut self.assembler
                            ; movss DWORD [rbp - offset as _], xmm2
                        ),
                        3 => dynasm!(&mut self.assembler
                            ; movss DWORD [rbp - offset as _], xmm3
                        ),
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
            dynasm!(&mut self.assembler
                ; mov r11, QWORD [rbp - pos as _]
                ; mov QWORD [rbp - target_pos as _], r11
            );
            offset += 8;
        }

        if offset > to_offset {
            return Err(Error::Unknown);
        }

        for _ in 0..dword_copies {
            let pos = from_offset + offset;
            let target_pos = to_offset + offset;
            dynasm!(&mut self.assembler
                ; mov r11d, DWORD [rbp - pos as _]
                ; mov DWORD [rbp - target_pos as _], r11d
            );
            offset += 4;
        }

        for _ in 0..word_copies {
            let pos = from_offset + offset;
            let target_pos = to_offset + offset;
            dynasm!(&mut self.assembler
                ; mov r11w, WORD [rbp - pos as _]
                ; mov WORD [rbp - target_pos as _], r11w
            );
            offset += 2;
        }

        for _ in 0..byte_copies {
            let pos = from_offset + offset;
            let target_pos = to_offset + offset;
            dynasm!(&mut self.assembler
                ; mov r11b, BYTE [rbp - pos as _]
                ; mov BYTE [rbp - target_pos as _], r11b
            );
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
            Expression::Binary(lhs_expr, _op, rhs_expr) => {
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

    fn build_fn_map(
        &self,
        mod_def: &ModuleDef,
        fn_map: &mut HashMap<String, AssemblyOffset>,
    ) -> Result<()> {
        for (uid, name) in mod_def.get_function_list()? {
            println!("Fn: {}", name);
            let dyn_label = self.get_dynamic_label_raw(&uid)?;
            let asm_offset = self
                .assembler
                .labels()
                .resolve_dynamic(dyn_label)
                .map_err(|_| Error::Unknown)?;
            fn_map.insert(name, asm_offset);
        }
        for (_, mod_def) in mod_def.modules.iter() {
            self.build_fn_map(mod_def, fn_map)?;
        }
        Ok(())
    }

    pub fn get_output(&mut self) -> Result<Output> {
        let mut fn_map = HashMap::new();
        let mod_def = self.get_current_module()?;
        self.build_fn_map(mod_def, &mut fn_map)?;
        let mut assembler = Assembler::new().map_err(|_| Error::Unknown)?;
        std::mem::swap(&mut self.assembler, &mut assembler);
        let buffer = assembler.finalize().unwrap();
        Ok(Output::new(fn_map, buffer))
    }
}
