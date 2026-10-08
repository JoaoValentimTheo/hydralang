use hydra_diagnostics::{Diagnostic, Phase};
use hydra_source::{SourceId, Span};

#[derive(Clone, Debug, PartialEq)]
pub enum TokenKind {
    Identifier(String),
    Int(String),
    Float(String),
    String(String),
    Fn,
    Let,
    Mut,
    If,
    Else,
    While,
    Return,
    Break,
    Continue,
    True,
    False,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Colon,
    Dot,
    Arrow,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Equal,
    EqualEqual,
    Bang,
    BangEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    AndAnd,
    OrOr,
    Newline,
    Eof,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug)]
pub struct LexResult {
    pub tokens: Vec<Token>,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn lex(source: SourceId, text: &str) -> LexResult {
    Lexer::new(source, text).run()
}

struct Lexer<'a> {
    source: SourceId,
    text: &'a str,
    offset: usize,
    tokens: Vec<Token>,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> Lexer<'a> {
    fn new(source: SourceId, text: &'a str) -> Self {
        Self {
            source,
            text,
            offset: 0,
            tokens: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    fn run(mut self) -> LexResult {
        while self.offset < self.text.len() {
            let start = self.offset;
            let Some(ch) = self.peek_char() else {
                break;
            };
            match ch {
                ' ' | '\t' | '\r' => {
                    self.bump();
                }
                '\n' => {
                    self.bump();
                    self.push(TokenKind::Newline, start, self.offset);
                }
                '/' if self.peek_second_char() == Some('/') => self.line_comment(),
                '"' => self.string(start),
                '0'..='9' => self.number(start),
                c if is_ident_start(c) => self.identifier(start),
                '(' => self.single(TokenKind::LeftParen, start),
                ')' => self.single(TokenKind::RightParen, start),
                '{' => self.single(TokenKind::LeftBrace, start),
                '}' => self.single(TokenKind::RightBrace, start),
                '[' => self.single(TokenKind::LeftBracket, start),
                ']' => self.single(TokenKind::RightBracket, start),
                ',' => self.single(TokenKind::Comma, start),
                ':' => self.single(TokenKind::Colon, start),
                '.' => self.single(TokenKind::Dot, start),
                '+' => self.single(TokenKind::Plus, start),
                '*' => self.single(TokenKind::Star, start),
                '%' => self.single(TokenKind::Percent, start),
                '-' => {
                    self.bump();
                    if self.consume_if('>') {
                        self.push(TokenKind::Arrow, start, self.offset);
                    } else {
                        self.push(TokenKind::Minus, start, self.offset);
                    }
                }
                '/' => self.single(TokenKind::Slash, start),
                '=' => {
                    self.bump();
                    let kind = if self.consume_if('=') {
                        TokenKind::EqualEqual
                    } else {
                        TokenKind::Equal
                    };
                    self.push(kind, start, self.offset);
                }
                '!' => {
                    self.bump();
                    let kind = if self.consume_if('=') {
                        TokenKind::BangEqual
                    } else {
                        TokenKind::Bang
                    };
                    self.push(kind, start, self.offset);
                }
                '<' => {
                    self.bump();
                    let kind = if self.consume_if('=') {
                        TokenKind::LessEqual
                    } else {
                        TokenKind::Less
                    };
                    self.push(kind, start, self.offset);
                }
                '>' => {
                    self.bump();
                    let kind = if self.consume_if('=') {
                        TokenKind::GreaterEqual
                    } else {
                        TokenKind::Greater
                    };
                    self.push(kind, start, self.offset);
                }
                '&' => {
                    self.bump();
                    if self.consume_if('&') {
                        self.push(TokenKind::AndAnd, start, self.offset);
                    } else {
                        self.unexpected(start, "expected `&&`; single `&` is not an operator");
                    }
                }
                '|' => {
                    self.bump();
                    if self.consume_if('|') {
                        self.push(TokenKind::OrOr, start, self.offset);
                    } else {
                        self.unexpected(start, "expected `||`; single `|` is not an operator");
                    }
                }
                _ => {
                    self.bump();
                    self.unexpected(start, format!("unexpected character `{ch}`"));
                }
            }
        }
        self.tokens.push(Token {
            kind: TokenKind::Eof,
            span: Span::empty(self.source, self.text.len()),
        });
        LexResult {
            tokens: self.tokens,
            diagnostics: self.diagnostics,
        }
    }

    fn line_comment(&mut self) {
        self.bump();
        self.bump();
        while let Some(ch) = self.peek_char() {
            if ch == '\n' {
                break;
            }
            self.bump();
        }
    }

    fn string(&mut self, start: usize) {
        self.bump();
        let mut value = String::new();
        let mut terminated = false;
        while let Some(ch) = self.peek_char() {
            if ch == '"' {
                self.bump();
                terminated = true;
                break;
            }
            if ch == '\n' {
                break;
            }
            if ch == '\\' {
                self.bump();
                let Some(escaped) = self.peek_char() else {
                    break;
                };
                self.bump();
                match escaped {
                    'n' => value.push('\n'),
                    'r' => value.push('\r'),
                    't' => value.push('\t'),
                    '"' => value.push('"'),
                    '\\' => value.push('\\'),
                    other => {
                        let span =
                            Span::new(self.source, self.offset - other.len_utf8(), self.offset);
                        self.diagnostics.push(
                            Diagnostic::error(
                                "E1003",
                                Phase::Lexer,
                                format!("unknown string escape `\\{other}`"),
                                span,
                            )
                            .with_help(r#"supported escapes are \n, \r, \t, \", and \\"#),
                        );
                        value.push(other);
                    }
                }
            } else {
                self.bump();
                value.push(ch);
            }
        }
        if terminated {
            self.push(TokenKind::String(value), start, self.offset);
        } else {
            self.diagnostics.push(Diagnostic::error(
                "E1002",
                Phase::Lexer,
                "unterminated string literal",
                Span::new(self.source, start, self.offset),
            ));
        }
    }

    fn number(&mut self, start: usize) {
        // Immediately after a projection dot, decimal digits are an index,
        // even when followed by another dot and more digits.
        let projection_index = matches!(self.tokens.last().map(|t| &t.kind), Some(TokenKind::Dot));
        while self.peek_char().is_some_and(|ch| ch.is_ascii_digit()) {
            self.bump();
        }
        let mut is_float = false;
        if !projection_index
            && self.peek_char() == Some('.')
            && self
                .text
                .get(self.offset + 1..)
                .and_then(|rest| rest.chars().next())
                .is_some_and(|ch| ch.is_ascii_digit())
        {
            is_float = true;
            self.bump();
            while self.peek_char().is_some_and(|ch| ch.is_ascii_digit()) {
                self.bump();
            }
        }
        let lexeme = self
            .text
            .get(start..self.offset)
            .unwrap_or_default()
            .to_owned();
        let kind = if is_float {
            TokenKind::Float(lexeme)
        } else {
            TokenKind::Int(lexeme)
        };
        self.push(kind, start, self.offset);
    }

    fn identifier(&mut self, start: usize) {
        self.bump();
        while self.peek_char().is_some_and(is_ident_continue) {
            self.bump();
        }
        let text = self.text.get(start..self.offset).unwrap_or_default();
        let kind = match text {
            "fn" => TokenKind::Fn,
            "let" => TokenKind::Let,
            "mut" => TokenKind::Mut,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "while" => TokenKind::While,
            "return" => TokenKind::Return,
            "break" => TokenKind::Break,
            "continue" => TokenKind::Continue,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            _ => TokenKind::Identifier(text.to_owned()),
        };
        self.push(kind, start, self.offset);
    }

    fn single(&mut self, kind: TokenKind, start: usize) {
        self.bump();
        self.push(kind, start, self.offset);
    }

    fn unexpected(&mut self, start: usize, message: impl Into<String>) {
        self.diagnostics.push(Diagnostic::error(
            "E1001",
            Phase::Lexer,
            message,
            Span::new(self.source, start, self.offset),
        ));
    }

    fn push(&mut self, kind: TokenKind, start: usize, end: usize) {
        self.tokens.push(Token {
            kind,
            span: Span::new(self.source, start, end),
        });
    }

    fn peek_char(&self) -> Option<char> {
        self.text.get(self.offset..)?.chars().next()
    }

    fn peek_second_char(&self) -> Option<char> {
        let mut chars = self.text.get(self.offset..)?.chars();
        chars.next()?;
        chars.next()
    }

    fn bump(&mut self) -> Option<char> {
        let ch = self.peek_char()?;
        self.offset += ch.len_utf8();
        Some(ch)
    }

    fn consume_if(&mut self, expected: char) -> bool {
        if self.peek_char() == Some(expected) {
            self.bump();
            true
        } else {
            false
        }
    }
}

fn is_ident_start(ch: char) -> bool {
    ch == '_' || ch.is_alphabetic()
}

fn is_ident_continue(ch: char) -> bool {
    ch == '_' || ch.is_alphanumeric()
}

#[cfg(test)]
mod tests {
    use super::{TokenKind, lex};
    use hydra_source::SourceId;

    #[test]
    fn lexes_function_core() {
        let result = lex(SourceId::new(0), "fn main() {\n let x = 42\n}\n");
        assert!(result.diagnostics.is_empty());
        assert!(matches!(result.tokens[0].kind, TokenKind::Fn));
        assert!(
            result
                .tokens
                .iter()
                .any(|token| matches!(token.kind, TokenKind::Int(ref n) if n == "42"))
        );
    }

    #[test]
    fn unexpected_unicode_still_makes_progress() {
        let result = lex(SourceId::new(0), "😀😀");
        assert_eq!(result.diagnostics.len(), 2);
        assert!(matches!(
            result.tokens.last().map(|token| &token.kind),
            Some(TokenKind::Eof)
        ));
    }

    #[test]
    fn loop_control_keywords_are_reserved_but_longer_words_remain_identifiers() {
        let text = "break continue breakfast continued";
        let tokens = lex(SourceId::new(0), text).tokens;
        assert!(matches!(tokens[0].kind, TokenKind::Break));
        assert!(matches!(tokens[1].kind, TokenKind::Continue));
        assert!(matches!(&tokens[2].kind, TokenKind::Identifier(name) if name == "breakfast"));
        assert!(matches!(&tokens[3].kind, TokenKind::Identifier(name) if name == "continued"));
    }

    #[test]
    fn tuple_projection_digits_do_not_consume_following_dot_or_change_floats() {
        let result = lex(SourceId::new(0), "t.0.1 + 1.25\nt . 0\nt.0 + 2.75\n");
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
        let kinds: Vec<_> = result.tokens.iter().map(|token| &token.kind).collect();
        assert!(matches!(kinds[0], TokenKind::Identifier(n) if n == "t"));
        assert!(matches!(kinds[1], TokenKind::Dot));
        assert!(matches!(kinds[2], TokenKind::Int(n) if n == "0"));
        assert!(matches!(kinds[3], TokenKind::Dot));
        assert!(matches!(kinds[4], TokenKind::Int(n) if n == "1"));
        assert!(matches!(kinds[6], TokenKind::Float(n) if n == "1.25"));
        assert!(matches!(kinds[10], TokenKind::Int(n) if n == "0"));
        assert!(matches!(kinds[16], TokenKind::Float(n) if n == "2.75"));
    }

    #[test]
    fn dot_mode_is_reset_by_newline_and_independent_numbers() {
        let result = lex(SourceId::new(0), "t.\n1.25\nt.name\n3.50\n");
        let kinds: Vec<_> = result.tokens.iter().map(|token| &token.kind).collect();
        assert!(matches!(kinds[2], TokenKind::Newline));
        assert!(matches!(kinds[3], TokenKind::Float(n) if n == "1.25"));
        assert!(matches!(kinds[9], TokenKind::Float(n) if n == "3.50"));
    }
}
