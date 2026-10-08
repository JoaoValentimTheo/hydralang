use hydra_resolve::{FunctionId, SymbolId};
use hydra_source::Span;
use hydra_stdlib::BuiltinId;
use hydra_types::Type;

#[derive(Clone, Debug)]
pub struct HirProgram {
    pub functions: Vec<HirFunction>,
}

#[derive(Clone, Debug)]
pub struct HirFunction {
    pub id: FunctionId,
    pub name: String,
    pub params: Vec<HirParam>,
    pub return_type: Type,
    pub body: HirBlock,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct HirParam {
    pub symbol: SymbolId,
    pub name: String,
    pub ty: Type,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct HirBlock {
    pub statements: Vec<HirStmt>,
    pub tail: Option<Box<HirExpr>>,
    pub ty: Type,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum HirStmt {
    Let {
        symbol: SymbolId,
        mutable: bool,
        init: HirExpr,
        span: Span,
    },
    Expr(HirExpr),
    While {
        condition: HirExpr,
        body: HirBlock,
        span: Span,
    },
    Break {
        span: Span,
    },
    Continue {
        span: Span,
    },
    Return {
        value: Option<HirExpr>,
        span: Span,
    },
}

#[derive(Clone, Debug)]
pub struct HirExpr {
    pub kind: HirExprKind,
    pub ty: Type,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum HirExprKind {
    Literal(HirLiteral),
    Local(SymbolId),
    Unary {
        op: HirUnaryOp,
        operand: Box<HirExpr>,
    },
    Binary {
        left: Box<HirExpr>,
        op: HirBinaryOp,
        right: Box<HirExpr>,
    },
    Assign {
        symbol: SymbolId,
        value: Box<HirExpr>,
    },
    Call {
        callee: HirCallee,
        args: Vec<HirExpr>,
    },
    If {
        condition: Box<HirExpr>,
        then_branch: HirBlock,
        else_branch: Option<Box<HirExpr>>,
    },
    Block(HirBlock),
}

#[derive(Clone, Debug)]
pub enum HirCallee {
    Function(FunctionId),
    Builtin(BuiltinId),
}

#[derive(Clone, Debug, PartialEq)]
pub enum HirLiteral {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Unit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HirUnaryOp {
    Negate,
    Not,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HirBinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Or,
}
