use hydra_ast::{BinaryOp, Block, Expr, ExprKind, Literal, Program, Stmt, TypeExpr, UnaryOp};
use hydra_diagnostics::{Diagnostic, Phase};
use hydra_hir::{
    HirBinaryOp, HirBlock, HirCallee, HirExpr, HirExprKind, HirFunction, HirLiteral, HirParam,
    HirProgram, HirStmt, HirUnaryOp,
};
use hydra_resolve::{FunctionId, Resolution, ResolvedName, SymbolId};
use hydra_source::Span;
use hydra_stdlib::BuiltinParam;
use hydra_types::Type;
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct CheckResult {
    pub hir: Option<HirProgram>,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Clone, Debug)]
struct FunctionSignature {
    params: Vec<Type>,
    return_type: Type,
}

// A bounded control-flow summary. `Never` describes the absence of a normal
// *value*; this summary retains the reason control did not fall through.
#[derive(Clone, Copy, Debug, Default)]
struct Outcome {
    falls_through: bool,
    returns: bool,
    breaks: bool,
    continues: bool,
    may_diverge: bool,
}

impl Outcome {
    const NORMAL: Self = Self {
        falls_through: true,
        returns: false,
        breaks: false,
        continues: false,
        may_diverge: false,
    };

    // Both branches can execute, so either branch's effects are possible.
    fn either(self, other: Self) -> Self {
        Self {
            falls_through: self.falls_through || other.falls_through,
            returns: self.returns || other.returns,
            breaks: self.breaks || other.breaks,
            continues: self.continues || other.continues,
            may_diverge: self.may_diverge || other.may_diverge,
        }
    }

    // The next operation is reached only on an ordinary path from `self`.
    fn then(self, next: Self) -> Self {
        Self {
            falls_through: self.falls_through && next.falls_through,
            returns: self.returns || (self.falls_through && next.returns),
            breaks: self.breaks || (self.falls_through && next.breaks),
            continues: self.continues || (self.falls_through && next.continues),
            may_diverge: self.may_diverge || (self.falls_through && next.may_diverge),
        }
    }
}

fn block_outcome(block: &HirBlock) -> Outcome {
    let mut result = Outcome::NORMAL;
    for statement in &block.statements {
        result = result.then(stmt_outcome(statement));
    }
    if let Some(tail) = &block.tail {
        result = result.then(expr_outcome(tail));
    }
    result
}

fn stmt_outcome(stmt: &HirStmt) -> Outcome {
    match stmt {
        HirStmt::Let { init, .. } | HirStmt::Expr(init) => expr_outcome(init),
        HirStmt::Break { .. } => Outcome {
            breaks: true,
            ..Outcome::default()
        },
        HirStmt::Continue { .. } => Outcome {
            continues: true,
            ..Outcome::default()
        },
        HirStmt::Return { value, .. } => {
            let operand = value.as_ref().map_or(Outcome::NORMAL, expr_outcome);
            operand.then(Outcome {
                returns: true,
                ..Outcome::default()
            })
        }
        HirStmt::While {
            condition, body, ..
        } => {
            // The condition runs at the *outer* loop depth; only body effects
            // are consumed by this while. A false condition may skip its body.
            let condition_effect = expr_outcome(condition);
            let body_effect = block_outcome(body);
            let loop_effect = Outcome {
                falls_through: true,
                returns: body_effect.returns,
                breaks: false,
                continues: false,
                may_diverge: body_effect.may_diverge,
            };
            condition_effect.then(loop_effect)
        }
    }
}

