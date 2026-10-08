use hydra_diagnostics::{Diagnostic, Phase};
use hydra_hir::{
    HirBinaryOp, HirBlock, HirCallee, HirExpr, HirExprKind, HirFunction, HirLiteral, HirProgram,
    HirStmt, HirUnaryOp,
};
use hydra_resolve::{FunctionId, SymbolId};
use hydra_source::Span;
use hydra_stdlib::BuiltinId;
use std::collections::BTreeMap;
use std::fmt::{self, Write as _};

const MAX_CALL_DEPTH: usize = 128;
const DEFAULT_STEP_BUDGET: u64 = 1_000_000;

type RuntimeResult<T> = Result<T, Box<Diagnostic>>;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Unit,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int(value) => write!(f, "{value}"),
            Self::Float(value) => write!(f, "{value}"),
            Self::Bool(value) => write!(f, "{value}"),
            Self::String(value) => f.write_str(value),
            Self::Unit => f.write_str("()"),
        }
    }
}

#[derive(Debug)]
pub struct RunResult {
    pub output: String,
    pub value: Option<Value>,
    pub diagnostics: Vec<Diagnostic>,
}

#[must_use]
pub fn execute(program: &HirProgram) -> RunResult {
    Interpreter::new(program).run_main()
}

enum Flow {
    Value(Value),
    Return(Value),
    Break(Span),
    Continue(Span),
}

struct Interpreter<'a> {
    program: &'a HirProgram,
    output: String,
    steps_remaining: u64,
    call_depth: usize,
}

impl<'a> Interpreter<'a> {
    fn new(program: &'a HirProgram) -> Self {
        Self {
            program,
            output: String::new(),
            steps_remaining: DEFAULT_STEP_BUDGET,
            call_depth: 0,
        }
    }

    fn run_main(mut self) -> RunResult {
        let Some(main) = self
            .program
            .functions
            .iter()
            .find(|function| function.name == "main")
        else {
            let span = self.program.functions.first().map_or_else(
                || Span::empty(hydra_source::SourceId::new(0), 0),
                |function| function.span,
            );
            return RunResult {
                output: self.output,
                value: None,
                diagnostics: vec![Diagnostic::error(
                    "E4001",
                    Phase::Runtime,
                    "program has no `main` function",
                    span,
                )],
            };
        };
        if !main.params.is_empty() {
            return RunResult {
                output: self.output,
                value: None,
                diagnostics: vec![Diagnostic::error(
                    "E4002",
                    Phase::Runtime,
                    "`main` must not take parameters",
                    main.span,
                )],
            };
        }
        match self.call_function(main.id, Vec::new(), main.span) {
            Ok(value) => RunResult {
                output: self.output,
                value: Some(value),
                diagnostics: Vec::new(),
            },
            Err(diagnostic) => RunResult {
                output: self.output,
                value: None,
                diagnostics: vec![*diagnostic],
            },
        }
    }

    fn call_function(
        &mut self,
        id: FunctionId,
        args: Vec<Value>,
        call_span: Span,
    ) -> RuntimeResult<Value> {
        let Some(function) = self.function(id).cloned() else {
            return Err(self.runtime_error("E9004", "missing HIR function", call_span));
        };
        if args.len() != function.params.len() {
            return Err(self.runtime_error(
                "E9004",
                "HIR call arity disagrees with function signature",
                call_span,
            ));
        }
        if self.call_depth >= MAX_CALL_DEPTH {
            return Err(self.runtime_error(
                "E4003",
                "maximum call depth of 128 exceeded",
                call_span,
            ));
        }
        let mut frame = BTreeMap::new();
        for (param, value) in function.params.iter().zip(args) {
            frame.insert(param.symbol, value);
        }
        self.call_depth += 1;
        let result = self.eval_block(&function.body, &mut frame);
        self.call_depth = self.call_depth.saturating_sub(1);
        match result? {
            Flow::Value(value) | Flow::Return(value) => Ok(value),
            Flow::Break(span) | Flow::Continue(span) => Err(self.runtime_error(
                "E9004",
                "loop control escaped a function boundary in typed HIR",
                span,
            )),
        }
    }

    fn eval_block(
        &mut self,
        block: &HirBlock,
        frame: &mut BTreeMap<SymbolId, Value>,
    ) -> RuntimeResult<Flow> {
        for stmt in &block.statements {
            match self.eval_stmt(stmt, frame)? {
                Flow::Value(_) => {}
                flow => return Ok(flow),
            }
        }
        if let Some(tail) = &block.tail {
            self.eval_expr(tail, frame)
        } else {
            Ok(Flow::Value(Value::Unit))
        }
    }

