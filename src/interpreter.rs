use crate::ir::*;

pub trait Builtin {
    fn println(&mut self, x: i32);
}

extern "C" {
    fn println(x: i32);
}

#[derive(Debug, PartialEq, Clone, Eq)]

pub struct WasmBuiltin;

impl Builtin for WasmBuiltin {
    fn println(&mut self, x: i32) {
        unsafe {
            println(x);
        }
    }
}

#[derive(Debug, PartialEq, Clone, Eq)]

pub struct RustBuiltin;

impl Builtin for RustBuiltin {
    fn println(&mut self, x: i32) {
        println!("{}", x);
    }
}

#[derive(Debug, PartialEq, Clone, Eq)]
pub struct PC {
    pub func: usize,
    pub instr: usize,
}

#[derive(Debug, PartialEq, Clone, Eq)]
pub struct StackFrame {
    pub pc: PC,
    pub base: usize,
}

#[derive(Debug, PartialEq, Clone, Eq)]
pub struct Interpreter<'a, B: Builtin> {
    pub pc: PC,
    pub stack: Vec<i32>,
    pub call_stack: Vec<StackFrame>,
    pub module: &'a Module,
    pub builtin: B,
}

impl<'a, B: Builtin> Interpreter<'a, B> {
    pub fn new(module: &'a Module, builtin: B) -> Self {
        Interpreter {
            pc: PC { func: 0, instr: 0 },
            stack: Vec::new(),
            call_stack: Vec::new(),
            module,
            builtin,
        }
    }