fn expr_outcome(expr: &HirExpr) -> Outcome {
    match &expr.kind {
        HirExprKind::Literal(_) | HirExprKind::Local(_) => Outcome::NORMAL,
        HirExprKind::Unary { operand, .. } => expr_outcome(operand),
        HirExprKind::Assign { value, .. } => expr_outcome(value),
        HirExprKind::Binary { left, op, right } => {
            let lhs = expr_outcome(left);
            let rhs = expr_outcome(right);
            if matches!(op, HirBinaryOp::And | HirBinaryOp::Or) {
                let is_skipped = matches!(
                    (&left.kind, op),
                    (
                        HirExprKind::Literal(HirLiteral::Bool(false)),
                        HirBinaryOp::And
                    ) | (
                        HirExprKind::Literal(HirLiteral::Bool(true)),
                        HirBinaryOp::Or
                    )
                );
                let is_always_evaluated = matches!(
                    (&left.kind, op),
                    (
                        HirExprKind::Literal(HirLiteral::Bool(true)),
                        HirBinaryOp::And
                    ) | (
                        HirExprKind::Literal(HirLiteral::Bool(false)),
                        HirBinaryOp::Or
                    )
                );
                if is_skipped {
                    lhs
                } else if is_always_evaluated {
                    lhs.then(rhs)
                } else {
                    lhs.then(Outcome::NORMAL.either(rhs))
                }
            } else {
                lhs.then(rhs)
            }
        }
        HirExprKind::Call { args, .. } => {
            let args_effect = args.iter().fold(Outcome::NORMAL, |outcome, arg| {
                outcome.then(expr_outcome(arg))
            });
            if expr.ty == Type::Never {
                args_effect.then(Outcome {
                    may_diverge: true,
                    ..Outcome::default()
                })
            } else {
                args_effect
            }
        }
        HirExprKind::If {
            condition,
            then_branch,
            else_branch,
        } => {
            let branches = block_outcome(then_branch).either(
                else_branch
                    .as_ref()
                    .map_or(Outcome::NORMAL, |branch| expr_outcome(branch)),
            );
            expr_outcome(condition).then(branches)
        }
        HirExprKind::Block(block) => block_outcome(block),
    }
}

#[must_use]
pub fn check(program: &Program, resolution: &Resolution) -> CheckResult {
    Checker::new(program, resolution).run()
}