    fn eval_stmt(
        &mut self,
        stmt: &HirStmt,
        frame: &mut BTreeMap<SymbolId, Value>,
    ) -> RuntimeResult<Flow> {
        let span = match stmt {
            HirStmt::Let { span, .. }
            | HirStmt::While { span, .. }
            | HirStmt::Return { span, .. }
            | HirStmt::Break { span }
            | HirStmt::Continue { span } => *span,
            HirStmt::Expr(expr) => expr.span,
        };
        self.tick(span)?;
        match stmt {
            HirStmt::Let { symbol, init, .. } => match self.eval_expr(init, frame)? {
                Flow::Value(value) => {
                    frame.insert(*symbol, value);
                    Ok(Flow::Value(Value::Unit))
                }
                flow => Ok(flow),
            },
            HirStmt::Expr(expr) => match self.eval_expr(expr, frame)? {
                Flow::Value(_) => Ok(Flow::Value(Value::Unit)),
                flow => Ok(flow),
            },
            HirStmt::Break { span } => Ok(Flow::Break(*span)),
            HirStmt::Continue { span } => Ok(Flow::Continue(*span)),
            HirStmt::While {
                condition, body, ..
            } => {
                loop {
                    let condition_value = match self.eval_expr(condition, frame)? {
                        Flow::Value(value) => value,
                        flow => return Ok(flow),
                    };
                    let Value::Bool(condition_value) = condition_value else {
                        return Err(self.runtime_error(
                            "E9004",
                            "typed HIR contains a non-boolean while condition",
                            condition.span,
                        ));
                    };
                    if !condition_value {
                        break;
                    }
                    match self.eval_block(body, frame)? {
                        Flow::Value(_) | Flow::Continue(_) => self.tick(body.span)?,
                        Flow::Break(_) => break,
                        flow @ Flow::Return(_) => return Ok(flow),
                    }
                }
                Ok(Flow::Value(Value::Unit))
            }
            HirStmt::Return { value, .. } => {
                let value = if let Some(value) = value {
                    match self.eval_expr(value, frame)? {
                        Flow::Value(value) => value,
                        flow => return Ok(flow),
                    }
                } else {
                    Value::Unit
                };
                Ok(Flow::Return(value))
            }
        }
    }

