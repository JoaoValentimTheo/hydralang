use hydra_ast::{
    BinaryOp, Block, Expr, ExprKind, Function, Literal, Param, Program, Stmt, TypeExpr,
    TypeExprKind, UnaryOp,
};
use hydra_diagnostics::{Diagnostic, Phase};
use hydra_lexer::{Token, TokenKind};
use hydra_source::{SourceId, Span};
use std::mem::discriminant;

const MAX_PARSE_DEPTH: usize = 128;
const MAX_EXPR_DEPTH: usize = 256;
const MAX_TUPLE_ARITY: usize = 64;
const MAX_TUPLE_TYPE_NESTING: usize = 64;
const MAX_LIST_ELEMENTS: usize = 256;

#[derive(Debug)]
pub struct ParseResult {
    pub program: Program,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn parse(tokens: &[Token]) -> ParseResult {
    Parser::new(tokens).parse_program()
}

struct Parser<'a> {
    tokens: &'a [Token],
    current: usize,
    depth: usize,
    synthetic_eof: Token,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        let synthetic_eof = tokens.last().map_or_else(
            || Token {
                kind: TokenKind::Eof,
                span: Span::empty(SourceId::new(0), 0),
            },
            |token| Token {
                kind: TokenKind::Eof,
                span: Span::empty(token.span.source, token.span.end),
            },
        );
        Self {
            tokens,
            current: 0,
            depth: 0,
            synthetic_eof,
            diagnostics: Vec::new(),
        }
    }

    fn parse_program(mut self) -> ParseResult {
        let mut functions = Vec::new();
        self.skip_newlines();
        while !self.at(&TokenKind::Eof) {
            let before = self.current;
            if self.at(&TokenKind::Fn) {
                if let Some(function) = self.parse_function() {
                    functions.push(function);
                }
            } else {
                let span = self.peek().span;
                self.error("E1101", "expected a function declaration", span);
                self.synchronize_top_level();
            }
            if self.current == before {
                self.advance();
            }
            self.skip_newlines();
        }
        ParseResult {
            program: Program { functions },
            diagnostics: self.diagnostics,
        }
    }

    fn parse_function(&mut self) -> Option<Function> {
        let start = self.expect(&TokenKind::Fn, "expected `fn`")?.span;
        let (name, name_span) = self.expect_identifier("expected function name")?;
        self.expect(&TokenKind::LeftParen, "expected `(` after function name")?;
        self.skip_newlines();
        let mut params = Vec::new();
        if !self.at(&TokenKind::RightParen) {
            loop {
                let (param_name, param_name_span) =
                    self.expect_identifier("expected parameter name")?;
                self.expect(&TokenKind::Colon, "expected `:` after parameter name")?;
                let ty = self.parse_type()?;
                let span = param_name_span.join(ty.span).unwrap_or(param_name_span);
                params.push(Param {
                    name: param_name,
                    name_span: param_name_span,
                    ty,
                    span,
                });
                self.skip_newlines();
                if !self.consume(&TokenKind::Comma) {
                    break;
                }
                self.skip_newlines();
            }
        }
        self.expect(&TokenKind::RightParen, "expected `)` after parameters")?;
        let return_type = if self.consume(&TokenKind::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };
        self.skip_newlines();
        let body = self.parse_block()?;
        let span = start.join(body.span).unwrap_or(start);
        Some(Function {
            name,
            name_span,
            params,
            return_type,
            body,
            span,
        })
    }

    fn parse_type(&mut self) -> Option<TypeExpr> {
        let ty = self.with_depth(|parser| parser.parse_type_inner())?;
        let mut pending = vec![(&ty, 0_usize)];
        while let Some((node, depth)) = pending.pop() {
            let next = match &node.kind {
                TypeExprKind::ListApplication(args) => {
                    pending.extend(args.iter().map(|arg| (arg, depth + 1)));
                    depth + 1
                }
                TypeExprKind::Tuple(fields) => {
                    pending.extend(fields.iter().map(|field| (field, depth + 1)));
                    depth + 1
                }
                _ => depth,
            };
            if next > MAX_TUPLE_TYPE_NESTING {
                self.error("E1105", "aggregate type nesting exceeds 64", node.span);
                return None;
            }
        }
        Some(ty)
    }

    fn parse_type_inner(&mut self) -> Option<TypeExpr> {
        if !self.at(&TokenKind::LeftParen) {
            let (name, span) = self.expect_identifier("expected type name")?;
            if name == "List" && self.consume(&TokenKind::Less) {
                self.skip_newlines();
                let mut arguments = Vec::new();
                if !self.at(&TokenKind::Greater) {
                    loop {
                        arguments.push(self.parse_type()?);
                        self.skip_newlines();
                        if !self.consume(&TokenKind::Comma) {
                            break;
                        }
                        self.skip_newlines();
                        if self.at(&TokenKind::Greater) {
                            break;
                        }
                    }
                }
                let end = self
                    .expect(&TokenKind::Greater, "expected `>` after List type")?
                    .span;
                return Some(TypeExpr {
                    kind: TypeExprKind::ListApplication(arguments),
                    span: span.join(end).unwrap_or(span),
                });
            }
            return Some(TypeExpr {
                kind: TypeExprKind::Name(name),
                span,
            });
        }
        let open = self.advance().span;
        self.skip_newlines();
        if self.at(&TokenKind::RightParen) {
            let close = self.advance().span;
            return Some(TypeExpr {
                kind: TypeExprKind::Unit,
                span: open.join(close).unwrap_or(open),
            });
        }
        let mut first = self.parse_type()?;
        self.skip_newlines();
        if !self.consume(&TokenKind::Comma) {
            let close = self
                .expect(&TokenKind::RightParen, "expected `)` after type")?
                .span;
            first.span = open.join(close).unwrap_or(open);
            return Some(first);
        }
        let mut elements = vec![first];
        loop {
            self.skip_newlines();
            if self.at(&TokenKind::RightParen) {
                break;
            }
            if elements.len() == MAX_TUPLE_ARITY {
                self.error("E1101", "tuple type arity exceeds 64", self.peek().span);
                return None;
            }
            elements.push(self.parse_type()?);
            self.skip_newlines();
            if !self.consume(&TokenKind::Comma) {
                break;
            }
        }
        let close = self
            .expect(&TokenKind::RightParen, "expected `)` after tuple type")?
            .span;
        let span = open.join(close).unwrap_or(open);
        Some(TypeExpr {
            kind: TypeExprKind::Tuple(elements),
            span,
        })
    }

    fn parse_block(&mut self) -> Option<Block> {
        self.with_depth(Self::parse_block_inner)
    }

    fn parse_block_inner(&mut self) -> Option<Block> {
        let open = self.expect(&TokenKind::LeftBrace, "expected `{`")?.span;
        self.skip_newlines();
        let mut statements = Vec::new();
        let mut tail = None;
        while !self.at(&TokenKind::RightBrace) && !self.at(&TokenKind::Eof) {
            let before = self.current;
            if self.at(&TokenKind::Let) {
                if let Some(stmt) = self.parse_let() {
                    statements.push(stmt);
                }
            } else if self.at(&TokenKind::While) {
                if let Some(stmt) = self.parse_while() {
                    statements.push(stmt);
                }
            } else if self.at(&TokenKind::Break) || self.at(&TokenKind::Continue) {
                let keyword = self.advance().clone();
                self.require_line_boundary();
                statements.push(match keyword.kind {
                    TokenKind::Break => Stmt::Break { span: keyword.span },
                    _ => Stmt::Continue { span: keyword.span },
                });
            } else if self.at(&TokenKind::Return) {
                if let Some(stmt) = self.parse_return() {
                    statements.push(stmt);
                }
            } else if let Some(expr) = self.parse_expr(0) {
                let expr_span = expr.span;
                let had_newline = self.consume_newlines();
                if self.at(&TokenKind::RightBrace) {
                    tail = Some(Box::new(expr));
                    break;
                }
                if had_newline {
                    statements.push(Stmt::Expr {
                        expr,
                        span: expr_span,
                    });
                } else {
                    self.error(
                        "E1101",
                        "expected a newline or `}` after expression",
                        self.peek().span,
                    );
                    self.synchronize_block();
                }
            } else {
                self.synchronize_block();
            }
            if self.current == before {
                self.advance();
            }
            self.skip_newlines();
        }
        let close = self
            .expect(&TokenKind::RightBrace, "expected `}` to close block")?
            .span;
        Some(Block {
            statements,
            tail,
            span: open.join(close).unwrap_or(open),
        })
    }

    fn parse_let(&mut self) -> Option<Stmt> {
        let start = self.expect(&TokenKind::Let, "expected `let`")?.span;
        let mutable = self.consume(&TokenKind::Mut);
        let (name, name_span) = self.expect_identifier("expected binding name")?;
        let ty = if self.consume(&TokenKind::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };
        self.expect(&TokenKind::Equal, "expected `=` in binding")?;
        let init = self.parse_expr(0)?;
        let span = start.join(init.span).unwrap_or(start);
        self.require_line_boundary();
        Some(Stmt::Let {
            mutable,
            name,
            name_span,
            ty,
            init,
            span,
        })
    }

    fn parse_while(&mut self) -> Option<Stmt> {
        let start = self.expect(&TokenKind::While, "expected `while`")?.span;
        let condition = self.parse_expr(0)?;
        self.skip_newlines();
        let body = self.parse_block()?;
        let span = start.join(body.span).unwrap_or(start);
        self.consume_newlines();
        Some(Stmt::While {
            condition,
            body,
            span,
        })
    }

    fn parse_return(&mut self) -> Option<Stmt> {
        let start = self.expect(&TokenKind::Return, "expected `return`")?.span;
        let value = if self.at(&TokenKind::Newline) || self.at(&TokenKind::RightBrace) {
            None
        } else {
            self.parse_expr(0)
        };
        let end = value.as_ref().map_or(start, |expr| expr.span);
        self.require_line_boundary();
        Some(Stmt::Return {
            value,
            span: start.join(end).unwrap_or(start),
        })
    }

    fn parse_expr(&mut self, min_binding_power: u8) -> Option<Expr> {
        self.with_depth(|parser| parser.parse_expr_inner(min_binding_power))
    }

    fn parse_expr_inner(&mut self, min_binding_power: u8) -> Option<Expr> {
        let mut left = self.parse_prefix()?;
        if !self.expression_depth_within_limit(&left) {
            return None;
        }
        loop {
            if self.at(&TokenKind::LeftParen) {
                let call_power = 21;
                if call_power < min_binding_power {
                    break;
                }
                left = self.finish_call(left)?;
                if !self.expression_depth_within_limit(&left) {
                    return None;
                }
                continue;
            }

            if self.at(&TokenKind::Dot) {
                if 21 < min_binding_power {
                    break;
                }
                let dot = self.advance().span;
                let token = self.advance().clone();
                let TokenKind::Int(text) = token.kind else {
                    self.error(
                        "E1101",
                        "expected decimal tuple projection index",
                        dot.join(token.span).unwrap_or(dot),
                    );
                    return None;
                };
                let index_span = dot.join(token.span).unwrap_or(dot);
                let Ok(index) = text.parse::<usize>() else {
                    self.error("E1102", "tuple projection index is too large", index_span);
                    return None;
                };
                let span = left.span.join(token.span).unwrap_or(left.span);
                left = Expr {
                    kind: ExprKind::Projection {
                        base: Box::new(left),
                        index,
                        index_span,
                    },
                    span,
                };
                if !self.expression_depth_within_limit(&left) {
                    return None;
                }
                continue;
            }

            if self.at(&TokenKind::LeftBracket) {
                if 21 < min_binding_power {
                    break;
                }
                self.advance();
                let index = self.parse_expr(0)?;
                self.skip_newlines();
                let close = self
                    .expect(&TokenKind::RightBracket, "expected `]` after index")?
                    .span;
                let span = left.span.join(close).unwrap_or(left.span);
                left = Expr {
                    kind: ExprKind::Index {
                        base: Box::new(left),
                        index: Box::new(index),
                    },
                    span,
                };
                if !self.expression_depth_within_limit(&left) {
                    return None;
                }
                continue;
            }

            if self.at(&TokenKind::Equal) {
                let (left_power, right_power) = (1, 1);
                if left_power < min_binding_power {
                    break;
                }
                let equal_span = self.advance().span;
                let value = self.parse_expr(right_power)?;
                let ExprKind::Name(name) = left.kind else {
                    self.error("E1104", "assignment target must be a name", equal_span);
                    return Some(left);
                };
                let span = left.span.join(value.span).unwrap_or(left.span);
                left = Expr {
                    kind: ExprKind::Assign {
                        name,
                        name_span: left.span,
                        value: Box::new(value),
                    },
                    span,
                };
                if !self.expression_depth_within_limit(&left) {
                    return None;
                }
                continue;
            }

            let Some((op, left_power, right_power)) = self.binary_operator() else {
                break;
            };
            if left_power < min_binding_power {
                break;
            }
            self.advance();
            while self.at(&TokenKind::Newline) {
                self.advance();
            }
            let right = self.parse_expr(right_power)?;
            let span = left.span.join(right.span).unwrap_or(left.span);
            left = Expr {
                kind: ExprKind::Binary {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                },
                span,
            };
            if !self.expression_depth_within_limit(&left) {
                return None;
            }
        }
        Some(left)
    }

    fn expression_depth_within_limit(&mut self, expr: &Expr) -> bool {
        let mut stack = vec![(expr, 1_usize)];
        while let Some((expr, depth)) = stack.pop() {
            if depth > MAX_EXPR_DEPTH {
                self.error(
                    "E1106",
                    "maximum expression tree depth of 256 exceeded",
                    expr.span,
                );
                return false;
            }
            let child_depth = depth + 1;
            match &expr.kind {
                ExprKind::Unary { operand, .. } => stack.push((operand, child_depth)),
                ExprKind::Tuple(fields) | ExprKind::List(fields) => {
                    stack.extend(fields.iter().map(|e| (e, child_depth)));
                }
                ExprKind::Index { base, index } => {
                    stack.push((base, child_depth));
                    stack.push((index, child_depth));
                }
                ExprKind::Projection { base, .. } => stack.push((base, child_depth)),
                ExprKind::Binary { left, right, .. } => {
                    stack.push((left, child_depth));
                    stack.push((right, child_depth));
                }
                ExprKind::Assign { value, .. } => stack.push((value, child_depth)),
                ExprKind::Call { callee, args } => {
                    stack.push((callee, child_depth));
                    stack.extend(args.iter().map(|arg| (arg, child_depth)));
                }
                ExprKind::If {
                    condition,
                    else_branch,
                    ..
                } => {
                    stack.push((condition, child_depth));
                    if let Some(else_branch) = else_branch {
                        stack.push((else_branch, child_depth));
                    }
                }
                ExprKind::Literal(_) | ExprKind::Name(_) | ExprKind::Block(_) => {}
            }
        }
        true
    }

    fn parse_prefix(&mut self) -> Option<Expr> {
        let token = self.advance().clone();
        match token.kind {
            TokenKind::Int(text) => match text.parse::<i64>() {
                Ok(value) => Some(Expr {
                    kind: ExprKind::Literal(Literal::Int(value)),
                    span: token.span,
                }),
                Err(_) => {
                    self.error(
                        "E1102",
                        "integer literal is outside the Int range",
                        token.span,
                    );
                    None
                }
            },
            TokenKind::Float(text) => match text.parse::<f64>() {
                Ok(value) => Some(Expr {
                    kind: ExprKind::Literal(Literal::Float(value)),
                    span: token.span,
                }),
                Err(_) => {
                    self.error("E1103", "invalid floating-point literal", token.span);
                    None
                }
            },
            TokenKind::String(value) => Some(Expr {
                kind: ExprKind::Literal(Literal::String(value)),
                span: token.span,
            }),
            TokenKind::True => Some(Expr {
                kind: ExprKind::Literal(Literal::Bool(true)),
                span: token.span,
            }),
            TokenKind::False => Some(Expr {
                kind: ExprKind::Literal(Literal::Bool(false)),
                span: token.span,
            }),
            TokenKind::Identifier(name) => Some(Expr {
                kind: ExprKind::Name(name),
                span: token.span,
            }),
            TokenKind::Minus | TokenKind::Bang => {
                let op = if matches!(token.kind, TokenKind::Minus) {
                    UnaryOp::Negate
                } else {
                    UnaryOp::Not
                };
                let operand = self.parse_expr(19)?;
                let span = token.span.join(operand.span).unwrap_or(token.span);
                Some(Expr {
                    kind: ExprKind::Unary {
                        op,
                        operand: Box::new(operand),
                    },
                    span,
                })
            }
            TokenKind::LeftParen => {
                self.skip_newlines();
                if self.at(&TokenKind::RightParen) {
                    let close = self.advance().span;
                    return Some(Expr {
                        kind: ExprKind::Literal(Literal::Unit),
                        span: token.span.join(close).unwrap_or(token.span),
                    });
                }
                let mut expr = self.parse_expr(0)?;
                self.skip_newlines();
                if self.consume(&TokenKind::Comma) {
                    let mut elements = vec![expr];
                    loop {
                        self.skip_newlines();
                        if self.at(&TokenKind::RightParen) {
                            break;
                        }
                        if elements.len() == MAX_TUPLE_ARITY {
                            self.error("E1101", "tuple arity exceeds 64", self.peek().span);
                            return None;
                        }
                        elements.push(self.parse_expr(0)?);
                        self.skip_newlines();
                        if !self.consume(&TokenKind::Comma) {
                            break;
                        }
                    }
                    let close = self
                        .expect(&TokenKind::RightParen, "expected `)` after tuple")?
                        .span;
                    return Some(Expr {
                        kind: ExprKind::Tuple(elements),
                        span: token.span.join(close).unwrap_or(token.span),
                    });
                }
                let close = self.expect(&TokenKind::RightParen, "expected `)`")?.span;
                expr.span = token.span.join(close).unwrap_or(expr.span);
                Some(expr)
            }
            TokenKind::LeftBracket => {
                self.skip_newlines();
                let mut elements = Vec::new();
                if !self.at(&TokenKind::RightBracket) {
                    loop {
                        if elements.len() == MAX_LIST_ELEMENTS {
                            self.error(
                                "E1101",
                                "List literal exceeds 256 elements",
                                self.peek().span,
                            );
                            return None;
                        }
                        elements.push(self.parse_expr(0)?);
                        self.skip_newlines();
                        if !self.consume(&TokenKind::Comma) {
                            break;
                        }
                        self.skip_newlines();
                        if self.at(&TokenKind::RightBracket) {
                            break;
                        }
                    }
                }
                let close = self
                    .expect(&TokenKind::RightBracket, "expected `]` after List")?
                    .span;
                Some(Expr {
                    kind: ExprKind::List(elements),
                    span: token.span.join(close).unwrap_or(token.span),
                })
            }
            TokenKind::If => self.parse_if(token.span),
            TokenKind::LeftBrace => {
                self.current = self.current.saturating_sub(1);
                let block = self.parse_block()?;
                let span = block.span;
                Some(Expr {
                    kind: ExprKind::Block(block),
                    span,
                })
            }
            _ => {
                self.error("E1101", "expected expression", token.span);
                None
            }
        }
    }

    fn parse_if(&mut self, start: Span) -> Option<Expr> {
        let condition = self.parse_expr(0)?;
        self.skip_newlines();
        let then_branch = self.parse_block()?;
        let mut end = then_branch.span;
        let checkpoint = self.current;
        self.skip_newlines();
        let else_branch = if self.consume(&TokenKind::Else) {
            self.skip_newlines();
            let branch = if self.at(&TokenKind::If) {
                let if_span = self.advance().span;
                self.parse_if(if_span)?
            } else if self.at(&TokenKind::LeftBrace) {
                let block = self.parse_block()?;
                let span = block.span;
                Expr {
                    kind: ExprKind::Block(block),
                    span,
                }
            } else {
                self.error(
                    "E1101",
                    "expected `if` or block after `else`",
                    self.peek().span,
                );
                return None;
            };
            end = branch.span;
            Some(Box::new(branch))
        } else {
            self.current = checkpoint;
            None
        };
        Some(Expr {
            kind: ExprKind::If {
                condition: Box::new(condition),
                then_branch,
                else_branch,
            },
            span: start.join(end).unwrap_or(start),
        })
    }

    fn finish_call(&mut self, callee: Expr) -> Option<Expr> {
        let open = self.expect(&TokenKind::LeftParen, "expected `(`")?.span;
        self.skip_newlines();
        let mut args = Vec::new();
        if !self.at(&TokenKind::RightParen) {
            loop {
                args.push(self.parse_expr(0)?);
                self.skip_newlines();
                if !self.consume(&TokenKind::Comma) {
                    break;
                }
                self.skip_newlines();
            }
        }
        let close = self
            .expect(&TokenKind::RightParen, "expected `)` after arguments")?
            .span;
        let span = callee
            .span
            .join(close)
            .or_else(|| open.join(close))
            .unwrap_or(callee.span);
        Some(Expr {
            kind: ExprKind::Call {
                callee: Box::new(callee),
                args,
            },
            span,
        })
    }

    fn binary_operator(&self) -> Option<(BinaryOp, u8, u8)> {
        match self.peek().kind {
            TokenKind::OrOr => Some((BinaryOp::Or, 3, 4)),
            TokenKind::AndAnd => Some((BinaryOp::And, 5, 6)),
            TokenKind::EqualEqual => Some((BinaryOp::Equal, 7, 8)),
            TokenKind::BangEqual => Some((BinaryOp::NotEqual, 7, 8)),
            TokenKind::Less => Some((BinaryOp::Less, 9, 10)),
            TokenKind::LessEqual => Some((BinaryOp::LessEqual, 9, 10)),
            TokenKind::Greater => Some((BinaryOp::Greater, 9, 10)),
            TokenKind::GreaterEqual => Some((BinaryOp::GreaterEqual, 9, 10)),
            TokenKind::Plus => Some((BinaryOp::Add, 11, 12)),
            TokenKind::Minus => Some((BinaryOp::Subtract, 11, 12)),
            TokenKind::Star => Some((BinaryOp::Multiply, 13, 14)),
            TokenKind::Slash => Some((BinaryOp::Divide, 13, 14)),
            TokenKind::Percent => Some((BinaryOp::Remainder, 13, 14)),
            _ => None,
        }
    }

    fn require_line_boundary(&mut self) {
        if self.at(&TokenKind::RightBrace) || self.at(&TokenKind::Eof) {
            return;
        }
        if !self.consume_newlines() {
            self.error(
                "E1101",
                "expected a newline after statement",
                self.peek().span,
            );
            self.synchronize_block();
        }
    }

    fn synchronize_block(&mut self) {
        while !self.at(&TokenKind::Newline)
            && !self.at(&TokenKind::RightBrace)
            && !self.at(&TokenKind::Eof)
        {
            self.advance();
        }
        self.consume_newlines();
    }

    fn synchronize_top_level(&mut self) {
        while !self.at(&TokenKind::Eof) {
            if self.at(&TokenKind::Fn) {
                return;
            }
            self.advance();
        }
    }

    fn expect_identifier(&mut self, message: &'static str) -> Option<(String, Span)> {
        let token = self.advance().clone();
        if let TokenKind::Identifier(name) = token.kind {
            Some((name, token.span))
        } else {
            self.error("E1101", message, token.span);
            None
        }
    }

    fn expect(&mut self, expected: &TokenKind, message: &'static str) -> Option<&Token> {
        if self.at(expected) {
            let index = self.current;
            self.advance();
            self.tokens.get(index)
        } else {
            self.error("E1101", message, self.peek().span);
            None
        }
    }

    fn consume(&mut self, expected: &TokenKind) -> bool {
        if self.at(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn consume_newlines(&mut self) -> bool {
        let start = self.current;
        self.skip_newlines();
        self.current != start
    }

    fn skip_newlines(&mut self) {
        while self.at(&TokenKind::Newline) {
            self.advance();
        }
    }

    fn with_depth<T>(&mut self, parse: impl FnOnce(&mut Self) -> Option<T>) -> Option<T> {
        if self.depth >= MAX_PARSE_DEPTH {
            let span = self.peek().span;
            self.error(
                "E1105",
                "maximum syntax nesting depth of 128 exceeded",
                span,
            );
            return None;
        }
        self.depth += 1;
        let result = parse(self);
        self.depth = self.depth.saturating_sub(1);
        result
    }

    fn at(&self, expected: &TokenKind) -> bool {
        discriminant(&self.peek().kind) == discriminant(expected)
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.current).unwrap_or(&self.synthetic_eof)
    }

    fn advance(&mut self) -> &Token {
        if self.current < self.tokens.len() {
            let index = self.current;
            self.current += 1;
            &self.tokens[index]
        } else {
            &self.synthetic_eof
        }
    }

    fn error(&mut self, code: &'static str, message: impl Into<String>, span: Span) {
        self.diagnostics
            .push(Diagnostic::error(code, Phase::Parser, message, span));
    }
}