struct Checker<'a> {
    program: &'a Program,
    resolution: &'a Resolution,
    signatures: BTreeMap<FunctionId, FunctionSignature>,
    local_types: BTreeMap<SymbolId, Type>,
    current_return: Type,
    loop_depth: usize,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> Checker<'a> {
    fn new(program: &'a Program, resolution: &'a Resolution) -> Self {
        Self {
            program,
            resolution,
            signatures: BTreeMap::new(),
            local_types: BTreeMap::new(),
            current_return: Type::Unit,
            loop_depth: 0,
            diagnostics: Vec::new(),
        }
    }

    fn run(mut self) -> CheckResult {
        self.collect_signatures();
        let mut functions = Vec::new();
        for function in &self.program.functions {
            let Some(id) = self.resolution.function(&function.name) else {
                continue;
            };
            let Some(signature) = self.signatures.get(&id).cloned() else {
                continue;
            };
            self.local_types.clear();
            self.current_return = signature.return_type.clone();
            self.loop_depth = 0;

            let mut params = Vec::new();
            for (param, ty) in function.params.iter().zip(signature.params.iter()) {
                if let Some(info) = self.resolution.declaration(param.name_span) {
                    self.local_types.insert(info.id, ty.clone());
                    params.push(HirParam {
                        symbol: info.id,
                        name: param.name.clone(),
                        ty: ty.clone(),
                        span: param.span,
                    });
                } else {
                    self.internal("missing resolved parameter declaration", param.name_span);
                }
            }

            let body = self.check_block(&function.body);
            if !compatible(&body.ty, &signature.return_type) {
                self.diagnostics.push(Diagnostic::error(
                    "E3002",
                    Phase::Type,
                    format!(
                        "function `{}` returns {}, but its body has type {}",
                        function.name, signature.return_type, body.ty
                    ),
                    function.body.span,
                ));
            }
            functions.push(HirFunction {
                id,
                name: function.name.clone(),
                params,
                return_type: signature.return_type,
                body,
                span: function.span,
            });
        }

        let hir = self
            .diagnostics
            .is_empty()
            .then_some(HirProgram { functions });
        CheckResult {
            hir,
            diagnostics: self.diagnostics,
        }
    }

    fn collect_signatures(&mut self) {
        for function in &self.program.functions {
            let Some(id) = self.resolution.function(&function.name) else {
                continue;
            };
            if self.signatures.contains_key(&id) {
                continue;
            }
            let params = function
                .params
                .iter()
                .map(|param| self.type_from_expr(&param.ty))
                .collect();
            let return_type = function
                .return_type
                .as_ref()
                .map_or(Type::Unit, |ty| self.type_from_expr(ty));
            self.signatures.insert(
                id,
                FunctionSignature {
                    params,
                    return_type,
                },
            );
        }
    }

    fn type_from_expr(&mut self, ty: &TypeExpr) -> Type {
        if let Some(ty) = Type::from_name(&ty.name) {
            ty
        } else {
            self.diagnostics.push(Diagnostic::error(
                "E3001",
                Phase::Type,
                format!("unknown type `{}`", ty.name),
                ty.span,
            ));
            Type::Unit
        }
    }

    fn check_block(&mut self, block: &Block) -> HirBlock {
        let mut statements = Vec::new();
        let mut outcome = Outcome::NORMAL;
        for stmt in &block.statements {
            let hir = self.check_stmt(stmt);
            outcome = outcome.then(stmt_outcome(&hir));
            statements.push(hir);
        }
        let tail = block
            .tail
            .as_ref()
            .map(|expr| Box::new(self.check_expr(expr)));
        let tail_outcome = tail
            .as_ref()
            .map_or(Outcome::NORMAL, |expr| expr_outcome(expr));
        let ty = if !outcome.then(tail_outcome).falls_through {
            Type::Never
        } else {
            tail.as_ref().map_or(Type::Unit, |expr| expr.ty.clone())
        };
        HirBlock {
            statements,
            tail,
            ty,
            span: block.span,
        }
    }

    fn check_stmt(&mut self, stmt: &Stmt) -> HirStmt {
        match stmt {
            Stmt::Let {
                mutable,
                name_span,
                ty,
                init,
                span,
                ..
            } => {
                let init = self.check_expr(init);
                let declared = ty.as_ref().map(|ty| self.type_from_expr(ty));
                if let Some(expected) = &declared {
                    self.require_type(&init.ty, expected, init.span, "binding initializer");
                }
                let binding_type = declared.unwrap_or_else(|| init.ty.clone());
                let symbol = self
                    .resolution
                    .declaration(*name_span)
                    .map(|info| info.id)
                    .unwrap_or_else(|| {
                        self.internal("missing resolved binding declaration", *name_span);
                        SymbolId(u32::MAX)
                    });
                self.local_types.insert(symbol, binding_type);
                HirStmt::Let {
                    symbol,
                    mutable: *mutable,
                    init,
                    span: *span,
                }
            }
            Stmt::Expr { expr, .. } => HirStmt::Expr(self.check_expr(expr)),
            Stmt::While {
                condition,
                body,
                span,
            } => {
                let condition = self.check_expr(condition);
                self.require_type(
                    &condition.ty,
                    &Type::Bool,
                    condition.span,
                    "while condition",
                );
                self.loop_depth += 1;
                let body = self.check_block(body);
                self.loop_depth -= 1;
                HirStmt::While {
                    condition,
                    body,
                    span: *span,
                }
            }
            Stmt::Break { span } => {
                if self.loop_depth == 0 {
                    self.type_error("E3010", "`break` outside of a while body", *span);
                }
                HirStmt::Break { span: *span }
            }
            Stmt::Continue { span } => {
                if self.loop_depth == 0 {
                    self.type_error("E3011", "`continue` outside of a while body", *span);
                }
                HirStmt::Continue { span: *span }
            }
            Stmt::Return { value, span } => {
                let value = value.as_ref().map(|expr| self.check_expr(expr));
                let actual = value.as_ref().map_or(Type::Unit, |expr| expr.ty.clone());
                let expected = self.current_return.clone();
                self.require_type(&actual, &expected, *span, "return value");
                HirStmt::Return { value, span: *span }
            }
        }
    }

    fn check_expr(&mut self, expr: &Expr) -> HirExpr {
        match &expr.kind {
            ExprKind::Literal(literal) => self.check_literal(literal, expr.span),
            ExprKind::Name(_) => self.check_name(expr),
            ExprKind::Unary { op, operand } => {
                let operand = self.check_expr(operand);
                let ty = if operand.ty == Type::Never {
                    Type::Never
                } else {
                    match op {
                        UnaryOp::Negate if matches!(operand.ty, Type::Int | Type::Float) => {
                            operand.ty.clone()
                        }
                        UnaryOp::Not if operand.ty == Type::Bool => Type::Bool,
                        UnaryOp::Negate => {
                            self.type_error(
                                "E3003",
                                format!("unary `-` requires Int or Float, found {}", operand.ty),
                                expr.span,
                            );
                            Type::Unit
                        }
                        UnaryOp::Not => {
                            self.type_error(
                                "E3003",
                                format!("unary `!` requires Bool, found {}", operand.ty),
                                expr.span,
                            );
                            Type::Unit
                        }
                    }
                };
                HirExpr {
                    kind: HirExprKind::Unary {
                        op: match op {
                            UnaryOp::Negate => HirUnaryOp::Negate,
                            UnaryOp::Not => HirUnaryOp::Not,
                        },
                        operand: Box::new(operand),
                    },
                    ty,
                    span: expr.span,
                }
            }
            ExprKind::Binary { left, op, right } => self.check_binary(expr.span, left, *op, right),
            ExprKind::Assign {
                name_span, value, ..
            } => {
                let value = self.check_expr(value);
                let symbol = match self.resolution.resolved(*name_span) {
                    Some(ResolvedName::Local(symbol)) => *symbol,
                    _ => {
                        self.internal("assignment was not resolved to a local", *name_span);
                        SymbolId(u32::MAX)
                    }
                };
                let expected = self.local_types.get(&symbol).cloned().unwrap_or_else(|| {
                    self.internal("missing local type for assignment", *name_span);
                    Type::Unit
                });
                self.require_type(&value.ty, &expected, value.span, "assignment value");
                let ty = if value.ty == Type::Never {
                    Type::Never
                } else {
                    Type::Unit
                };
                HirExpr {
                    kind: HirExprKind::Assign {
                        symbol,
                        value: Box::new(value),
                    },
                    ty,
                    span: expr.span,
                }
            }
            ExprKind::Call { callee, args } => self.check_call(expr.span, callee, args),
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let condition = self.check_expr(condition);
                self.require_type(&condition.ty, &Type::Bool, condition.span, "if condition");
                let then_branch = self.check_block(then_branch);
                let else_branch = else_branch
                    .as_ref()
                    .map(|branch| Box::new(self.check_expr(branch)));
                let ty = if condition.ty == Type::Never {
                    Type::Never
                } else if let Some(else_branch) = &else_branch {
                    Type::join(&then_branch.ty, &else_branch.ty).unwrap_or_else(|| {
                        self.type_error(
                            "E3004",
                            format!(
                                "if branches have incompatible types {} and {}",
                                then_branch.ty, else_branch.ty
                            ),
                            expr.span,
                        );
                        Type::Unit
                    })
                } else if matches!(then_branch.ty, Type::Unit | Type::Never) {
                    Type::Unit
                } else {
                    self.type_error(
                        "E3005",
                        "an `if` without `else` must not produce a value",
                        expr.span,
                    );
                    Type::Unit
                };
                HirExpr {
                    kind: HirExprKind::If {
                        condition: Box::new(condition),
                        then_branch,
                        else_branch,
                    },
                    ty,
                    span: expr.span,
                }
            }
            ExprKind::Block(block) => {
                let block = self.check_block(block);
                let ty = block.ty.clone();
                HirExpr {
                    kind: HirExprKind::Block(block),
                    ty,
                    span: expr.span,
                }
            }
        }
    }

    fn check_literal(&self, literal: &Literal, span: Span) -> HirExpr {
        let (literal, ty) = match literal {
            Literal::Int(value) => (HirLiteral::Int(*value), Type::Int),
            Literal::Float(value) => (HirLiteral::Float(*value), Type::Float),
            Literal::Bool(value) => (HirLiteral::Bool(*value), Type::Bool),
            Literal::String(value) => (HirLiteral::String(value.clone()), Type::String),
            Literal::Unit => (HirLiteral::Unit, Type::Unit),
        };
        HirExpr {
            kind: HirExprKind::Literal(literal),
            ty,
            span,
        }
    }

    fn check_name(&mut self, expr: &Expr) -> HirExpr {
        match self.resolution.resolved(expr.span) {
            Some(ResolvedName::Local(symbol)) => {
                let ty = self.local_types.get(symbol).cloned().unwrap_or_else(|| {
                    self.internal("missing type for resolved local", expr.span);
                    Type::Unit
                });
                HirExpr {
                    kind: HirExprKind::Local(*symbol),
                    ty,
                    span: expr.span,
                }
            }
            Some(ResolvedName::Function(_)) | Some(ResolvedName::Builtin(_)) => {
                self.type_error(
                    "E3006",
                    "functions are not first-class values in Hydra 0.1; call the name directly",
                    expr.span,
                );
                HirExpr {
                    kind: HirExprKind::Literal(HirLiteral::Unit),
                    ty: Type::Unit,
                    span: expr.span,
                }
            }
            None => {
                self.internal("missing resolution for name", expr.span);
                HirExpr {
                    kind: HirExprKind::Literal(HirLiteral::Unit),
                    ty: Type::Unit,
                    span: expr.span,
                }
            }
        }
    }

    fn check_call(&mut self, span: Span, callee: &Expr, args: &[Expr]) -> HirExpr {
        let resolved = self.resolution.resolved(callee.span).cloned();
        let checked_args: Vec<_> = args.iter().map(|arg| self.check_expr(arg)).collect();
        match resolved {
            Some(ResolvedName::Function(id)) => {
                let signature = self.signatures.get(&id).cloned().unwrap_or_else(|| {
                    self.internal("missing signature for resolved function", callee.span);
                    FunctionSignature {
                        params: Vec::new(),
                        return_type: Type::Unit,
                    }
                });
                self.check_arity(signature.params.len(), checked_args.len(), span);
                for (expected, actual) in signature.params.iter().zip(checked_args.iter()) {
                    self.require_type(&actual.ty, expected, actual.span, "function argument");
                }
                let return_type = if checked_args.iter().any(|arg| arg.ty == Type::Never) {
                    Type::Never
                } else {
                    signature.return_type
                };
                HirExpr {
                    kind: HirExprKind::Call {
                        callee: HirCallee::Function(id),
                        args: checked_args,
                    },
                    ty: return_type,
                    span,
                }
            }
            Some(ResolvedName::Builtin(id)) => {
                let signature = hydra_stdlib::signature(id);
                let name = signature.name;
                self.check_arity(signature.params.len(), checked_args.len(), span);
                for (expected, actual) in signature.params.iter().zip(checked_args.iter()) {
                    let valid = match expected {
                        BuiltinParam::Exact(expected) => compatible(&actual.ty, expected),
                        BuiltinParam::Printable => matches!(
                            actual.ty,
                            Type::Int
                                | Type::Float
                                | Type::Bool
                                | Type::String
                                | Type::Unit
                                | Type::Never
                        ),
                    };
                    if !valid {
                        self.type_error(
                            "E3007",
                            format!(
                                "unsupported argument type {} for builtin `{name}`",
                                actual.ty
                            ),
                            actual.span,
                        );
                    }
                }
                let return_type = if checked_args.iter().any(|arg| arg.ty == Type::Never) {
                    Type::Never
                } else {
                    signature.return_type
                };
                HirExpr {
                    kind: HirExprKind::Call {
                        callee: HirCallee::Builtin(id),
                        args: checked_args,
                    },
                    ty: return_type,
                    span,
                }
            }
            Some(ResolvedName::Local(_)) => {
                self.type_error(
                    "E3008",
                    "calling local values is not supported in Hydra 0.1",
                    callee.span,
                );
                HirExpr {
                    kind: HirExprKind::Literal(HirLiteral::Unit),
                    ty: Type::Unit,
                    span,
                }
            }
            None => {
                self.internal("call target has no resolution", callee.span);
                HirExpr {
                    kind: HirExprKind::Literal(HirLiteral::Unit),
                    ty: Type::Unit,
                    span,
                }
            }
        }
    }

    fn check_binary(&mut self, span: Span, left: &Expr, op: BinaryOp, right: &Expr) -> HirExpr {
        let left = self.check_expr(left);
        let right = self.check_expr(right);
        let ty = if left.ty == Type::Never {
            let valid = match op {
                BinaryOp::Add => matches!(
                    right.ty,
                    Type::Int | Type::Float | Type::String | Type::Never
                ),
                BinaryOp::Subtract
                | BinaryOp::Multiply
                | BinaryOp::Divide
                | BinaryOp::Remainder
                | BinaryOp::Less
                | BinaryOp::LessEqual
                | BinaryOp::Greater
                | BinaryOp::GreaterEqual => {
                    matches!(right.ty, Type::Int | Type::Float | Type::Never)
                }
                BinaryOp::Equal | BinaryOp::NotEqual => true,
                BinaryOp::And | BinaryOp::Or => {
                    matches!(right.ty, Type::Bool | Type::Never)
                }
            };
            if valid {
                Type::Never
            } else {
                self.type_error(
                    "E3003",
                    format!(
                        "operator {op:?} is not defined for {} and {}",
                        left.ty, right.ty
                    ),
                    span,
                );
                Type::Unit
            }
        } else if right.ty == Type::Never {
            match op {
                BinaryOp::Add if matches!(left.ty, Type::Int | Type::Float | Type::String) => {
                    Type::Never
                }
                BinaryOp::Subtract
                | BinaryOp::Multiply
                | BinaryOp::Divide
                | BinaryOp::Remainder
                    if matches!(left.ty, Type::Int | Type::Float) =>
                {
                    Type::Never
                }
                BinaryOp::Equal | BinaryOp::NotEqual => Type::Never,
                BinaryOp::Less
                | BinaryOp::LessEqual
                | BinaryOp::Greater
                | BinaryOp::GreaterEqual
                    if matches!(left.ty, Type::Int | Type::Float) =>
                {
                    Type::Never
                }
                BinaryOp::And | BinaryOp::Or if left.ty == Type::Bool => {
                    // A statically known left operand can force evaluation of
                    // the Never-typed RHS, leaving no normal Boolean value.
                    // Otherwise short-circuiting still supplies a Bool path.
                    let forces_rhs = matches!(
                        (&left.kind, op),
                        (HirExprKind::Literal(HirLiteral::Bool(true)), BinaryOp::And)
                            | (HirExprKind::Literal(HirLiteral::Bool(false)), BinaryOp::Or)
                    );
                    if forces_rhs { Type::Never } else { Type::Bool }
                }
                _ => {
                    self.type_error(
                        "E3003",
                        format!(
                            "operator {op:?} is not defined for {} and {}",
                            left.ty, right.ty
                        ),
                        span,
                    );
                    Type::Unit
                }
            }
        } else {
            match op {
                BinaryOp::Add
                    if left.ty == right.ty
                        && matches!(left.ty, Type::Int | Type::Float | Type::String) =>
                {
                    left.ty.clone()
                }
                BinaryOp::Subtract
                | BinaryOp::Multiply
                | BinaryOp::Divide
                | BinaryOp::Remainder
                    if left.ty == right.ty && matches!(left.ty, Type::Int | Type::Float) =>
                {
                    left.ty.clone()
                }
                BinaryOp::Equal | BinaryOp::NotEqual if left.ty == right.ty => Type::Bool,
                BinaryOp::Less
                | BinaryOp::LessEqual
                | BinaryOp::Greater
                | BinaryOp::GreaterEqual
                    if left.ty == right.ty && matches!(left.ty, Type::Int | Type::Float) =>
                {
                    Type::Bool
                }
                BinaryOp::And | BinaryOp::Or if left.ty == Type::Bool && right.ty == Type::Bool => {
                    Type::Bool
                }
                _ => {
                    self.type_error(
                        "E3003",
                        format!(
                            "operator {op:?} is not defined for {} and {}",
                            left.ty, right.ty
                        ),
                        span,
                    );
                    Type::Unit
                }
            }
        };
        HirExpr {
            kind: HirExprKind::Binary {
                left: Box::new(left),
                op: map_binary(op),
                right: Box::new(right),
            },
            ty,
            span,
        }
    }

    fn check_arity(&mut self, expected: usize, actual: usize, span: Span) {
        if expected != actual {
            self.type_error(
                "E3009",
                format!("expected {expected} arguments, found {actual}"),
                span,
            );
        }
    }

    fn require_type(&mut self, actual: &Type, expected: &Type, span: Span, context: &str) {
        if !compatible(actual, expected) {
            self.type_error(
                "E3002",
                format!("{context}: expected {expected}, found {actual}"),
                span,
            );
        }
    }

    fn type_error(&mut self, code: &'static str, message: impl Into<String>, span: Span) {
        self.diagnostics
            .push(Diagnostic::error(code, Phase::Type, message, span));
    }

    fn internal(&mut self, message: &'static str, span: Span) {
        self.diagnostics
            .push(Diagnostic::error("E9003", Phase::Internal, message, span));
    }
}

