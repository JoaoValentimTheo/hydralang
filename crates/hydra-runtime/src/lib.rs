use hydra_diagnostics::{Diagnostic, Phase};
use hydra_hir::{
    HirBinaryOp, HirBlock, HirCallee, HirExpr, HirExprKind, HirFunction, HirLiteral, HirProgram,
    HirStmt, HirUnaryOp,
};
use hydra_resolve::{FunctionId, SymbolId};
use hydra_source::Span;
use hydra_stdlib::BuiltinId;
use hydra_types::Type;
use std::collections::{BTreeMap, HashSet};
use std::fmt::{self, Write as _};
use std::rc::Rc;

const MAX_CALL_DEPTH: usize = 128;
const DEFAULT_STEP_BUDGET: u64 = 1_000_000;

type RuntimeResult<T> = Result<T, Box<Diagnostic>>;

#[derive(Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Unit,
    Tuple(Rc<[Value]>),
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int(v) => f.debug_tuple("Int").field(v).finish(),
            Self::Float(v) => f.debug_tuple("Float").field(v).finish(),
            Self::Bool(v) => f.debug_tuple("Bool").field(v).finish(),
            Self::String(v) => f.debug_tuple("String").field(v).finish(),
            Self::Unit => f.write_str("Unit"),
            Self::Tuple(v) => f.debug_tuple("Tuple").field(&v.len()).finish(),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        values_equal(self, other, || Ok(())).unwrap_or(false)
    }
}