#[cfg(test)]
mod tests {
    use super::parse;
    use hydra_lexer::{Token, TokenKind, lex};
    use hydra_source::{SourceId, Span};

    #[test]
    fn parses_function_and_tail_expression() {
        let source = SourceId::new(0);
        let lexed = lex(source, "fn add(a: Int, b: Int) -> Int {\n a + b\n}\n");
        assert!(lexed.diagnostics.is_empty());
        let parsed = parse(&lexed.tokens);
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        assert_eq!(parsed.program.functions.len(), 1);
        assert!(parsed.program.functions[0].body.tail.is_some());
    }

    #[test]
    fn loop_control_statements_preserve_keyword_spans_and_recover_after_bad_operand() {
        use hydra_ast::Stmt;
        let source = SourceId::new(0);
        let text = "fn main() {\n while true {\n break\n continue\n }\n}\n";
        let lexed = lex(source, text);
        let parsed = parse(&lexed.tokens);
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let statements = &parsed.program.functions[0].body.statements;
        let Stmt::While { body, .. } = &statements[0] else {
            panic!("missing while");
        };
        assert!(
            matches!(&body.statements[0], Stmt::Break { span } if &text[span.start..span.end] == "break")
        );
        assert!(
            matches!(&body.statements[1], Stmt::Continue { span } if &text[span.start..span.end] == "continue")
        );

        let text = "fn main() {\n break(7)\n continue 3\n}\nfn valid() {}\n";
        let parsed = parse(&lex(source, text).tokens);
        assert!(parsed.diagnostics.iter().any(|d| d.code == "E1101"));
        assert_eq!(
            parsed.program.functions.len(),
            2,
            "parser should retain later functions"
        );
    }