fn compatible(actual: &Type, expected: &Type) -> bool {
    actual == expected || *actual == Type::Never
}

fn map_binary(op: BinaryOp) -> HirBinaryOp {
    match op {
        BinaryOp::Add => HirBinaryOp::Add,
        BinaryOp::Subtract => HirBinaryOp::Subtract,
        BinaryOp::Multiply => HirBinaryOp::Multiply,
        BinaryOp::Divide => HirBinaryOp::Divide,
        BinaryOp::Remainder => HirBinaryOp::Remainder,
        BinaryOp::Equal => HirBinaryOp::Equal,
        BinaryOp::NotEqual => HirBinaryOp::NotEqual,
        BinaryOp::Less => HirBinaryOp::Less,
        BinaryOp::LessEqual => HirBinaryOp::LessEqual,
        BinaryOp::Greater => HirBinaryOp::Greater,
        BinaryOp::GreaterEqual => HirBinaryOp::GreaterEqual,
        BinaryOp::And => HirBinaryOp::And,
        BinaryOp::Or => HirBinaryOp::Or,
    }
}

#[cfg(test)]
mod tests {
    use super::check;
    use hydra_hir::HirProgram;
    use hydra_lexer::lex;
    use hydra_parser::parse;
    use hydra_resolve::resolve;
    use hydra_source::SourceId;
    use hydra_types::Type;

