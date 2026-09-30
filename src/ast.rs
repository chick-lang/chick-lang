pub type Program = Vec<Stmt>;
type Identifier = String;

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOpKind {
    Xor,
    LogAnd,
    LogOr,

    Plus,
    Minus,
    Mult,
    Div,
    Mod,
    Power,
    BitAnd,
    BitOr,

    Assign,
    AndAssign,
    OrAssign,
    XorAssign,
    PlusAssign,
    MinusAssign,
    MultAssign,
    DivAssign,
    ModAssign,
    PowerAssign,

    Equal,
    NotEqual,
    Greater,
    Less,
    GreaterEqual,
    LessEqual,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOpKind {
    Not,
    Increment,
    Decrement,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Break,
    Continue,
    Expr(Expr),
    For {
        var: Identifier,
        iter: Expr,
        body: Box<Stmt>,
    },
    If {
        cond: Expr,
        body: Box<Stmt>,
    },
    Let {
        name: Identifier,
        initiator: Expr,
    },
    Return(Expr),
    While {
        cond: Expr,
        body: Box<Stmt>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    BinaryOp {
        left: Box<Expr>,
        op: BinaryOpKind,
        right: Box<Expr>,
    },
    Call {
        calee: Identifier,
        args: Vec<Expr>,
    },
    Class {
        name: Option<Identifier>,
        body: Vec<Stmt>,
    },
    Float(String),
    Function {
        name: Option<Identifier>,
        args: Vec<Identifier>,
        body: Program,
    },
    Int(String),
    Range {
        start: Box<Expr>,
        finish: Box<Expr>,
    },
    UnaryOp {
        target: Box<Expr>,
        op: UnaryOpKind,
    },
    Variable(Identifier),
}
