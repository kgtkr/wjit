use crate::ir;
use crate::wasm_generator;
use parity_wasm::elements::{
    CodeSection, ElementSection, ElementSegment, ExportEntry, ExportSection, External, Func,
    FuncBody, FunctionSection, FunctionType, ImportEntry, ImportSection, InitExpr, Instruction,
    Instructions, Internal, Local, Module, Section, TableSection, TableType, Type, TypeSection,
    ValueType,
};

#[derive(Debug, PartialEq, Clone, Hash, Eq)]

pub struct Compiler<'a> {
    module: &'a ir::Module,
    // typeはめんどくさいのでパラメータが0〜5のものをそれぞれindex 0〜5で
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
        let func = &self.module.funcs[idx];

        let mut generator = wasm_generator::InstrsGenerator::new();

        generator.types = (0..5).map(|x| (x, x as u32)).collect();
        generator.func_refs = (0..self.module.funcs.len())
            .map(|x| (x, wasm_generator::FuncRef::Indirect(x as u32)))
            .collect();
        generator.builtin_func_refs = vec![(
            wasm_generator::BuiltinFunc::Println,
            wasm_generator::FuncRef::Direct(0),
        )]
        .into_iter()
        .collect();

        let instrs = generator.gen_instrs(&func.instrs);

        FuncBody::new(
            vec![Local::new(func.locals_count as u32, ValueType::I32)],
            Instructions::new(instrs),
        )
    }
}