    fn eval_expr(
        &mut self,
        expr: &HirExpr,
        frame: &mut BTreeMap<SymbolId, Value>,
    ) -> RuntimeResult<Flow> {
        self.tick(expr.span)?;
        match &expr.kind {
            HirExprKind::Literal(literal) => Ok(Flow::Value(match literal {
                HirLiteral::Int(value) => Value::Int(*value),
                HirLiteral::Float(value) => Value::Float(*value),
                HirLiteral::Bool(value) => Value::Bool(*value),
                HirLiteral::String(value) => Value::String(value.clone()),
                HirLiteral::Unit => Value::Unit,
            })),
            HirExprKind::Local(symbol) => {
                frame.get(symbol).cloned().map(Flow::Value).ok_or_else(|| {
                    self.runtime_error("E9004", "typed HIR references an unbound local", expr.span)
                })
            }
            HirExprKind::Unary { op, operand } => {
                let value = match self.eval_expr(operand, frame)? {
                    Flow::Value(value) => value,
                    flow => return Ok(flow),
                };
                self.eval_unary(*op, value, expr.span).map(Flow::Value)
            }
            HirExprKind::Binary { left, op, right } => {
                let left = match self.eval_expr(left, frame)? {
                    Flow::Value(value) => value,
                    flow => return Ok(flow),
                };
                if *op == HirBinaryOp::And && left == Value::Bool(false) {
                    return Ok(Flow::Value(Value::Bool(false)));
                }
                if *op == HirBinaryOp::Or && left == Value::Bool(true) {
                    return Ok(Flow::Value(Value::Bool(true)));
                }
                let right = match self.eval_expr(right, frame)? {
                    Flow::Value(value) => value,
                    flow => return Ok(flow),
                };
                self.eval_binary(*op, left, right, expr.span)
                    .map(Flow::Value)
            }
            HirExprKind::Assign { symbol, value } => {
                let value = match self.eval_expr(value, frame)? {
                    Flow::Value(value) => value,
                    flow => return Ok(flow),
                };
                let Some(slot) = frame.get_mut(symbol) else {
                    return Err(self.runtime_error(
                        "E9004",
                        "typed HIR assignment references an unbound local",
                        expr.span,
                    ));
                };
                *slot = value;
                Ok(Flow::Value(Value::Unit))
            }
            HirExprKind::Call { callee, args } => {
                let mut values = Vec::with_capacity(args.len());
                for arg in args {
                    match self.eval_expr(arg, frame)? {
                        Flow::Value(value) => values.push(value),
                        flow => return Ok(flow),
                    }
                }
                let value = match callee {
                    HirCallee::Function(id) => self.call_function(*id, values, expr.span)?,
                    HirCallee::Builtin(id) => self.call_builtin(*id, values, expr.span)?,
                };
                Ok(Flow::Value(value))
            }
            HirExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let condition = match self.eval_expr(condition, frame)? {
                    Flow::Value(value) => value,
                    flow => return Ok(flow),
                };
                let Value::Bool(condition) = condition else {
                    return Err(self.runtime_error(
                        "E9004",
                        "typed HIR contains a non-boolean if condition",
                        expr.span,
                    ));
                };
                if condition {
                    self.eval_block(then_branch, frame)
                } else if let Some(else_branch) = else_branch {
                    self.eval_expr(else_branch, frame)
                } else {
                    Ok(Flow::Value(Value::Unit))
                }
            }
            HirExprKind::Block(block) => self.eval_block(block, frame),
        }
    }

    fn eval_unary(&self, op: HirUnaryOp, value: Value, span: Span) -> RuntimeResult<Value> {
        match (op, value) {
            (HirUnaryOp::Negate, Value::Int(value)) => value
                .checked_neg()
                .map(Value::Int)
                .ok_or_else(|| self.runtime_error("E4004", "integer overflow", span)),
            (HirUnaryOp::Negate, Value::Float(value)) => Ok(Value::Float(-value)),
            (HirUnaryOp::Not, Value::Bool(value)) => Ok(Value::Bool(!value)),
            _ => Err(self.runtime_error("E9004", "invalid typed unary operation", span)),
        }
    }

    fn eval_binary(
        &self,
        op: HirBinaryOp,
        left: Value,
        right: Value,
        span: Span,
    ) -> RuntimeResult<Value> {
        use HirBinaryOp as Op;
        match (op, left, right) {
            (Op::Add, Value::Int(a), Value::Int(b)) => checked_int(a.checked_add(b), span, self),
            (Op::Subtract, Value::Int(a), Value::Int(b)) => {
                checked_int(a.checked_sub(b), span, self)
            }
            (Op::Multiply, Value::Int(a), Value::Int(b)) => {
                checked_int(a.checked_mul(b), span, self)
            }
            (Op::Divide, Value::Int(_), Value::Int(0))
            | (Op::Remainder, Value::Int(_), Value::Int(0)) => {
                Err(self.runtime_error("E4005", "integer division by zero", span))
            }
            (Op::Divide, Value::Int(a), Value::Int(b)) => checked_int(a.checked_div(b), span, self),
            (Op::Remainder, Value::Int(a), Value::Int(b)) => {
                checked_int(a.checked_rem(b), span, self)
            }
            (Op::Add, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Op::Subtract, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            (Op::Multiply, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
            (Op::Divide, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
            (Op::Remainder, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a % b)),
            (Op::Add, Value::String(mut a), Value::String(b)) => {
                a.push_str(&b);
                Ok(Value::String(a))
            }
            (Op::Equal, a, b) => Ok(Value::Bool(a == b)),
            (Op::NotEqual, a, b) => Ok(Value::Bool(a != b)),
            (Op::Less, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a < b)),
            (Op::LessEqual, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a <= b)),
            (Op::Greater, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a > b)),
            (Op::GreaterEqual, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a >= b)),
            (Op::Less, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a < b)),
            (Op::LessEqual, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a <= b)),
            (Op::Greater, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a > b)),
            (Op::GreaterEqual, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a >= b)),
            (Op::And, Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a && b)),
            (Op::Or, Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a || b)),
            _ => Err(self.runtime_error("E9004", "invalid typed binary operation", span)),
        }
    }

    fn call_builtin(
        &mut self,
        id: BuiltinId,
        args: Vec<Value>,
        span: Span,
    ) -> RuntimeResult<Value> {
        let signature = hydra_stdlib::signature(id);
        if args.len() != signature.params.len() {
            return Err(self.runtime_error(
                "E9004",
                "HIR builtin call arity disagrees with builtin signature",
                span,
            ));
        }
        match id {
            BuiltinId::Print => {
                let Some(value) = args.into_iter().next() else {
                    return Err(self.runtime_error(
                        "E9004",
                        "HIR builtin call arity disagrees with builtin signature",
                        span,
                    ));
                };
                let _ = write!(self.output, "{value}");
                Ok(Value::Unit)
            }
            BuiltinId::Println => {
                let Some(value) = args.into_iter().next() else {
                    return Err(self.runtime_error(
                        "E9004",
                        "HIR builtin call arity disagrees with builtin signature",
                        span,
                    ));
                };
                let _ = writeln!(self.output, "{value}");
                Ok(Value::Unit)
            }
        }
    }

    fn function(&self, id: FunctionId) -> Option<&HirFunction> {
        self.program
            .functions
            .iter()
            .find(|function| function.id == id)
    }

    fn tick(&mut self, span: Span) -> RuntimeResult<()> {
        if self.steps_remaining == 0 {
            return Err(self.runtime_error("E4006", "execution step budget exhausted", span));
        }
        self.steps_remaining -= 1;
        Ok(())
    }

    fn runtime_error(
        &self,
        code: &'static str,
        message: &'static str,
        span: Span,
    ) -> Box<Diagnostic> {
        Box::new(Diagnostic::error(code, Phase::Runtime, message, span))
    }
}