    #[test]
    fn parser_recovers_from_bad_top_level_input() {
        let source = SourceId::new(0);
        let lexed = lex(source, "???\nfn main() {}\n");
        let parsed = parse(&lexed.tokens);
        assert!(!parsed.diagnostics.is_empty() || !lexed.diagnostics.is_empty());
        assert_eq!(parsed.program.functions.len(), 1);
    }

    #[test]
    fn parser_tolerates_missing_eof_token() {
        let source = SourceId::new(0);
        let tokens = [Token {
            kind: TokenKind::Fn,
            span: Span::new(source, 0, 2),
        }];
        let parsed = parse(&tokens);
        assert!(!parsed.diagnostics.is_empty());
    }

    #[test]
    fn malformed_input_without_physical_eof_has_bounded_recovery() {
        let source = SourceId::new(0);
        let tokens: Vec<_> = (0..512)
            .map(|offset| Token {
                kind: TokenKind::Identifier("junk".to_owned()),
                span: Span::new(source, offset, offset + 1),
            })
            .collect();
        let parsed = parse(&tokens);
        assert_eq!(parsed.diagnostics.len(), 1);
        assert!(parsed.program.functions.is_empty());
    }

    #[test]
    fn rejects_pathological_left_deep_expression_before_host_stack_exhaustion() {
        let source = SourceId::new(0);
        let mut text = String::from("fn main() -> Int {\n1");
        for _ in 0..1_024 {
            text.push_str(" + 1");
        }
        text.push_str("\n}\n");

        let lexed = lex(source, &text);
        assert!(lexed.diagnostics.is_empty(), "{:?}", lexed.diagnostics);
        let parsed = parse(&lexed.tokens);
        assert!(parsed.diagnostics.iter().any(|d| d.code == "E1106"));
    }