    pub fn step(&mut self) {
        let func = &self.module.funcs[self.pc.func];
        let instr = &func.instrs[self.pc.instr];
        let stack_frame = self.call_stack.last().unwrap();

        match instr {
            Instr::NonControl(non_control) => {
                match non_control {
                    NonControlInstr::IntConst(x) => {
                        self.stack.push(*x);
                    }
                    NonControlInstr::VarRef(idx) => {
                        self.stack.push(self.stack[stack_frame.base + *idx]);
                    }
                    NonControlInstr::Assign(idx) => {
                        let x = self.stack.pop().unwrap();
                        self.stack[stack_frame.base + *idx] = x;
                    }
                    NonControlInstr::Println => {
                        let x = self.stack.pop().unwrap();
                        self.builtin.println(x);
                        self.stack.push(0);
                    }
                    NonControlInstr::Add => {
                        let y = self.stack.pop().unwrap();
                        let x = self.stack.pop().unwrap();
                        self.stack.push(x + y);
                    }
                    NonControlInstr::Sub => {
                        let y = self.stack.pop().unwrap();
                        let x = self.stack.pop().unwrap();
                        self.stack.push(x - y);
                    }
                    NonControlInstr::Mul => {
                        let y = self.stack.pop().unwrap();
                        let x = self.stack.pop().unwrap();
                        self.stack.push(x * y);
                    }
                    NonControlInstr::Div => {
                        let y = self.stack.pop().unwrap();
                        let x = self.stack.pop().unwrap();
                        self.stack.push(x / y);
                    }
                    NonControlInstr::Mod => {
                        let y = self.stack.pop().unwrap();
                        let x = self.stack.pop().unwrap();
                        self.stack.push(x % y);
                    }
                    NonControlInstr::Lt => {
                        let y = self.stack.pop().unwrap();
                        let x = self.stack.pop().unwrap();
                        self.stack.push(if x < y { 1 } else { 0 });
                    }
                    NonControlInstr::Gt => {
                        let y = self.stack.pop().unwrap();
                        let x = self.stack.pop().unwrap();
                        self.stack.push(if x > y { 1 } else { 0 });
                    }
                    NonControlInstr::Le => {
                        let y = self.stack.pop().unwrap();
                        let x = self.stack.pop().unwrap();
                        self.stack.push(if x <= y { 1 } else { 0 });
                    }
                    NonControlInstr::Ge => {
                        let y = self.stack.pop().unwrap();
                        let x = self.stack.pop().unwrap();
                        self.stack.push(if x >= y { 1 } else { 0 });
                    }
                    NonControlInstr::Eq => {
                        let y = self.stack.pop().unwrap();
                        let x = self.stack.pop().unwrap();
                        self.stack.push(if x == y { 1 } else { 0 });
                    }
                    NonControlInstr::Ne => {
                        let y = self.stack.pop().unwrap();
                        let x = self.stack.pop().unwrap();
                        self.stack.push(if x != y { 1 } else { 0 });
                    }
                    NonControlInstr::And => {
                        let y = self.stack.pop().unwrap();
                        let x = self.stack.pop().unwrap();
                        self.stack.push(if x != 0 && y != 0 { 1 } else { 0 });
                    }
                    NonControlInstr::Or => {
                        let y = self.stack.pop().unwrap();
                        let x = self.stack.pop().unwrap();
                        self.stack.push(if x != 0 || y != 0 { 1 } else { 0 });
                    }
                    NonControlInstr::Not => {
                        let x = self.stack.pop().unwrap();
                        self.stack.push(if x == 0 { 1 } else { 0 });
                    }
                    NonControlInstr::Minus => {
                        let x = self.stack.pop().unwrap();
                        self.stack.push(-x);
                    }
                    NonControlInstr::Drop => {
                        self.stack.pop();
                    }
                }
                self.pc.instr += 1;
            }
            Instr::Call { func, args_count } => {
                self.call_stack.push(StackFrame {
                    pc: {
                        let mut pc = self.pc.clone();
                        pc.instr += 1;
                        pc
                    },
                    base: self.stack.len() - *args_count,
                });
                self.stack
                    .extend((0..self.module.funcs[*func].locals_count).map(|_| 0));
                self.pc = PC {
                    func: *func,
                    instr: 0,
                };
            }
            &Instr::If(if_id) => {
                let x = self.stack.pop().unwrap();
                if x != 0 {
                    self.pc.instr += 1;
                } else {
                    self.pc.instr = func.if_infos[if_id].else_ + 1;
                }
            }
            &Instr::Else(if_id) => {
                self.pc.instr = func.if_infos[if_id].if_end + 1;
            }
            &Instr::IfEnd(_) => {
                self.pc.instr += 1;
            }
            &Instr::Loop(_) => {
                self.pc.instr += 1;
            }
            &Instr::LoopThen(loop_id) => {
                let x = self.stack.pop().unwrap();
                if x != 0 {
                    self.pc.instr += 1;
                } else {
                    self.pc.instr = func.loop_infos[loop_id].loop_end + 1;
                }
            }
            &Instr::LoopEnd(loop_id) => {
                self.pc.instr = func.loop_infos[loop_id].loop_;
            }
            Instr::Return => {
                let ret_val = self.stack.pop().unwrap();
                self.pc = stack_frame.pc.clone();
                self.stack.truncate(stack_frame.base);
                self.call_stack.pop();
                self.stack.push(ret_val);
            }
        }
    }

    pub fn dummy_func(&self) -> usize {
        self.module.funcs.len()
    }

    pub fn call_prepare(&mut self, func: usize, args: &[i32]) {
        self.call_stack.push(StackFrame {
            pc: PC {
                func: self.dummy_func(),
                instr: 0,
            },
            base: 0,
        });
        self.pc = PC { func, instr: 0 };
        self.stack.extend(args.iter().cloned());
        self.stack
            .extend((0..self.module.funcs[func].locals_count).map(|_| 0));
    }

    pub fn call_result(&mut self) -> Option<i32> {
        if self.pc.func == self.dummy_func() {
            Some(self.stack.pop().unwrap())
        } else {
            None
        }
    }

    pub fn call(&mut self, func: usize, args: &[i32]) -> i32 {
        self.call_prepare(func, args);
        loop {
            self.step();
            if let Some(ret_val) = self.call_result() {
                return ret_val;
            }
        }
    }
}
