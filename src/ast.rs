#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub start_line: usize,
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
}

#[derive(Debug, Clone)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}
#[derive(Debug, Clone)]
pub enum Expr {
    String(String),
    Number(i64),
    Bool(bool),
    Array(Vec<Expr>),
    Nil,

    Assign {
        name: String,
        indices: Vec<Expr>,
        value: Box<Expr>,
    },
    
    Declare {
        name: String,
        value: Box<Expr>
    },

    Lambda {
        args: Vec<String>,
        body: Box<Expr>,
    },

    Ident(String),

    Block(Vec<Expr>),

    Return(Option<Box<Expr>>),

    If {
        cond: Box<Expr>,
        then: Box<Expr>,
        else_: Option<Box<Expr>>,
    },

    For {
        var: String,
        iter: Box<Expr>,
        body: Box<Expr>,
    },

    While {
        cond: Box<Expr>,
        body: Box<Expr>,
    },

    Break,
    Continue,

    Mod {
        modulus: Box<Expr>,
        body: Box<Expr>,
    },

    Range {
        start: Box<Expr>,
        end: Box<Expr>,
        inclusive: bool,
    },

    Call {
        func: Box<Expr>,
        args: Vec<Expr>,
    },

    BinOp {
        left: Box<Expr>,
        right: Box<Expr>,
        op: BinOp,
    },

    UnaryOp {
        op: UnaryOp,
        expr: Box<Expr>,
    },

    Index {
        object: Box<Expr>,
        index: Box<Expr>,
    },
    
    Len(Box<Expr>),
    Input,
    Print(Box<Expr>),
    Push(Box<Expr>, Box<Expr>),
    
    LogicalOp {
        left: Box<Expr>,
        right: Box<Expr>,
        op: LogicalOp
    }
}

#[derive(Clone)]
#[derive(Debug)]
pub enum LogicalOp {
    And,
    Or
}

#[derive(Debug, Clone)]
pub enum BinOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Rem,

    // Comp
    Eq,
    Neq,
    Gt,
    Ge,
    Lt,
    Le,

    // Bit
    And,
    Or,
    Xor,
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Neg,
    Not,
}
