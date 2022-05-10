use crate::interpreter;
use crate::ir::*;

#[derive(Debug, PartialEq, Clone, Eq)]
pub enum LoopState {
    Profiling { count: usize },
}

#[derive(Debug, PartialEq, Clone, Eq)]
pub struct GeneratedMeta {
    pub guards: Vec<Guard>,
}

#[derive(Debug, PartialEq, Clone, Eq)]
pub struct Guard {
    pub pc: interpreter::PC,
    // stackのサイズ、call_stackのサイズ
}

#[derive(Debug, PartialEq, Clone, Eq)]
pub struct Vm<'a> {
    module: &'a Module,
    interpreter: interpreter::Interpreter<'a, interpreter::WasmBuiltin>,
    loop_states: Vec<Vec<LoopState>>,
}

impl<'a> Vm<'a> {
    pub fn new(module: &'a Module) -> Self {
        let interpreter = interpreter::Interpreter::new(module, interpreter::WasmBuiltin);
        let loop_states = module
            .funcs
            .iter()
            .map(|f| std::vec::from_elem(LoopState::Profiling { count: 0 }, f.loop_infos.len()))
            .collect();

        Vm {
            module,
            interpreter,
            loop_states,
        }
    }

    pub fn step(&mut self) {
        match &self.module.funcs[self.interpreter.pc.func].instrs[self.interpreter.pc.instr] {
            &Instr::Loop(idx) => {
                let loop_state = &mut self.loop_states[self.interpreter.pc.func][idx];
                match loop_state {
                    LoopState::Profiling { count } => {
                        *count += 1;
                    }
                }
            }
            _ => {}
        }
        self.interpreter.step();
    }
}
