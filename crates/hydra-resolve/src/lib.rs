use hydra_ast::{Block, Expr, ExprKind, Program, Stmt};
use hydra_diagnostics::{Diagnostic, Phase};
use hydra_source::Span;
use hydra_stdlib::BuiltinId;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FunctionId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SymbolId(pub u32);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResolvedName {
    Local(SymbolId),
    Function(FunctionId),
    Builtin(BuiltinId),
}

#[derive(Clone, Debug)]
pub struct SymbolInfo {
    pub id: SymbolId,
    pub name: String,
    pub mutable: bool,
    pub declaration: Span,
}

#[derive(Clone, Debug, Default)]
pub struct Resolution {
    functions: BTreeMap<String, FunctionId>,
    references: BTreeMap<SpanKey, ResolvedName>,
    declarations: BTreeMap<SpanKey, SymbolInfo>,
}

impl Resolution {
    #[must_use]
    pub fn function(&self, name: &str) -> Option<FunctionId> {
        self.functions.get(name).copied()
    }

    #[must_use]
    pub fn resolved(&self, span: Span) -> Option<&ResolvedName> {
        self.references.get(&SpanKey::from(span))
    }

    #[must_use]
    pub fn declaration(&self, span: Span) -> Option<&SymbolInfo> {
        self.declarations.get(&SpanKey::from(span))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct SpanKey {
    source: u32,
    start: usize,
    end: usize,
}

impl From<Span> for SpanKey {
    fn from(span: Span) -> Self {
        Self {
            source: span.source.raw(),
            start: span.start,
            end: span.end,
        }
    }
}

#[derive(Debug)]
pub struct ResolveResult {
    pub resolution: Resolution,
    pub diagnostics: Vec<Diagnostic>,
}

#[must_use]
pub fn resolve(program: &Program) -> ResolveResult {
    Resolver::new().resolve(program)
}

struct Resolver {
    resolution: Resolution,
    diagnostics: Vec<Diagnostic>,
    scopes: Vec<BTreeMap<String, SymbolInfo>>,
    next_symbol: u32,
}

impl Resolver {
    fn new() -> Self {
        Self {
            resolution: Resolution::default(),
            diagnostics: Vec::new(),
            scopes: Vec::new(),
            next_symbol: 0,
        }
    }

    fn resolve(mut self, program: &Program) -> ResolveResult {
        for (index, function) in program.functions.iter().enumerate() {
            let Ok(raw_id) = u32::try_from(index) else {
                self.diagnostics.push(Diagnostic::error(
                    "E9001",
                    Phase::Internal,
                    "too many functions to assign stable IDs",
                    function.name_span,
                ));
                break;
            };
            if self.resolution.functions.contains_key(&function.name) {
                self.diagnostics.push(Diagnostic::error(
                    "E2001",
                    Phase::Resolution,
                    format!("duplicate function `{}`", function.name),
                    function.name_span,
                ));
            } else {
                self.resolution
                    .functions
                    .insert(function.name.clone(), FunctionId(raw_id));
            }
        }

        for function in &program.functions {
            self.scopes.clear();
            self.scopes.push(BTreeMap::new());
            for param in &function.params {
                self.declare(&param.name, param.name_span, false, "parameter");
            }
            self.resolve_block(&function.body, false);
        }

        ResolveResult {
            resolution: self.resolution,
            diagnostics: self.diagnostics,
        }
    }

    fn resolve_block(&mut self, block: &Block, create_scope: bool) {
        if create_scope {
            self.scopes.push(BTreeMap::new());
        }
        for statement in &block.statements {
            self.resolve_stmt(statement);
        }
        if let Some(tail) = &block.tail {
            self.resolve_expr(tail);
        }
        if create_scope {
            self.scopes.pop();
        }
    }

    fn resolve_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let {
                mutable,
                name,
                name_span,
                init,
                ..
            } => {
                self.resolve_expr(init);
                self.declare(name, *name_span, *mutable, "binding");
            }
            Stmt::Expr { expr, .. } => self.resolve_expr(expr),
            Stmt::While {
                condition, body, ..
            } => {
                self.resolve_expr(condition);
                self.resolve_block(body, true);
            }
            Stmt::Break { .. } | Stmt::Continue { .. } => {}
            Stmt::Return { value, .. } => {
                if let Some(value) = value {
                    self.resolve_expr(value);
                }
            }
        }
    }

