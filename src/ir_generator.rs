use std::collections::HashMap;

use crate::ast;
use crate::ir::*;

#[derive(Debug, PartialEq, Clone, Eq)]

struct IrGenerator<'a> {
    module: &'a ast::Module,
    func_refs: HashMap<String, FuncRef>,
}

#[derive(Debug, PartialEq, Clone, Hash, Eq)]

enum FuncRef {
    UserDefined { idx: usize },
    Builtin { kind: BuiltinFunc },
}

#[derive(Debug, PartialEq, Clone, Hash, Eq)]
enum BuiltinFunc {
    Println,
}

impl<'a> IrGenerator<'a> {
    fn new(module: &'a ast::Module) -> Option<Self> {
        let mut func_refs = HashMap::new();

        func_refs.insert(
            "println".to_string(),
            FuncRef::Builtin {
                kind: BuiltinFunc::Println,
            },
        );

        for (i, func) in module.funcs.iter().enumerate() {
            match func_refs.insert(func.name.clone(), FuncRef::UserDefined { idx: i }) {
                None => (),
                _ => return None,
            }
        }

        Some(IrGenerator { module, func_refs })
    }

    fn generate(&self) -> Module {
        let mut funcs = Vec::new();
        for i in 0..self.module.funcs.len() {
            funcs.push(self.gen_func(i));
        }

        Module { funcs }
    }
    fn gen_func(&self, idx: usize) -> Func {
        let mut state = GenFuncState::new();

        let func = &self.module.funcs[idx];

        for name in &func.args {
            state.add_local(name.clone());
        }

        self.gen_expr(&mut state, &func.body);
        state.instrs.push(Instr::Return);

        Func {
            args_count: func.args.len(),
            locals_count: state.locals_count,
            instrs: state.instrs,
            if_infos: state.if_infos,
            loop_infos: state.loop_infos,
            name: func.name.clone(),
        }
    }

    fn gen_expr(&self, state: &mut GenFuncState, expr: &ast::Expr) {
        match expr {
            ast::Expr::IntLiteral(x) => {
                state.instrs.push(Instr::IntConst(*x));
            }
            ast::Expr::Ident(name) => {
                let local_idx = state.locals.get(name).cloned().unwrap();
                state.instrs.push(Instr::VarRef(local_idx));
            }
            ast::Expr::BinaryOp(expr1, op, expr2) => {
                self.gen_expr(state, expr1);
                self.gen_expr(state, expr2);
                match op {
                    ast::BinaryOp::Add => state.instrs.push(Instr::Add),
                    ast::BinaryOp::Sub => state.instrs.push(Instr::Sub),
                    ast::BinaryOp::Mul => state.instrs.push(Instr::Mul),
                    ast::BinaryOp::Div => state.instrs.push(Instr::Div),
                    ast::BinaryOp::Mod => state.instrs.push(Instr::Mod),
                    ast::BinaryOp::Lt => state.instrs.push(Instr::Lt),
                    ast::BinaryOp::Gt => state.instrs.push(Instr::Gt),
                    ast::BinaryOp::Le => state.instrs.push(Instr::Le),
                    ast::BinaryOp::Ge => state.instrs.push(Instr::Ge),
                    ast::BinaryOp::Eq => state.instrs.push(Instr::Eq),
                    ast::BinaryOp::Ne => state.instrs.push(Instr::Ne),
                    ast::BinaryOp::And => state.instrs.push(Instr::And),
                    ast::BinaryOp::Or => state.instrs.push(Instr::Or),
                }
            }
            ast::Expr::PrefixOp(op, expr) => {
                self.gen_expr(state, expr);
                match op {
                    ast::PrefixOp::Not => state.instrs.push(Instr::Not),
                    ast::PrefixOp::Minus => {
                        state.instrs.push(Instr::Minus);
                    }
                }
            }
            ast::Expr::Assign(ident, expr) => {
                self.gen_expr(state, expr);
                let local_idx = state.locals.get(ident).cloned().unwrap();
                state.instrs.push(Instr::Assign(local_idx));
                state.instrs.push(Instr::IntConst(0));
            }
            ast::Expr::Call(ident, exprs) => {
                for expr in exprs {
                    self.gen_expr(state, expr);
                }

                let func_ref = self.func_refs.get(ident).cloned().unwrap();
                match func_ref {
                    FuncRef::UserDefined { idx, .. } => {
                        state.instrs.push(Instr::Call {
                            func: idx,
                            args_count: exprs.len(),
                        });
                    }
                    FuncRef::Builtin { kind } => match kind {
                        BuiltinFunc::Println => {
                            state.instrs.push(Instr::Println);
                        }
                    },
                };
            }
            ast::Expr::While(cond, body) => {
                let loop_id = state.loop_infos.len();
                let mut loop_info = LoopInfo::dummy();
                state.loop_infos.push(loop_info.clone());
                loop_info.loop_ = state.instrs.len();
                state.instrs.push(Instr::Loop(loop_id));
                self.gen_expr(state, cond);
                loop_info.loop_then = state.instrs.len();
                state.instrs.push(Instr::LoopThen(loop_id));
                self.gen_expr(state, body);
                state.instrs.push(Instr::Drop);
                loop_info.loop_end = state.instrs.len();
                state.instrs.push(Instr::LoopEnd(loop_id));
                state.loop_infos[loop_id] = loop_info;

                state.instrs.push(Instr::IntConst(0));
            }
            ast::Expr::If(cond, body, else_body) => {
                let if_id = state.if_infos.len();
                let mut if_info = IfInfo::dummy();
                state.if_infos.push(if_info.clone());
                self.gen_expr(state, cond);
                if_info.if_ = state.instrs.len();
                state.instrs.push(Instr::If(if_id));
                self.gen_expr(state, body);
                if_info.else_ = state.instrs.len();
                state.instrs.push(Instr::Else(if_id));
                self.gen_expr(state, else_body);
                if_info.if_end = state.instrs.len();
                state.instrs.push(Instr::IfEnd(if_id));
                state.if_infos[if_id] = if_info;
            }
            ast::Expr::Block(exprs) => match exprs.split_last() {
                Some((expr, last)) => {
                    for expr in last {
                        self.gen_expr(state, expr);
                        state.instrs.push(Instr::Drop);
                    }
                    self.gen_expr(state, expr);
                }
                None => state.instrs.push(Instr::IntConst(0)),
            },
            ast::Expr::Var(ident, expr1, expr2) => {
                self.gen_expr(state, expr1);
                let prev_locals = state.locals.clone();
                let local_idx = state.add_local(ident.clone());
                state.instrs.push(Instr::Assign(local_idx));
                self.gen_expr(state, expr2);
                state.locals = prev_locals;
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone, Eq)]

struct GenFuncState {
    locals: HashMap<String, usize>,
    locals_count: usize,
    instrs: Vec<Instr>,
    if_infos: Vec<IfInfo>,
    loop_infos: Vec<LoopInfo>,
}

impl GenFuncState {
    fn new() -> Self {
        GenFuncState {
            locals: HashMap::new(),
            locals_count: 0,
            instrs: Vec::new(),
            if_infos: Vec::new(),
            loop_infos: Vec::new(),
        }
    }

    fn add_local(&mut self, name: String) -> usize {
        let idx = self.locals_count;
        self.locals_count += 1;
        self.locals.insert(name, idx);
        idx
    }
}

pub fn generate(module: &ast::Module) -> Module {
    let gen = IrGenerator::new(&module).unwrap();
    gen.generate()
}
