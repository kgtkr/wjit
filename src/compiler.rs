use std::collections::HashMap;

use crate::ast;
use elements::{
    BlockType, External, FuncBody, FunctionSection, FunctionType, ImportEntry, ImportSection,
    Instruction, Instructions, Local, Module, Section, TableSection, TableType, Type, TypeSection,
    ValueType,
};
use parity_wasm::elements::{
    self, CodeSection, ElementSection, ElementSegment, ExportEntry, ExportSection, InitExpr,
    Internal,
};

#[derive(Debug, PartialEq, Clone)]

pub struct Compiler {
    module: ast::Module,
    func_refs: HashMap<String, FuncRef>,
    // typeはめんどくさいのでパラメータが0〜5のものをそれぞれindex 0〜5で
}

#[derive(Debug, PartialEq, Clone)]

pub enum FuncRef {
    UserDefined { module_idx: usize, table_idx: u32 },
    Builtin { kind: BuiltinFunc, func_idx: u32 },
}

#[derive(Debug, PartialEq, Clone)]

pub enum BuiltinFunc {
    Println,
}

impl Compiler {
    pub fn new(module: ast::Module) -> Option<Self> {
        let mut func_refs = HashMap::new();

        func_refs.insert(
            "println".to_string(),
            FuncRef::Builtin {
                kind: BuiltinFunc::Println,
                func_idx: 0,
            },
        );

        for (i, func) in module.funcs.iter().enumerate() {
            match func_refs.insert(
                func.name.clone(),
                FuncRef::UserDefined {
                    module_idx: i,
                    table_idx: i as u32,
                },
            ) {
                None => (),
                _ => return None,
            }
        }

        Some(Compiler { module, func_refs })
    }

    fn type_section() -> TypeSection {
        TypeSection::with_types(
            (0..5)
                .map(|i| {
                    Type::Function(FunctionType::new(
                        {
                            let mut params = Vec::new();
                            params.resize_with(i, || ValueType::I32);
                            params
                        },
                        vec![ValueType::I32],
                    ))
                })
                .collect(),
        )
    }

    pub fn compile_skeleton(&self) -> Module {
        Module::new(vec![
            Section::Type(Self::type_section()),
            Section::Import(ImportSection::with_entries(vec![ImportEntry::new(
                "env".to_string(),
                "compile_func".to_string(),
                External::Function(1),
            )])),
            Section::Function(FunctionSection::with_entries(
                self.module
                    .funcs
                    .iter()
                    .map(|func| elements::Func::new(func.args.len() as u32))
                    .collect(),
            )),
            Section::Table(TableSection::with_entries(vec![TableType::new(
                self.module.funcs.len() as u32,
                None,
            )])),
            Section::Export(ExportSection::with_entries({
                let mut entries = Vec::new();
                for (i, func) in self.module.funcs.iter().enumerate() {
                    entries.push(ExportEntry::new(
                        func.name.clone(),
                        Internal::Function(i as u32 + 1), // TODO: importした関数を考慮にいれて+1しているが、もう少し綺麗にするべき
                    ));
                }
                entries.push(ExportEntry::new("_table".to_string(), Internal::Table(0)));
                entries
            })),
            Section::Element(ElementSection::with_entries(vec![ElementSegment::new(
                0,
                Some(InitExpr::new(vec![
                    Instruction::I32Const(0),
                    Instruction::End,
                ])),
                self.module
                    .funcs
                    .iter()
                    .enumerate()
                    .map(|(i, _)| (i as u32) + 1) // TODO: importした関数を考慮にいれて+1しているが、もう少し綺麗にするべき
                    .collect(),
            )])),
            Section::Code(CodeSection::with_bodies(
                self.module
                    .funcs
                    .iter()
                    .enumerate()
                    .map(|(i, _)| self.compile_skeleton_func(i))
                    .collect(),
            )),
        ])
    }

    pub fn compile_skeleton_func(&self, idx: usize) -> FuncBody {
        let func = &self.module.funcs[idx];
        FuncBody::new(
            vec![],
            Instructions::new({
                let mut instrs = Vec::new();

                instrs.extend((0..func.args.len()).map(|i| Instruction::GetLocal(i as u32)));

                instrs.extend(vec![
                    Instruction::I32Const(idx as i32),
                    Instruction::Call(0),
                    Instruction::Drop,
                    Instruction::I32Const(idx as i32),
                    Instruction::CallIndirect(func.args.len() as u32, 0),
                    Instruction::End,
                ]);
                instrs
            }),
        )
    }

    pub fn compile_func_module(&self, idx: usize) -> Module {
        let func = &self.module.funcs[idx];
        Module::new(vec![
            Section::Type(Self::type_section()),
            Section::Import(ImportSection::with_entries(vec![
                ImportEntry::new(
                    "env".to_string(),
                    "println".to_string(),
                    External::Function(1),
                ),
                ImportEntry::new(
                    "env".to_string(),
                    "_table".to_string(),
                    External::Table(TableType::new(self.module.funcs.len() as u32, None)),
                ),
            ])),
            Section::Function(FunctionSection::with_entries(vec![elements::Func::new(
                func.args.len() as u32,
            )])),
            Section::Element(ElementSection::with_entries(vec![ElementSegment::new(
                0,
                Some(InitExpr::new(vec![
                    Instruction::I32Const(idx as i32),
                    Instruction::End,
                ])),
                vec![1],
            )])),
            Section::Code(CodeSection::with_bodies(vec![self.compile_func(idx)])),
        ])
    }