// A first-seen pair always checks descendants, including identical Rc nodes:
// a shared leaf may contain NaN, for which semantic equality is false.
fn values_equal(
    left: &Value,
    right: &Value,
    mut charge: impl FnMut() -> RuntimeResult<()>,
) -> RuntimeResult<bool> {
    let mut pending = vec![(left, right, false)];
    let mut visited = HashSet::new();
    while let Some((left, right, field)) = pending.pop() {
        if field {
            charge()?;
        }
        match (left, right) {
            (Value::Tuple(a), Value::Tuple(b)) => {
                charge()?;
                if a.len() != b.len() {
                    return Ok(false);
                }
                let key = (
                    Rc::as_ptr(a) as *const () as usize,
                    Rc::as_ptr(b) as *const () as usize,
                );
                if visited.insert(key) {
                    for (lhs, rhs) in a.iter().zip(b.iter()).rev() {
                        pending.push((lhs, rhs, true));
                    }
                }
            }
            (Value::Int(a), Value::Int(b)) if a == b => {}
            (Value::Float(a), Value::Float(b)) if a == b => {}
            (Value::Bool(a), Value::Bool(b)) if a == b => {}
            (Value::String(a), Value::String(b)) if a == b => {}
            (Value::Unit, Value::Unit) => {}
            _ => return Ok(false),
        }
    }
    Ok(true)
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int(value) => write!(f, "{value}"),
            Self::Float(value) => write!(f, "{value}"),
            Self::Bool(value) => write!(f, "{value}"),
            Self::String(value) => f.write_str(value),
            Self::Unit => f.write_str("()"),
            Self::Tuple(_) => f.write_str("<tuple>"),
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
            self.validate_runtime_value_type(&value, &param.ty, call_span)?;
            frame.insert(param.symbol, value);
        }
        self.call_depth += 1;
        let result = self.eval_block(&function.body, &mut frame);
        self.call_depth = self.call_depth.saturating_sub(1);
        match result? {
            Flow::Value(value) | Flow::Return(value) => {
                self.validate_runtime_value_type(&value, &function.return_type, call_span)?;
                Ok(value)
            }
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
            HirExprKind::Tuple(fields) => {
                if fields.is_empty() || fields.len() > 64 {
                    return Err(self.runtime_error(
                        "E9004",
                        "typed HIR tuple arity must be between 1 and 64",
                        expr.span,
                    ));
                }
                match &expr.ty {
                    Type::Tuple(tys)
                        if tys.len() == fields.len()
                            && tys
                                .iter()
                                .zip(fields)
                                .all(|(ty, field)| hir_types_match(ty, &field.ty)) => {}
                    Type::Never => {} // Valid D001 control transfer prevents normal construction.
                    _ => {
                        return Err(self.runtime_error(
                            "E9004",
                            "HIR tuple type disagrees with its fields",
                            expr.span,
                        ));
                    }
                }
                let mut values = Vec::new();
                values.try_reserve(fields.len()).map_err(|_| {
                    self.runtime_error("E9004", "tuple allocation failed", expr.span)
                })?;
                for field in fields {
                    self.tick(field.span)?;
                    match self.eval_expr(field, frame)? {
                        Flow::Value(value) => {
                            self.validate_runtime_value_type(&value, &field.ty, field.span)?;
                            values.push(value);
                        }
                        flow => return Ok(flow),
                    }
                }
                if expr.ty == Type::Never {
                    return Err(self.runtime_error(
                        "E9004",
                        "HIR Never tuple unexpectedly produced a normal value",
                        expr.span,
                    ));
                }
                Ok(Flow::Value(Value::Tuple(Rc::from(values))))
            }
            HirExprKind::Projection { base, index } => {
                if let Type::Tuple(types) = &base.ty {
                    if types.len() > 64
                        || types
                            .get(*index)
                            .is_none_or(|ty| !hir_types_match(ty, &expr.ty))
                    {
                        return Err(self.runtime_error(
                            "E9004",
                            "HIR projection type or index disagrees with its tuple base",
                            expr.span,
                        ));
                    }
                } else if base.ty != Type::Never || expr.ty != Type::Never {
                    return Err(self.runtime_error(
                        "E9004",
                        "HIR projection requires a tuple type",
                        expr.span,
                    ));
                }
                let base_value = match self.eval_expr(base, frame)? {
                    Flow::Value(value) => value,
                    flow => return Ok(flow),
                };
                if base.ty == Type::Never {
                    return Err(self.runtime_error(
                        "E9004",
                        "HIR Never projection base unexpectedly produced a normal value",
                        expr.span,
                    ));
                }
                // A fabricated HIR local may hide an invalid field that is not
                // selected. Check the whole immutable tuple before exposing a
                // projection as a correctly typed value.
                self.validate_runtime_value_type(&base_value, &base.ty, expr.span)?;
                let Value::Tuple(fields) = base_value else {
                    return Err(self.runtime_error(
                        "E9004",
                        "HIR projected a non-tuple value",
                        expr.span,
                    ));
                };
                if let Type::Tuple(types) = &base.ty {
                    if fields.is_empty() || fields.len() != types.len() {
                        return Err(self.runtime_error(
                            "E9004",
                            "HIR projection tuple arity disagrees with its type",
                            expr.span,
                        ));
                    }
                }
                let value = fields.get(*index).ok_or_else(|| {
                    self.runtime_error(
                        "E9004",
                        "HIR tuple projection index out of range",
                        expr.span,
                    )
                })?;
                self.validate_runtime_value_type(value, &expr.ty, expr.span)?;
                Ok(Flow::Value(value.clone()))
            }
            HirExprKind::Unary { op, operand } => {
                let value = match self.eval_expr(operand, frame)? {
                    Flow::Value(value) => value,
                    flow => return Ok(flow),
                };
                self.eval_unary(*op, value, expr.span).map(Flow::Value)
            }
            HirExprKind::Binary { left, op, right } => {
                self.eval_binary_expr(left, *op, right, &expr.ty, expr.span, frame)
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

    fn eval_binary_expr(
        &mut self,
        left: &HirExpr,
        op: HirBinaryOp,
        right: &HirExpr,
        result_ty: &Type,
        span: Span,
        frame: &mut BTreeMap<SymbolId, Value>,
    ) -> RuntimeResult<Flow> {
        let left_value = match self.eval_expr(left, frame)? {
            Flow::Value(value) => value,
            flow => return Ok(flow),
        };
        if op == HirBinaryOp::And && left_value == Value::Bool(false) {
            return Ok(Flow::Value(Value::Bool(false)));
        }
        if op == HirBinaryOp::Or && left_value == Value::Bool(true) {
            return Ok(Flow::Value(Value::Bool(true)));
        }
        let right_value = match self.eval_expr(right, frame)? {
            Flow::Value(value) => value,
            flow => return Ok(flow),
        };
        // This is reached after both operands have produced normal values.
        // Valid checked source guarantees their structural type identity.
        if matches!(op, HirBinaryOp::Equal | HirBinaryOp::NotEqual)
            && (matches!(left.ty, Type::Tuple(_))
                || matches!(right.ty, Type::Tuple(_))
                || matches!(left_value, Value::Tuple(_))
                || matches!(right_value, Value::Tuple(_)))
            && (!matches!((&left.ty, &right.ty), (Type::Tuple(_), Type::Tuple(_)))
                || !hir_types_match(&left.ty, &right.ty)
                || !matches!(result_ty, Type::Bool))
        {
            return Err(self.runtime_error(
                "E9004",
                "HIR tuple equality requires matching static tuple types",
                span,
            ));
        }
        if matches!((&left.ty, &right.ty), (Type::Tuple(_), Type::Tuple(_)))
            && matches!(op, HirBinaryOp::Equal | HirBinaryOp::NotEqual)
        {
            self.validate_runtime_value_type(&left_value, &left.ty, span)?;
            self.validate_runtime_value_type(&right_value, &right.ty, span)?;
        }
        self.eval_binary(op, left_value, right_value, span)
            .map(Flow::Value)
    }

    fn eval_binary(
        &mut self,
        op: HirBinaryOp,
        left: Value,
        right: Value,
        span: Span,
    ) -> RuntimeResult<Value> {
        use HirBinaryOp as Op;
        if matches!(op, Op::Equal | Op::NotEqual)
            && (matches!(left, Value::Tuple(_)) || matches!(right, Value::Tuple(_)))
        {
            if !matches!((&left, &right), (Value::Tuple(_), Value::Tuple(_))) {
                return Err(self.runtime_error(
                    "E9004",
                    "typed HIR tuple equality requires two tuples",
                    span,
                ));
            }
            let is_equal = values_equal(&left, &right, || self.tick(span))?;
            return Ok(Value::Bool(if op == Op::Equal {
                is_equal
            } else {
                !is_equal
            }));
        }
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
                if matches!(value, Value::Tuple(_)) {
                    return Err(self.runtime_error("E9004", "HIR cannot print a tuple", span));
                }
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
                if matches!(value, Value::Tuple(_)) {
                    return Err(self.runtime_error("E9004", "HIR cannot print a tuple", span));
                }
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

    // Checked source guarantees this invariant. Directly fabricated HIR may
    // instead lie about the runtime fields of a local tuple. Validate the
    // fields iteratively before returning a projection or tuple equality.
    fn validate_runtime_value_type(
        &mut self,
        value: &Value,
        ty: &Type,
        span: Span,
    ) -> RuntimeResult<()> {
        let mut pending = vec![(value, ty, 0_usize)];
        let mut seen = HashSet::new();
        while let Some((value, ty, depth)) = pending.pop() {
            match (value, ty) {
                (Value::Tuple(values), Type::Tuple(types)) => {
                    if depth >= 64
                        || values.is_empty()
                        || values.len() > 64
                        || values.len() != types.len()
                    {
                        return Err(self.runtime_error(
                            "E9004",
                            "runtime tuple shape disagrees with its HIR type",
                            span,
                        ));
                    }
                    let key = (
                        Rc::as_ptr(values) as *const () as usize,
                        ty as *const Type as usize,
                        // The same shared pair can be reached through paths
                        // with different depths. Validate each depth so that
                        // DAG memoization cannot bypass the 64-layer guard.
                        depth,
                    );
                    if seen.insert(key) {
                        for (field, field_ty) in values.iter().zip(types.iter()).rev() {
                            self.tick(span)?;
                            pending.push((field, field_ty, depth + 1));
                        }
                    }
                }
                (Value::Int(_), Type::Int)
                | (Value::Float(_), Type::Float)
                | (Value::Bool(_), Type::Bool)
                | (Value::String(_), Type::String)
                | (Value::Unit, Type::Unit) => {}
                _ => {
                    return Err(self.runtime_error(
                        "E9004",
                        "runtime value disagrees with its HIR type",
                        span,
                    ));
                }
            }
        }
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

// Iterative structural check: malformed hand-built HIR must not trigger recursive
// type comparisons or silently bypass source tuple arity/nesting limits.
fn hir_types_match(left: &Type, right: &Type) -> bool {
    let mut pending = vec![(left, right, 0_usize)];
    let mut seen = HashSet::new();
    while let Some((left, right, depth)) = pending.pop() {
        match (left, right) {
            (Type::Tuple(a), Type::Tuple(b)) => {
                if depth >= 64 || a.is_empty() || a.len() > 64 || a.len() != b.len() {
                    return false;
                }
                // Shape/depth must be validated before skipping a repeated
                // pair. Distinct depths are distinct visits.
                let key = (
                    Rc::as_ptr(a) as *const () as usize,
                    Rc::as_ptr(b) as *const () as usize,
                    depth,
                );
                if seen.insert(key) {
                    pending.extend(a.iter().zip(b.iter()).map(|(x, y)| (x, y, depth + 1)));
                }
            }
            (Type::Int, Type::Int)
            | (Type::Float, Type::Float)
            | (Type::Bool, Type::Bool)
            | (Type::String, Type::String)
            | (Type::Unit, Type::Unit)
            | (Type::Never, Type::Never) => {}
            _ => return false,
        }
    }
    true
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
    use super::{HirBinaryOp, Interpreter, Value, execute, hir_types_match, values_equal};
    use hydra_hir::{
        HirBlock, HirCallee, HirExpr, HirExprKind, HirFunction, HirLiteral, HirParam, HirProgram,
    };
    use hydra_resolve::{FunctionId, SymbolId};
    use hydra_source::{SourceId, Span};
    use hydra_stdlib::BuiltinId;
    use hydra_types::Type;
    use std::rc::Rc;

    fn tuple(fields: Vec<Value>) -> Value {
        Value::Tuple(Rc::from(fields))
    }

    fn program_with_tail(tail: HirExpr) -> HirProgram {
        let span = tail.span;
        HirProgram {
            functions: vec![HirFunction {
                id: FunctionId(0),
                name: "main".to_owned(),
                params: vec![],
                return_type: Type::Unit,
                body: HirBlock {
                    statements: vec![],
                    tail: Some(Box::new(tail)),
                    ty: Type::Unit,
                    span,
                },
                span,
            }],
        }
    }

    #[test]
    fn tuple_alias_nan_never_becomes_reflexively_equal() {
        let leaf = tuple(vec![Value::Float(f64::NAN)]);
        let shared = tuple(vec![leaf.clone(), leaf.clone()]);
        assert_ne!(leaf, leaf.clone());
        assert_ne!(shared, shared.clone());
        assert!(!values_equal(&shared, &shared, || Ok(())).unwrap());
        assert_eq!(format!("{shared}"), "<tuple>");
        assert_eq!(format!("{shared:?}"), "Tuple(2)");
    }

    #[test]
    fn shared_dag_equality_short_circuits_without_exponential_expansion() {
        let mut equal_left = tuple(vec![Value::Int(9)]);
        let mut equal_right = tuple(vec![Value::Int(9)]);
        let mut unequal_right = tuple(vec![Value::Int(8)]);
        for _ in 0..48 {
            equal_left = tuple(vec![equal_left.clone(), equal_left]);
            equal_right = tuple(vec![equal_right.clone(), equal_right]);
            unequal_right = tuple(vec![unequal_right.clone(), unequal_right]);
        }
        let mut work = 0;
        assert!(
            values_equal(&equal_left, &equal_right, || {
                work += 1;
                Ok(())
            })
            .unwrap()
        );
        assert!(work < 500, "repeated shared nodes expanded: {work}");
        let mut unequal_work = 0;
        assert!(
            !values_equal(&equal_left, &unequal_right, || {
                unequal_work += 1;
                Ok(())
            })
            .unwrap()
        );
        assert!(unequal_work < 500);
    }

    #[test]
    fn malformed_hir_shared_tuple_depth_respects_exact_sixty_four_layer_limit() {
        let span = Span::new(SourceId::new(17), 2, 11);
        let program = HirProgram { functions: vec![] };
        let mut vm = Interpreter::new(&program);
        let mut ty = Type::tuple(vec![Type::Int]);
        let mut value = tuple(vec![Value::Int(3)]);

        // The logical tree doubles on every iteration, but physical storage
        // and the number of distinct identity pairs stay linear.
        for _ in 1..64 {
            ty = Type::tuple(vec![ty.clone(), ty]);
            value = tuple(vec![value.clone(), value]);
        }
        assert!(hir_types_match(&ty, &ty));
        assert!(vm.validate_runtime_value_type(&value, &ty, span).is_ok());

        // A fabricated HIR may nest one layer beyond the source type limit;
        // sharing and memoization must not conceal its excess depth.
        let invalid_ty = Type::tuple(vec![ty]);
        let invalid_value = tuple(vec![value]);
        assert!(!hir_types_match(&invalid_ty, &invalid_ty));
        let error = vm
            .validate_runtime_value_type(&invalid_value, &invalid_ty, span)
            .expect_err("malformed HIR with 65 tuple layers must be rejected");
        assert_eq!(error.code, "E9004");
        assert_eq!(error.primary, span);
    }

    #[test]
    fn tuple_equality_consumes_existing_fuel_and_not_equal_is_complement() {
        let span = Span::new(SourceId::new(7), 3, 10);
        let program = HirProgram { functions: vec![] };
        let mut vm = Interpreter::new(&program);
        vm.steps_remaining = 2;
        let input = tuple(vec![Value::Int(1), Value::Int(2)]);
        let err = vm
            .eval_binary(HirBinaryOp::Equal, input.clone(), input.clone(), span)
            .expect_err("field comparisons must charge the existing budget");
        assert_eq!(err.code, "E4006");
        assert_eq!(err.primary, span);

        vm.steps_remaining = 200;
        assert_eq!(
            vm.eval_binary(HirBinaryOp::Equal, input.clone(), input.clone(), span),
            Ok(Value::Bool(true))
        );
        assert_eq!(
            vm.eval_binary(HirBinaryOp::NotEqual, input.clone(), input, span),
            Ok(Value::Bool(false))
        );
    }

    #[test]
    fn malformed_hir_tuple_scalar_equality_is_rejected() {
        let span = Span::new(SourceId::new(11), 5, 12);
        let program = HirProgram { functions: vec![] };
        let mut vm = Interpreter::new(&program);
        for op in [HirBinaryOp::Equal, HirBinaryOp::NotEqual] {
            for (left, right) in [
                (tuple(vec![Value::Int(1)]), Value::Int(1)),
                (Value::Int(1), tuple(vec![Value::Int(1)])),
            ] {
                let error = vm.eval_binary(op, left, right, span).unwrap_err();
                assert_eq!(error.code, "E9004");
                assert_eq!(error.primary, span);
            }
        }
    }

    #[test]
    fn malformed_hir_tuple_equality_static_types_must_match() {
        use std::collections::BTreeMap;

        let span = Span::new(SourceId::new(12), 3, 19);
        let program = HirProgram { functions: vec![] };
        let mut vm = Interpreter::new(&program);
        let mut frame = BTreeMap::new();
        frame.insert(SymbolId(1), tuple(vec![Value::Int(7)]));
        frame.insert(SymbolId(2), tuple(vec![Value::Int(7)]));

        for op in [HirBinaryOp::Equal, HirBinaryOp::NotEqual] {
            let binary = HirExpr {
                kind: HirExprKind::Binary {
                    left: Box::new(HirExpr {
                        kind: HirExprKind::Local(SymbolId(1)),
                        ty: Type::tuple(vec![Type::Int]),
                        span,
                    }),
                    op,
                    right: Box::new(HirExpr {
                        kind: HirExprKind::Local(SymbolId(2)),
                        ty: Type::tuple(vec![Type::Bool]),
                        span,
                    }),
                },
                ty: Type::Bool,
                span,
            };
            let err = match vm.eval_expr(&binary, &mut frame) {
                Err(err) => err,
                Ok(_) => panic!("inconsistent tuple equality HIR must not execute"),
            };
            assert_eq!(err.code, "E9004");
            assert_eq!(err.primary, span);
        }
    }

    #[test]
    fn malformed_hir_tuple_values_must_match_declared_field_types() {
        use std::collections::BTreeMap;

        let span = Span::new(SourceId::new(13), 2, 18);
        let program = HirProgram { functions: vec![] };
        let mut vm = Interpreter::new(&program);
        let mut frame = BTreeMap::new();
        frame.insert(SymbolId(1), tuple(vec![Value::Bool(true)]));
        let dishonest_local = || HirExpr {
            kind: HirExprKind::Local(SymbolId(1)),
            ty: Type::tuple(vec![Type::Int]),
            span,
        };

        let projection = HirExpr {
            kind: HirExprKind::Projection {
                base: Box::new(dishonest_local()),
                index: 0,
            },
            ty: Type::Int,
            span,
        };
        let equality = HirExpr {
            kind: HirExprKind::Binary {
                left: Box::new(dishonest_local()),
                op: HirBinaryOp::Equal,
                right: Box::new(dishonest_local()),
            },
            ty: Type::Bool,
            span,
        };
        for expression in [&projection, &equality] {
            let error = match vm.eval_expr(expression, &mut frame) {
                Err(error) => error,
                Ok(_) => panic!("incorrect runtime tuple field must not escape malformed HIR"),
            };
            assert_eq!(error.code, "E9004");
            assert_eq!(error.primary, span);
        }
    }

    #[test]
    fn malformed_hir_projection_rejects_corrupt_unselected_fields() {
        use std::collections::BTreeMap;

        let span = Span::new(SourceId::new(16), 7, 19);
        let program = HirProgram { functions: vec![] };
        let mut vm = Interpreter::new(&program);
        let mut frame = BTreeMap::new();
        frame.insert(SymbolId(3), tuple(vec![Value::Int(1), Value::Bool(true)]));
        let projection = HirExpr {
            kind: HirExprKind::Projection {
                base: Box::new(HirExpr {
                    kind: HirExprKind::Local(SymbolId(3)),
                    ty: Type::tuple(vec![Type::Int, Type::Int]),
                    span,
                }),
                index: 0,
            },
            ty: Type::Int,
            span,
        };
        let error = match vm.eval_expr(&projection, &mut frame) {
            Err(error) => error,
            Ok(_) => panic!("projection accepted malformed HIR with corrupt unselected field"),
        };
        assert_eq!(error.code, "E9004");
        assert_eq!(error.primary, span);
    }

    #[test]
    fn malformed_hir_tuple_construction_cannot_capture_wrongly_typed_local() {
        use std::collections::BTreeMap;

        let span = Span::new(SourceId::new(14), 4, 23);
        let program = HirProgram { functions: vec![] };
        let mut vm = Interpreter::new(&program);
        let mut frame = BTreeMap::new();
        frame.insert(SymbolId(1), tuple(vec![Value::Bool(true)]));

        let expression = HirExpr {
            kind: HirExprKind::Tuple(vec![HirExpr {
                kind: HirExprKind::Local(SymbolId(1)),
                ty: Type::tuple(vec![Type::Int]),
                span,
            }]),
            ty: Type::tuple(vec![Type::tuple(vec![Type::Int])]),
            span,
        };
        let err = match vm.eval_expr(&expression, &mut frame) {
            Err(err) => err,
            Ok(_) => panic!("malformed HIR constructed a tuple with wrongly typed fields"),
        };
        assert_eq!(err.code, "E9004");
        assert_eq!(err.primary, span);
    }

    #[test]
    fn malformed_hir_function_boundary_rejects_incorrect_tuple_values() {
        use hydra_hir::HirStmt;

        let span = Span::new(SourceId::new(15), 2, 17);
        let tuple_ty = Type::tuple(vec![Type::Int]);
        let function_with_param = HirFunction {
            id: FunctionId(0),
            name: "param".to_owned(),
            params: vec![HirParam {
                symbol: SymbolId(1),
                name: "x".to_owned(),
                ty: tuple_ty.clone(),
                span,
            }],
            return_type: tuple_ty.clone(),
            body: HirBlock {
                statements: vec![],
                tail: Some(Box::new(HirExpr {
                    kind: HirExprKind::Local(SymbolId(1)),
                    ty: tuple_ty.clone(),
                    span,
                })),
                ty: tuple_ty.clone(),
                span,
            },
            span,
        };
        let function_with_wrong_return = HirFunction {
            id: FunctionId(1),
            name: "return".to_owned(),
            params: vec![],
            return_type: tuple_ty.clone(),
            body: HirBlock {
                statements: vec![],
                tail: Some(Box::new(HirExpr {
                    kind: HirExprKind::Literal(HirLiteral::Bool(true)),
                    ty: tuple_ty.clone(),
                    span,
                })),
                ty: tuple_ty,
                span,
            },
            span,
        };
        let function_with_wrong_explicit_return = HirFunction {
            id: FunctionId(2),
            name: "explicit_return".to_owned(),
            params: vec![],
            return_type: Type::tuple(vec![Type::Int]),
            body: HirBlock {
                statements: vec![HirStmt::Return {
                    value: Some(HirExpr {
                        kind: HirExprKind::Literal(HirLiteral::Bool(true)),
                        ty: Type::tuple(vec![Type::Int]),
                        span,
                    }),
                    span,
                }],
                tail: None,
                ty: Type::Never,
                span,
            },
            span,
        };
        let program = HirProgram {
            functions: vec![
                function_with_param,
                function_with_wrong_return,
                function_with_wrong_explicit_return,
            ],
        };
        let mut vm = Interpreter::new(&program);
        for (id, args) in [
            (FunctionId(0), vec![tuple(vec![Value::Bool(true)])]),
            (FunctionId(1), Vec::new()),
            (FunctionId(2), Vec::new()),
        ] {
            let result = vm.call_function(id, args, span);
            let error = result.expect_err("malformed HIR must reject invalid tuple boundary");
            assert_eq!(error.code, "E9004");
            assert_eq!(error.primary, span);
            assert_eq!(vm.call_depth, 0);
        }
    }

    #[test]
    fn malformed_hir_tuple_projections_report_spanned_e9004() {
        let span = Span::new(SourceId::new(9), 11, 16);
        for base in [
            literal_int(3, span),
            HirExpr {
                kind: HirExprKind::Tuple(vec![literal_int(3, span)]),
                ty: Type::tuple(vec![Type::Int]),
                span,
            },
        ] {
            let index = if matches!(base.kind, HirExprKind::Tuple(_)) {
                2
            } else {
                0
            };
            let projection = HirExpr {
                kind: HirExprKind::Projection {
                    base: Box::new(base),
                    index,
                },
                ty: Type::Int,
                span,
            };
            let result = execute(&program_with_tail(projection));
            assert_eq!(result.value, None);
            assert_eq!(result.diagnostics.len(), 1);
            assert_eq!(result.diagnostics[0].code, "E9004");
            assert_eq!(result.diagnostics[0].primary, span);
        }
    }

    #[test]
    fn malformed_hir_tuple_over_arity_limit_reports_e9004() {
        let span = Span::new(SourceId::new(8), 0, 20);
        let expr = HirExpr {
            kind: HirExprKind::Tuple(vec![literal_int(2, span); 65]),
            ty: Type::tuple(vec![Type::Int; 65]),
            span,
        };
        let result = execute(&program_with_tail(expr));
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "E9004");
        assert_eq!(result.diagnostics[0].primary, span);
    }

    #[test]
    fn malformed_hir_tuple_static_shape_and_projection_are_rejected() {
        let span = Span::new(SourceId::new(10), 4, 17);
        for expr in [
            HirExpr {
                kind: HirExprKind::Tuple(vec![]),
                ty: Type::tuple(vec![]),
                span,
            },
            HirExpr {
                kind: HirExprKind::Tuple(vec![literal_int(1, span)]),
                ty: Type::Int,
                span,
            },
            HirExpr {
                kind: HirExprKind::Tuple(vec![literal_int(1, span)]),
                ty: Type::tuple(vec![Type::Int, Type::Bool]),
                span,
            },
            HirExpr {
                kind: HirExprKind::Projection {
                    base: Box::new(HirExpr {
                        kind: HirExprKind::Tuple(vec![literal_int(1, span)]),
                        ty: Type::tuple(vec![Type::Int]),
                        span,
                    }),
                    index: 0,
                },
                ty: Type::Bool,
                span,
            },
            HirExpr {
                kind: HirExprKind::Projection {
                    base: Box::new(HirExpr {
                        kind: HirExprKind::Literal(HirLiteral::Int(3)),
                        ty: Type::Never,
                        span,
                    }),
                    index: 0,
                },
                ty: Type::Never,
                span,
            },
        ] {
            let run = execute(&program_with_tail(expr));
            assert_eq!(run.diagnostics.len(), 1);
            assert_eq!(run.diagnostics[0].code, "E9004");
            assert_eq!(run.diagnostics[0].primary, span);
        }
    }

    #[test]
    fn malformed_hir_tuple_cannot_be_printed_through_builtin() {
        let span = Span::new(SourceId::new(11), 1, 12);
        let field = literal_int(9, span);
        let call = HirExpr {
            kind: HirExprKind::Call {
                callee: HirCallee::Builtin(BuiltinId::Println),
                args: vec![HirExpr {
                    kind: HirExprKind::Tuple(vec![field]),
                    ty: Type::tuple(vec![Type::Int]),
                    span,
                }],
            },
            ty: Type::Unit,
            span,
        };
        let run = execute(&program_with_tail(call));
        assert_eq!(run.output, "");
        assert_eq!(run.diagnostics.len(), 1);
        assert_eq!(run.diagnostics[0].code, "E9004");
        assert_eq!(run.diagnostics[0].primary, span);
    }

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

    #[test]
    fn nested_malformed_hir_escape_from_callee_restores_call_depth() {
        use hydra_hir::HirStmt;

        let span = Span::new(SourceId::new(7), 0, 80);
        for (statement, keyword_span) in [
            (
                HirStmt::Break {
                    span: Span::new(SourceId::new(7), 21, 26),
                },
                Span::new(SourceId::new(7), 21, 26),
            ),
            (
                HirStmt::Continue {
                    span: Span::new(SourceId::new(7), 35, 43),
                },
                Span::new(SourceId::new(7), 35, 43),
            ),
        ] {
            let nested = HirExpr {
                kind: HirExprKind::Block(HirBlock {
                    statements: vec![statement],
                    tail: None,
                    ty: Type::Never,
                    span,
                }),
                ty: Type::Never,
                span,
            };
            let guarded = HirExpr {
                kind: HirExprKind::If {
                    condition: Box::new(HirExpr {
                        kind: HirExprKind::Literal(HirLiteral::Bool(true)),
                        ty: Type::Bool,
                        span,
                    }),
                    then_branch: HirBlock {
                        statements: Vec::new(),
                        tail: Some(Box::new(nested)),
                        ty: Type::Never,
                        span,
                    },
                    else_branch: None,
                },
                ty: Type::Unit,
                span,
            };
            let body = |tail: HirExpr| HirBlock {
                statements: Vec::new(),
                tail: Some(Box::new(tail)),
                ty: Type::Unit,
                span,
            };
            let program = HirProgram {
                functions: vec![
                    HirFunction {
                        id: FunctionId(0),
                        name: "main".to_owned(),
                        params: Vec::new(),
                        return_type: Type::Unit,
                        body: body(HirExpr {
                            kind: HirExprKind::Call {
                                callee: HirCallee::Function(FunctionId(1)),
                                args: Vec::new(),
                            },
                            ty: Type::Unit,
                            span,
                        }),
                        span,
                    },
                    HirFunction {
                        id: FunctionId(1),
                        name: "bad".to_owned(),
                        params: Vec::new(),
                        return_type: Type::Unit,
                        body: body(guarded),
                        span,
                    },
                    HirFunction {
                        id: FunctionId(2),
                        name: "good".to_owned(),
                        params: Vec::new(),
                        return_type: Type::Int,
                        body: body(literal_int(7, span)),
                        span,
                    },
                ],
            };

            let result = execute(&program);
            assert_eq!(result.diagnostics.len(), 1);
            assert_eq!(result.diagnostics[0].code, "E9004");
            assert_eq!(result.diagnostics[0].primary, keyword_span);

            let mut interpreter = Interpreter::new(&program);
            let error = interpreter
                .call_function(FunctionId(0), Vec::new(), span)
                .expect_err("nested loop control must not escape its callee");
            assert_eq!(error.code, "E9004");
            assert_eq!(error.primary, keyword_span);
            assert_eq!(interpreter.call_depth, 0);
            assert_eq!(
                interpreter.call_function(FunctionId(2), Vec::new(), span),
                Ok(Value::Int(7))
            );
            assert_eq!(interpreter.call_depth, 0);
        }
    }
}