    fn check_source(source_text: &str) -> HirProgram {
        let source = SourceId::new(0);
        let lexed = lex(source, source_text);
        assert!(lexed.diagnostics.is_empty(), "{:?}", lexed.diagnostics);
        let parsed = parse(&lexed.tokens);
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let resolved = resolve(&parsed.program);
        assert!(
            resolved.diagnostics.is_empty(),
            "{:?}",
            resolved.diagnostics
        );
        let checked = check(&parsed.program, &resolved.resolution);
        assert!(checked.diagnostics.is_empty(), "{:?}", checked.diagnostics);
        checked.hir.expect("valid program should lower to HIR")
    }

    fn function_body_type(hir: &HirProgram, name: &str) -> Type {
        hir.functions
            .iter()
            .find(|function| function.name == name)
            .unwrap_or_else(|| panic!("missing function `{name}`"))
            .body
            .ty
            .clone()
    }

    #[test]
    fn lowers_typed_function_to_hir() {
        let source = SourceId::new(0);
        let lexed = lex(source, "fn add(a: Int, b: Int) -> Int {\n a + b\n}\n");
        let parsed = parse(&lexed.tokens);
        let resolved = resolve(&parsed.program);
        let checked = check(&parsed.program, &resolved.resolution);
        assert!(checked.diagnostics.is_empty(), "{:?}", checked.diagnostics);
        assert_eq!(checked.hir.as_ref().map(|hir| hir.functions.len()), Some(1));
    }

