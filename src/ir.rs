#[derive(Debug, PartialEq, Clone)]
pub struct Module {
    pub funcs: Vec<Func>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct IfInfo {
    pub if_: usize,
    pub else_: usize,
    pub if_end: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub struct LoopInfo {
    pub loop_: usize,
    pub loop_then: usize,
    pub loop_end: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Func {
    pub args_count: usize,
    pub instrs: Vec<Instr>,
    pub if_infos: Vec<IfInfo>,
    pub loop_infos: Vec<LoopInfo>,
}

pub type LoopId = usize;
pub type IfId = usize;

#[derive(Debug, PartialEq, Clone)]
pub enum Instr {
    IntLiteral(i32),
    VarRef(usize),
    BinaryOp(BinaryOp),
    PrefixOp(PrefixOp),
    Assign(usize),
    Call(usize),
    If(IfId),
    Else(IfId),
    EndIf(IfId),
    Loop(LoopId),
    LoopThen(LoopId),
    EndLoop(LoopId),
    VarDef,
}

#[derive(Debug, PartialEq, Clone)]
pub enum BinaryOp {
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
}

#[derive(Debug, PartialEq, Clone)]
pub enum PrefixOp {
    Not,
    Minus,
}