fn checked_int(
    value: Option<i64>,
    span: Span,
    interpreter: &Interpreter<'_>,
) -> RuntimeResult<Value> {
    value
        .map(Value::Int)
        .ok_or_else(|| interpreter.runtime_error("E4004", "integer overflow", span))
}

#[cfg(test)]
mod tests {
    use super::{Interpreter, Value, execute};
    use hydra_hir::{
        HirBlock, HirCallee, HirExpr, HirExprKind, HirFunction, HirLiteral, HirProgram,
    };
    use hydra_resolve::{FunctionId, SymbolId};
    use hydra_source::{SourceId, Span};
    use hydra_stdlib::BuiltinId;
    use hydra_types::Type;

    fn literal_int(value: i64, span: Span) -> HirExpr {
        HirExpr {
            kind: HirExprKind::Literal(HirLiteral::Int(value)),
            ty: Type::Int,
            span,
        }
    }

    #[test]
    fn malformed_builtin_arity_is_rejected_as_invalid_hir() {
        let span = Span::new(SourceId::new(0), 0, 1);
        let call = HirExpr {
            kind: HirExprKind::Call {
                callee: HirCallee::Builtin(BuiltinId::Println),
                args: vec![literal_int(1, span), literal_int(2, span)],
            },
            ty: Type::Unit,
            span,
        };
        let program = HirProgram {
            functions: vec![HirFunction {
                id: FunctionId(0),
                name: "main".to_owned(),
                params: Vec::new(),
                return_type: Type::Unit,
                body: HirBlock {
                    statements: Vec::new(),
                    tail: Some(Box::new(call)),
                    ty: Type::Unit,
                    span,
                },
                span,
            }],
        };

        let result = execute(&program);
        assert_eq!(result.output, "");
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "E9004");
    }

    #[test]
    fn call_depth_is_restored_after_success_and_error() {
        let span = Span::new(SourceId::new(0), 0, 1);
        let good = HirFunction {
            id: FunctionId(0),
            name: "good".to_owned(),
            params: Vec::new(),
            return_type: Type::Int,
            body: HirBlock {
                statements: Vec::new(),
                tail: Some(Box::new(literal_int(7, span))),
                ty: Type::Int,
                span,
            },
            span,
        };
        let bad = HirFunction {
            id: FunctionId(1),
            name: "bad".to_owned(),
            params: Vec::new(),
            return_type: Type::Int,
            body: HirBlock {
                statements: Vec::new(),
                tail: Some(Box::new(HirExpr {
                    kind: HirExprKind::Local(SymbolId(999)),
                    ty: Type::Int,
                    span,
                })),
                ty: Type::Int,
                span,
            },
            span,
        };
        let program = HirProgram {
            functions: vec![good, bad],
        };
        let mut interpreter = Interpreter::new(&program);

        assert_eq!(
            interpreter.call_function(FunctionId(0), Vec::new(), span),
            Ok(Value::Int(7))
        );
        assert_eq!(interpreter.call_depth, 0);

        let error = interpreter
            .call_function(FunctionId(1), Vec::new(), span)
            .expect_err("malformed HIR local should fail");
        assert_eq!(error.code, "E9004");
        assert_eq!(interpreter.call_depth, 0);
    }

    #[test]
    fn escaped_loop_control_in_malformed_hir_preserves_keyword_span() {
        use hydra_hir::HirStmt;

        let function_span = Span::new(SourceId::new(4), 0, 38);
        for (statement, keyword_span) in [
            (
                HirStmt::Break {
                    span: Span::new(SourceId::new(4), 12, 17),
                },
                Span::new(SourceId::new(4), 12, 17),
            ),
            (
                HirStmt::Continue {
                    span: Span::new(SourceId::new(4), 20, 28),
                },
                Span::new(SourceId::new(4), 20, 28),
            ),
        ] {
            let program = HirProgram {
                functions: vec![HirFunction {
                    id: FunctionId(0),
                    name: "main".to_owned(),
                    params: Vec::new(),
                    return_type: Type::Unit,
                    body: HirBlock {
                        statements: vec![statement],
                        tail: None,
                        ty: Type::Never,
                        span: function_span,
                    },
                    span: function_span,
                }],
            };
            let result = execute(&program);
            assert_eq!(result.value, None);
            assert_eq!(result.diagnostics.len(), 1);
            assert_eq!(result.diagnostics[0].code, "E9004");
            assert_eq!(result.diagnostics[0].primary, keyword_span);
        }
    }
}
