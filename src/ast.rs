pub type Program = Vec<Stmt>;
type Identifier = String;
type IdentifierTyped = (Identifier, Option<Identifier>);

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

    Equal,
    NotEqual,
    Greater,
    Less,
    GreaterEqual,
    LessEqual,

    Range,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOpKind {
    Not,
    Plus,
    Minus,
    PreIncrement,
    PreDecrement,
    AfterIncrement,
    AfterDecrement,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Assign {
        target: Identifier,
        op: Option<BinaryOpKind>,
        value: Expr,
    },
    Break,
    Continue,
    Expr(Expr),
    For {
        var: Identifier,
        iter: Expr,
        body: Program,
    },
    If {
        cond: Expr,
        body: Program,
        else_branch: Option<Program>,
    },
    Let {
        variable: IdentifierTyped,
        initiator: Option<Expr>,
    },
    Return(Expr),
    While {
        cond: Expr,
        body: Program,
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
    Char(char),
    False,
    Float(String),
    Function {
        name: Option<Identifier>,
        args: Vec<IdentifierTyped>,
        body: Program,
    },
    Int(String),
    String(String),
    True,
    UnaryOp {
        target: Box<Expr>,
        op: UnaryOpKind,
    },
    Variable(Identifier),
}
