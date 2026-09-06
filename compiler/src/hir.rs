use crate::ast::Span;

pub type LocalId = usize;
pub type FunctionId = usize;
pub type BlockId = usize;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
    Null,
    Never,
    Bool,
    Int {
        bits: u32,
        signed: bool,
    },
    Float {
        bits: u32,
    },
    String,
    Record {
        primary: Box<Type>,
        fields: Vec<(String, Type)>,
    },
}

#[derive(Clone, Debug)]
pub struct Program {
    pub body: Block,
    pub functions: Vec<Function>,
    pub locals: Vec<Type>,
}

#[derive(Clone, Debug)]
pub struct Function {
    pub id: FunctionId,
    pub name: String,
    pub params: Vec<LocalId>,
    pub result: Type,
    pub body: Block,
}

#[derive(Clone, Debug)]
pub struct Block {
    pub id: BlockId,
    pub ty: Type,
    pub stmts: Vec<Stmt>,
}

#[derive(Clone, Debug)]
pub enum Stmt {
    Bind {
        id: LocalId,
        value: Expr,
    },
    Assign {
        id: LocalId,
        value: Expr,
    },
    Emit {
        target: BlockId,
        field: Option<String>,
        value: Expr,
    },
    If {
        condition: Expr,
        then: Vec<Stmt>,
        otherwise: Vec<Stmt>,
    },
    Leave(BlockId),
    Restart(BlockId),
    Expr(Expr),
}

#[derive(Clone, Debug)]
pub struct Expr {
    pub kind: ExprKind,
    pub ty: Type,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum ExprKind {
    Null,
    Bool(bool),
    Int(i128),
    Float(f64),
    String(String),
    Local(LocalId),
    Unary {
        op: String,
        value: Box<Expr>,
    },
    Binary {
        op: String,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Call {
        id: FunctionId,
        args: Vec<Expr>,
    },
    Print {
        parts: Vec<Expr>,
        newline: bool,
    },
    Panic {
        parts: Vec<Expr>,
    },
    Block(Block),
    Field {
        value: Box<Expr>,
        index: usize,
    },
    Primary(Box<Expr>),
    StringSize(Box<Expr>),
}