    #[test]
    fn never_propagates_through_strict_expression_contexts() {
        let cases = [
            ("fn f() -> Int {\n -{ return 7 }\n}\n", "unary"),
            ("fn f() -> Int {\n 1 + { return 7 }\n}\n", "binary rhs"),
            ("fn f() -> Int {\n { return 7 } + 1\n}\n", "binary lhs"),
            (
                "fn id(x: Int) -> Int { x }\nfn f() -> Int {\n id({ return 7 })\n}\n",
                "function argument",
            ),
            (
                "fn f() -> Int {\n let mut x = 0\n x = { return 7 }\n}\n",
                "assignment rhs",
            ),
            (
                "fn f() -> Int {\n if { return 7 } { 1 } else { 2 }\n}\n",
                "if condition",
            ),
        ];

        for (source, context) in cases {
            let hir = check_source(source);
            assert_eq!(
                function_body_type(&hir, "f"),
                Type::Never,
                "{context} should make the enclosing expression diverge"
            );
        }
    }

    #[test]
    fn never_is_accepted_by_print_builtins_and_propagates() {
        let hir = check_source(
            "fn with_print() {\n print({ return })\n}\n\
             fn with_println() {\n println({ return })\n}\n",
        );
        assert_eq!(function_body_type(&hir, "with_print"), Type::Never);
        assert_eq!(function_body_type(&hir, "with_println"), Type::Never);
    }