    pub fn compile_func(&self, idx: usize) -> FuncBody {
        let mut ctx = CompileCtx::new();

        let func = &self.module.funcs[idx];

        for name in &func.args {
            ctx.add_local(name.clone());
        }

        self.compile_expr(&mut ctx, &func.body);
        ctx.instrs.push(Instruction::End);

        FuncBody::new(
            vec![Local::new(
                ctx.local_count as u32 - func.args.len() as u32,
                ValueType::I32,
            )],
            Instructions::new(ctx.instrs),
        )
    }

    fn compile_expr(&self, ctx: &mut CompileCtx, expr: &ast::Expr) {
        match expr {
            ast::Expr::IntLiteral(x) => {
                ctx.instrs.push(Instruction::I32Const(*x));
            }
            ast::Expr::Ident(name) => {
                let local_idx = ctx.locals.get(name).cloned().unwrap();
                ctx.instrs.push(Instruction::GetLocal(local_idx));
            }
            ast::Expr::BinaryOp(expr1, op, expr2) => {
                self.compile_expr(ctx, expr1);
                self.compile_expr(ctx, expr2);
                match op {
                    ast::BinaryOp::Add => ctx.instrs.push(Instruction::I32Add),
                    ast::BinaryOp::Sub => ctx.instrs.push(Instruction::I32Sub),
                    ast::BinaryOp::Mul => ctx.instrs.push(Instruction::I32Mul),
                    ast::BinaryOp::Div => ctx.instrs.push(Instruction::I32DivS),
                    ast::BinaryOp::Mod => ctx.instrs.push(Instruction::I32RemS),
                    ast::BinaryOp::Lt => ctx.instrs.push(Instruction::I32LtS),
                    ast::BinaryOp::Gt => ctx.instrs.push(Instruction::I32GtS),
                    ast::BinaryOp::Le => ctx.instrs.push(Instruction::I32LeS),
                    ast::BinaryOp::Ge => ctx.instrs.push(Instruction::I32GeS),
                    ast::BinaryOp::Eq => ctx.instrs.push(Instruction::I32Eq),
                    ast::BinaryOp::Ne => ctx.instrs.push(Instruction::I32Ne),
                    ast::BinaryOp::And => ctx.instrs.push(Instruction::I32And),
                    ast::BinaryOp::Or => ctx.instrs.push(Instruction::I32Or),
                }
            }
            ast::Expr::PrefixOp(op, expr) => {
                self.compile_expr(ctx, expr);
                match op {
                    ast::PrefixOp::Not => ctx.instrs.push(Instruction::I32Eqz),
                    ast::PrefixOp::Minus => {
                        ctx.instrs.push(Instruction::I32Const(0));
                        ctx.instrs.push(Instruction::I32Sub);
                    }
                }
            }
            ast::Expr::Assign(ident, expr) => {
                self.compile_expr(ctx, expr);
                let local_idx = ctx.locals.get(ident).cloned().unwrap();
                ctx.instrs.push(Instruction::SetLocal(local_idx));
                ctx.instrs.push(Instruction::I32Const(0));
            }
            ast::Expr::Call(ident, exprs) => {
                for expr in exprs {
                    self.compile_expr(ctx, expr);
                }

                let func_ref = self.func_refs.get(ident).cloned().unwrap();
                match func_ref {
                    FuncRef::UserDefined { table_idx, .. } => {
                        ctx.instrs.push(Instruction::I32Const(table_idx as i32));
                        ctx.instrs
                            .push(Instruction::CallIndirect(exprs.len() as u32, 0));
                    }
                    FuncRef::Builtin { func_idx, .. } => {
                        ctx.instrs.push(Instruction::Call(func_idx));
                    }
                };
            }
            ast::Expr::While(cond, body) => {
                ctx.instrs.push(Instruction::Block(BlockType::NoResult));
                ctx.instrs.push(Instruction::Loop(BlockType::NoResult));
                self.compile_expr(ctx, cond);
                ctx.instrs.push(Instruction::I32Eqz);
                ctx.instrs.push(Instruction::BrIf(1));
                self.compile_expr(ctx, body);
                ctx.instrs.push(Instruction::Drop);
                ctx.instrs.push(Instruction::Br(0));
                ctx.instrs.push(Instruction::End);
                ctx.instrs.push(Instruction::End);
                ctx.instrs.push(Instruction::I32Const(0));
            }
            ast::Expr::If(cond, body, else_body) => {
                self.compile_expr(ctx, cond);
                ctx.instrs
                    .push(Instruction::If(BlockType::Value(ValueType::I32)));
                self.compile_expr(ctx, body);
                ctx.instrs.push(Instruction::Else);
                self.compile_expr(ctx, else_body);
                ctx.instrs.push(Instruction::End);
            }
            ast::Expr::Block(exprs) => match exprs.split_last() {
                Some((expr, last)) => {
                    for expr in last {
                        self.compile_expr(ctx, expr);
                        ctx.instrs.push(Instruction::Drop);
                    }
                    self.compile_expr(ctx, expr);
                }
                None => ctx.instrs.push(Instruction::I32Const(0)),
            },
            ast::Expr::Var(ident, expr1, expr2) => {
                self.compile_expr(ctx, expr1);
                let prev_locals = ctx.locals.clone();
                let local_idx = ctx.add_local(ident.clone());
                ctx.instrs.push(Instruction::SetLocal(local_idx));
                self.compile_expr(ctx, expr2);
                ctx.locals = prev_locals;
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone)]

struct CompileCtx {
    locals: HashMap<String, u32>,
    local_count: u32,
    instrs: Vec<Instruction>,
}

impl CompileCtx {
    fn new() -> Self {
        CompileCtx {
            locals: HashMap::new(),
            local_count: 0,
            instrs: Vec::new(),
        }
    }

    fn add_local(&mut self, name: String) -> u32 {
        let idx = self.local_count;
        self.local_count += 1;
        self.locals.insert(name, idx);
        idx
    }
}