    #[test]
    fn parser_rejects_pathological_nesting_before_host_stack_exhaustion() {
        let source = SourceId::new(0);
        let nested = "(".repeat(256) + "1" + &")".repeat(256);
        let text = format!("fn main() {{\n{nested}\n}}\n");
        let lexed = lex(source, &text);
        assert!(lexed.diagnostics.is_empty());
        let parsed = parse(&lexed.tokens);
        assert!(
            parsed
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "E1105"),
            "{:?}",
            parsed.diagnostics
        );
    }

    #[test]
    fn parser_reports_invalid_float_token() {
        let source = SourceId::new(0);
        let tokens = [
            Token {
                kind: TokenKind::Fn,
                span: Span::new(source, 0, 2),
            },
            Token {
                kind: TokenKind::Identifier("main".to_owned()),
                span: Span::new(source, 3, 7),
            },
            Token {
                kind: TokenKind::LeftParen,
                span: Span::new(source, 7, 8),
            },
            Token {
                kind: TokenKind::RightParen,
                span: Span::new(source, 8, 9),
            },
            Token {
                kind: TokenKind::LeftBrace,
                span: Span::new(source, 10, 11),
            },
            Token {
                kind: TokenKind::Float("not-a-float".to_owned()),
                span: Span::new(source, 12, 23),
            },
            Token {
                kind: TokenKind::RightBrace,
                span: Span::new(source, 24, 25),
            },
            Token {
                kind: TokenKind::Eof,
                span: Span::empty(source, 25),
            },
        ];
        let parsed = parse(&tokens);
        assert!(
            parsed
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "E1103"),
            "{:?}",
            parsed.diagnostics
        );
    }

    #[test]
    fn unit_literal_span_includes_both_parentheses() {
        let source = SourceId::new(0);
        let text = "fn main() {\n ()\n}\n";
        let unit_start = text.rfind("()").expect("test source contains unit literal");
        let lexed = lex(source, text);
        assert!(lexed.diagnostics.is_empty(), "{:?}", lexed.diagnostics);
        let parsed = parse(&lexed.tokens);
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let tail = parsed.program.functions[0]
            .body
            .tail
            .as_deref()
            .expect("unit literal should be the block tail");
        assert_eq!(tail.span, Span::new(source, unit_start, unit_start + 2));
    }
}