    fn resolve_expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Literal(_) => {}
            ExprKind::Name(name) => {
                if let Some(resolved) = self.lookup(name) {
                    self.resolution
                        .references
                        .insert(SpanKey::from(expr.span), resolved);
                } else {
                    self.undefined(name, expr.span);
                }
            }
            ExprKind::Unary { operand, .. } => self.resolve_expr(operand),
            ExprKind::Binary { left, right, .. } => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            }
            ExprKind::Assign {
                name,
                name_span,
                value,
            } => {
                self.resolve_expr(value);
                match self.lookup_local(name) {
                    Some(info) if info.mutable => {
                        self.resolution
                            .references
                            .insert(SpanKey::from(*name_span), ResolvedName::Local(info.id));
                    }
                    Some(info) => {
                        self.diagnostics.push(
                            Diagnostic::error(
                                "E2005",
                                Phase::Resolution,
                                format!("cannot assign to immutable binding `{name}`"),
                                *name_span,
                            )
                            .with_label(info.declaration, "binding declared immutable here"),
                        );
                    }
                    None => self.undefined(name, *name_span),
                }
            }
            ExprKind::Call { callee, args } => {
                self.resolve_expr(callee);
                for arg in args {
                    self.resolve_expr(arg);
                }
            }
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.resolve_expr(condition);
                self.resolve_block(then_branch, true);
                if let Some(else_branch) = else_branch {
                    self.resolve_expr(else_branch);
                }
            }
            ExprKind::Block(block) => self.resolve_block(block, true),
        }
    }

    fn declare(&mut self, name: &str, span: Span, mutable: bool, kind: &str) {
        let Some(scope) = self.scopes.last_mut() else {
            self.diagnostics.push(Diagnostic::error(
                "E9002",
                Phase::Internal,
                "resolver scope stack is empty",
                span,
            ));
            return;
        };
        if let Some(previous) = scope.get(name) {
            self.diagnostics.push(
                Diagnostic::error(
                    "E2002",
                    Phase::Resolution,
                    format!("duplicate {kind} `{name}` in the same scope"),
                    span,
                )
                .with_label(previous.declaration, "previous declaration here"),
            );
            return;
        }
        let id = SymbolId(self.next_symbol);
        self.next_symbol = self.next_symbol.saturating_add(1);
        let info = SymbolInfo {
            id,
            name: name.to_owned(),
            mutable,
            declaration: span,
        };
        scope.insert(name.to_owned(), info.clone());
        self.resolution
            .declarations
            .insert(SpanKey::from(span), info);
    }

    fn lookup(&self, name: &str) -> Option<ResolvedName> {
        if let Some(info) = self.lookup_local(name) {
            return Some(ResolvedName::Local(info.id));
        }
        if let Some(id) = self.resolution.functions.get(name) {
            return Some(ResolvedName::Function(*id));
        }
        hydra_stdlib::lookup(name).map(|builtin| ResolvedName::Builtin(builtin.id))
    }

    fn lookup_local(&self, name: &str) -> Option<&SymbolInfo> {
        self.scopes.iter().rev().find_map(|scope| scope.get(name))
    }

    fn undefined(&mut self, name: &str, span: Span) {
        self.diagnostics.push(Diagnostic::error(
            "E2004",
            Phase::Resolution,
            format!("undefined name `{name}`"),
            span,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::resolve;
    use hydra_lexer::lex;
    use hydra_parser::parse;
    use hydra_source::SourceId;

    #[test]
    fn rejects_immutable_assignment() {
        let source = SourceId::new(0);
        let lexed = lex(source, "fn main() {\n let x = 1\n x = 2\n}\n");
        let parsed = parse(&lexed.tokens);
        let result = resolve(&parsed.program);
        assert!(result.diagnostics.iter().any(|d| d.code == "E2005"));
    }

    #[test]
    fn function_and_nested_scope_state_do_not_leak() {
        let source = SourceId::new(0);
        let lexed = lex(
            source,
            "fn broken() {\n let x = 1\n let x = 2\n}\n\
             fn valid() {\n let mut x = 1\n {\n  let mut x = 2\n  x = 3\n }\n x = 4\n}\n",
        );
        assert!(lexed.diagnostics.is_empty(), "{:?}", lexed.diagnostics);
        let parsed = parse(&lexed.tokens);
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);

        let result = resolve(&parsed.program);
        let codes: Vec<_> = result.diagnostics.iter().map(|d| d.code).collect();
        assert_eq!(codes, vec!["E2002"]);
        assert!(result.resolution.function("valid").is_some());
    }
}
