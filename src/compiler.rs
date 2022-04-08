use crate::ir;
use parity_wasm::elements::{
    BlockType, CodeSection, ElementSection, ElementSegment, ExportEntry, ExportSection, External,
    Func, FuncBody, FunctionSection, FunctionType, ImportEntry, ImportSection, InitExpr,
    Instruction, Instructions, Internal, Local, Module, Section, TableSection, TableType, Type,
    TypeSection, ValueType,
};

#[derive(Debug, PartialEq, Clone)]

pub struct Compiler<'a> {
    module: &'a ir::Module,
    // typeはめんどくさいのでパラメータが0〜5のものをそれぞれindex 0〜5で
}

#[derive(Debug, PartialEq, Clone)]

pub enum BuiltinFunc {
    Println,
}

impl<'a> Compiler<'a> {
    pub fn new(module: &'a ir::Module) -> Self {
        Compiler { module }
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
                    .map(|func| Func::new(func.args_count as u32))
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

                instrs.extend((0..func.args_count).map(|i| Instruction::GetLocal(i as u32)));

                instrs.extend(vec![
                    Instruction::I32Const(idx as i32),
                    Instruction::Call(0), // compile
                    Instruction::Drop,
                    Instruction::I32Const(idx as i32),
                    Instruction::CallIndirect(func.args_count as u32, 0),
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
            Section::Function(FunctionSection::with_entries(vec![Func::new(
                func.args_count as u32,
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
        for instr in &func.instrs {
            self.compile_instr(&mut ctx, instr);
        }
        ctx.instrs.push(Instruction::End);

        FuncBody::new(
            vec![Local::new(func.locals_count as u32, ValueType::I32)],
            Instructions::new(ctx.instrs),
        )
    }

    fn compile_instr(&self, ctx: &mut CompileCtx, instr: &ir::Instr) {
        match instr {
            ir::Instr::IntConst(x) => {
                ctx.instrs.push(Instruction::I32Const(*x));
            }
            ir::Instr::VarRef(idx) => {
                ctx.instrs.push(Instruction::GetLocal(*idx as u32));
            }
            ir::Instr::Add => ctx.instrs.push(Instruction::I32Add),
            ir::Instr::Sub => ctx.instrs.push(Instruction::I32Sub),
            ir::Instr::Mul => ctx.instrs.push(Instruction::I32Mul),
            ir::Instr::Div => ctx.instrs.push(Instruction::I32DivS),
            ir::Instr::Mod => ctx.instrs.push(Instruction::I32RemS),
            ir::Instr::Lt => ctx.instrs.push(Instruction::I32LtS),
            ir::Instr::Gt => ctx.instrs.push(Instruction::I32GtS),
            ir::Instr::Le => ctx.instrs.push(Instruction::I32LeS),
            ir::Instr::Ge => ctx.instrs.push(Instruction::I32GeS),
            ir::Instr::Eq => ctx.instrs.push(Instruction::I32Eq),
            ir::Instr::Ne => ctx.instrs.push(Instruction::I32Ne),
            ir::Instr::And => ctx.instrs.push(Instruction::I32And),
            ir::Instr::Or => ctx.instrs.push(Instruction::I32Or),
            ir::Instr::Not => ctx.instrs.push(Instruction::I32Eqz),
            ir::Instr::Minus => {
                ctx.instrs.push(Instruction::I32Const(0));
                ctx.instrs.push(Instruction::I32Sub);
            }
            ir::Instr::Assign(idx) => {
                ctx.instrs.push(Instruction::SetLocal(*idx as u32));
            }
            ir::Instr::Call { func, args_count } => {
                ctx.instrs.push(Instruction::I32Const(*func as i32));
                ctx.instrs
                    .push(Instruction::CallIndirect(*args_count as u32, 0));
            }
            ir::Instr::Loop(_) => {
                ctx.instrs.push(Instruction::Block(BlockType::NoResult));
                ctx.instrs.push(Instruction::Loop(BlockType::NoResult));
            }
            ir::Instr::LoopThen(_) => {
                ctx.instrs.push(Instruction::I32Eqz);
                ctx.instrs.push(Instruction::BrIf(1));
            }
            ir::Instr::LoopEnd(_) => {
                ctx.instrs.push(Instruction::Br(0));
                ctx.instrs.push(Instruction::End);
                ctx.instrs.push(Instruction::End);
            }
            ir::Instr::If(_) => {
                ctx.instrs
                    .push(Instruction::If(BlockType::Value(ValueType::I32)));
            }
            ir::Instr::Else(_) => {
                ctx.instrs.push(Instruction::Else);
            }
            ir::Instr::IfEnd(_) => {
                ctx.instrs.push(Instruction::End);
            }
            ir::Instr::Println => {
                ctx.instrs.push(Instruction::Call(0));
            }
            ir::Instr::Return => {
                ctx.instrs.push(Instruction::Return);
            }
            ir::Instr::Drop => {
                ctx.instrs.push(Instruction::Drop);
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone)]

struct CompileCtx {
    instrs: Vec<Instruction>,
}

impl CompileCtx {
    fn new() -> Self {
        CompileCtx { instrs: Vec::new() }
    }
}
