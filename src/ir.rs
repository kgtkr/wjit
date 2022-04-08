#[derive(Debug, PartialEq, Clone)]
pub struct Module {
    pub funcs: Vec<Func>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Func {
    pub args_count: usize,
    pub instrs: Vec<Instr>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Instr {
    IntLiteral(i32),
    VarRef(usize),
    BinaryOp(BinaryOp),
    PrefixOp(PrefixOp),
    Assign(usize),
    Call(usize),
    While(usize, usize),
    If(usize),   // Else
    Else(usize), // EndIf
    EndIf,
    Loop,
    LoopThen(usize), // EndLoop
    EndLoop(usize),  // Loop
    VarDef(usize, usize),
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