    #[test]
    fn short_circuit_rhs_never_keeps_boolean_expression_type() {
        let hir = check_source(
            "fn and_case(flag: Bool) -> Bool {\n flag && { return true }\n}\n\
             fn or_case(flag: Bool) -> Bool {\n flag || { return false }\n}\n",
        );
        assert_eq!(function_body_type(&hir, "and_case"), Type::Bool);
        assert_eq!(function_body_type(&hir, "or_case"), Type::Bool);
    }

    #[test]
    fn checker_function_state_resets_after_diverging_function() {
        let hir = check_source(
            "fn diverge(flag: Bool) -> Int {\n if flag { return 1 } else { return 2 }\n}\n\
             fn ordinary() -> Int {\n let value = 2\n value + 3\n}\n",
        );
        assert_eq!(function_body_type(&hir, "diverge"), Type::Never);
        assert_eq!(function_body_type(&hir, "ordinary"), Type::Int);
    }

    #[test]
    fn checker_error_in_one_function_does_not_leak_into_the_next_function() {
        let source = SourceId::new(0);
        let lexed = lex(
            source,
            "fn broken() -> Bool {\n 1\n}\n\
             fn valid() -> Int {\n let value = 2\n value + 3\n}\n",
        );
        assert!(lexed.diagnostics.is_empty(), "{:?}", lexed.diagnostics);
        let parsed = parse(&lexed.tokens);
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let resolved = resolve(&parsed.program);
        assert!(
            resolved.diagnostics.is_empty(),
            "{:?}",
            resolved.diagnostics
        );

        let checked = check(&parsed.program, &resolved.resolution);
        let codes: Vec<_> = checked.diagnostics.iter().map(|d| d.code).collect();
        assert_eq!(codes, vec!["E3002"]);
    }
}
