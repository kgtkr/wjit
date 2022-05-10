use std::collections::HashMap;

use crate::ir;
use parity_wasm::elements::{BlockType, Instruction, ValueType};

#[derive(Debug, PartialEq, Clone, Hash, Eq)]

pub enum BuiltinFunc {
    Println,
}

#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub enum FuncRef {
    Direct(u32),
    Indirect(u32),
}

pub struct InstrsGenerator {
    // args_count -> type_id
    pub types: HashMap<usize, u32>,
    pub func_refs: HashMap<usize, FuncRef>,
    pub builtin_func_refs: HashMap<BuiltinFunc, FuncRef>,
}

struct InstrsGeneratorState {
    instrs: Vec<Instruction>,
}

impl InstrsGeneratorState {
    fn new() -> Self {
        InstrsGeneratorState { instrs: Vec::new() }
    }
}

impl InstrsGenerator {
    pub fn new() -> Self {
        InstrsGenerator {
            types: HashMap::new(),
            func_refs: HashMap::new(),
            builtin_func_refs: HashMap::new(),
        }
    }

    pub fn gen_instrs(&self, instrs: &Vec<ir::Instr>) -> Vec<Instruction> {
        let mut state = InstrsGeneratorState::new();
        for instr in instrs {
            self.gen_instr(&mut state, instr);
        }
        state.instrs.push(Instruction::End);
        state.instrs
    }

    fn gen_instr(&self, state: &mut InstrsGeneratorState, instr: &ir::Instr) {
        match instr {
            ir::Instr::NonControl(non_control) => match non_control {
                ir::NonControlInstr::IntConst(x) => {
                    state.instrs.push(Instruction::I32Const(*x));
                }
                ir::NonControlInstr::VarRef(idx) => {
                    state.instrs.push(Instruction::GetLocal(*idx as u32));
                }
                ir::NonControlInstr::Add => state.instrs.push(Instruction::I32Add),
                ir::NonControlInstr::Sub => state.instrs.push(Instruction::I32Sub),
                ir::NonControlInstr::Mul => state.instrs.push(Instruction::I32Mul),
                ir::NonControlInstr::Div => state.instrs.push(Instruction::I32DivS),
                ir::NonControlInstr::Mod => state.instrs.push(Instruction::I32RemS),
                ir::NonControlInstr::Lt => state.instrs.push(Instruction::I32LtS),
                ir::NonControlInstr::Gt => state.instrs.push(Instruction::I32GtS),
                ir::NonControlInstr::Le => state.instrs.push(Instruction::I32LeS),
                ir::NonControlInstr::Ge => state.instrs.push(Instruction::I32GeS),
                ir::NonControlInstr::Eq => state.instrs.push(Instruction::I32Eq),
                ir::NonControlInstr::Ne => state.instrs.push(Instruction::I32Ne),
                ir::NonControlInstr::And => state.instrs.push(Instruction::I32And),
                ir::NonControlInstr::Or => state.instrs.push(Instruction::I32Or),
                ir::NonControlInstr::Not => state.instrs.push(Instruction::I32Eqz),
                ir::NonControlInstr::Minus => {
                    state.instrs.push(Instruction::I32Const(0));
                    state.instrs.push(Instruction::I32Sub);
                }
                ir::NonControlInstr::Assign(idx) => {
                    state.instrs.push(Instruction::SetLocal(*idx as u32));
                }
                ir::NonControlInstr::Println => {
                    let func_ref = &self.builtin_func_refs[&BuiltinFunc::Println];
                    self.gen_func_refs(state, func_ref, 1);
                }
                ir::NonControlInstr::Drop => {
                    state.instrs.push(Instruction::Drop);
                }
            },
            ir::Instr::Call { func, args_count } => {
                let func_ref = &self.func_refs[func];
                self.gen_func_refs(state, func_ref, *args_count);
            }
            ir::Instr::Loop(_) => {
                state.instrs.push(Instruction::Block(BlockType::NoResult));
                state.instrs.push(Instruction::Loop(BlockType::NoResult));
            }
            ir::Instr::LoopThen(_) => {
                state.instrs.push(Instruction::I32Eqz);
                state.instrs.push(Instruction::BrIf(1));
            }
            ir::Instr::LoopEnd(_) => {
                state.instrs.push(Instruction::Br(0));
                state.instrs.push(Instruction::End);
                state.instrs.push(Instruction::End);
            }
            ir::Instr::If(_) => {
                state
                    .instrs
                    .push(Instruction::If(BlockType::Value(ValueType::I32)));
            }
            ir::Instr::Else(_) => {
                state.instrs.push(Instruction::Else);
            }
            ir::Instr::IfEnd(_) => {
                state.instrs.push(Instruction::End);
            }
            ir::Instr::Return => {
                state.instrs.push(Instruction::Return);
            }
        }
    }

    fn gen_func_refs(
        &self,
        state: &mut InstrsGeneratorState,
        func_ref: &FuncRef,
        args_count: usize,
    ) {
        match func_ref {
            FuncRef::Direct(idx) => {
                state.instrs.push(Instruction::Call(*idx));
            }
            FuncRef::Indirect(idx) => {
                state.instrs.push(Instruction::I32Const(*idx as i32));
                state
                    .instrs
                    .push(Instruction::CallIndirect(self.types[&args_count], 0));
            }
        }
    }
}
