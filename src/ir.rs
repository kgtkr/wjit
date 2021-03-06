#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub struct Module {
    pub funcs: Vec<Func>,
}

#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub struct IfInfo {
    pub if_: usize,
    pub else_: usize,
    pub if_end: usize,
}

impl IfInfo {
    pub fn dummy() -> Self {
        IfInfo {
            if_: 0,
            else_: 0,
            if_end: 0,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub struct LoopInfo {
    pub loop_: usize,
    pub loop_then: usize,
    pub loop_end: usize,
}

impl LoopInfo {
    pub fn dummy() -> Self {
        LoopInfo {
            loop_: 0,
            loop_then: 0,
            loop_end: 0,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub struct Func {
    pub args_count: usize,
    pub locals_count: usize,
    pub instrs: Vec<Instr>,
    pub if_infos: Vec<IfInfo>,
    pub loop_infos: Vec<LoopInfo>,
    pub name: String,
}

pub type LoopId = usize;
pub type IfId = usize;

#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub enum Instr {
    Call { func: usize, args_count: usize },
    If(IfId),
    Else(IfId),
    IfEnd(IfId),
    Loop(LoopId),
    LoopThen(LoopId),
    LoopEnd(LoopId),
    Return,
    NonControl(NonControlInstr),
}

#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub enum NonControlInstr {
    IntConst(i32),
    VarRef(usize),
    Assign(usize),
    Println,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Lt,
    Gt,
    Le,
    Ge,
    Eq,
    Ne,
    And,
    Or,
    Not,
    Minus,
    Drop,
}
