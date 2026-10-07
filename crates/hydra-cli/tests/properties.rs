use hydra_cli::compile;
use hydra_lexer::{Token, TokenKind, lex};
use hydra_parser::parse;
use hydra_source::{SourceFile, SourceId, Span};

const CASES: usize = 384;

fn generated_sources() -> Vec<String> {
    let alphabet = [
        'a', 'Z', '_', '0', '9', ' ', '\t', '\n', '(', ')', '{', '}', '[', ']', ',', ':', '.', '+',
        '-', '*', '/', '%', '=', '!', '<', '>', '&', '|', '"', '\\', 'α', 'β', 'λ', '中', '🙂',
    ];
    let mut state = 0x9E37_79B9_7F4A_7C15_u64;
    let mut cases = Vec::with_capacity(CASES);

    for index in 0..CASES {
        let len = index % 97;
        let mut source = String::new();
        for _ in 0..len {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let slot = usize::try_from(state % alphabet.len() as u64).unwrap_or(0);
            source.push(alphabet[slot]);
        }
        cases.push(source);
    }

    cases
}

#[test]
fn lexer_makes_progress_and_emits_valid_spans_for_generated_utf8() {
    let source_id = SourceId::new(7);

    for text in generated_sources() {
        let lexed = lex(source_id, &text);
        assert!(!lexed.tokens.is_empty());

        let mut previous_end = 0;
        for token in &lexed.tokens {
            assert_eq!(token.span.source, source_id);
            assert!(token.span.start <= token.span.end);
            assert!(token.span.end <= text.len());
            assert!(text.is_char_boundary(token.span.start));
            assert!(text.is_char_boundary(token.span.end));
            assert!(token.span.start >= previous_end);
            previous_end = token.span.end;
        }

        let eof = lexed.tokens.last().expect("lexer always appends EOF");
        assert!(matches!(eof.kind, TokenKind::Eof));
        assert_eq!(eof.span.start, text.len());
        assert_eq!(eof.span.end, text.len());

        for diagnostic in &lexed.diagnostics {
            assert_eq!(diagnostic.primary.source, source_id);
            assert!(diagnostic.primary.start <= diagnostic.primary.end);
            assert!(diagnostic.primary.end <= text.len());
            assert!(text.is_char_boundary(diagnostic.primary.start));
            assert!(text.is_char_boundary(diagnostic.primary.end));
        }
    }
}

#[test]
fn parser_terminates_for_generated_lexer_output() {
    let source_id = SourceId::new(11);

    for text in generated_sources() {
        let lexed = lex(source_id, &text);
        let parsed = parse(&lexed.tokens);
        for diagnostic in &parsed.diagnostics {
            assert_eq!(diagnostic.primary.source, source_id);
            assert!(diagnostic.primary.start <= diagnostic.primary.end);
            assert!(diagnostic.primary.end <= text.len());
            assert!(text.is_char_boundary(diagnostic.primary.start));
            assert!(text.is_char_boundary(diagnostic.primary.end));
        }
        let _ = format!("{:?}", parsed.program);
    }
}

#[test]
fn parser_terminates_for_generated_token_streams_without_eof_contract() {
    let source_id = SourceId::new(17);
    let kinds = [
        TokenKind::Fn,
        TokenKind::Let,
        TokenKind::If,
        TokenKind::Else,
        TokenKind::While,
        TokenKind::Return,
        TokenKind::LeftParen,
        TokenKind::RightParen,
        TokenKind::LeftBrace,
        TokenKind::RightBrace,
        TokenKind::Comma,
        TokenKind::Colon,
        TokenKind::Plus,
        TokenKind::Minus,
        TokenKind::Equal,
        TokenKind::Newline,
    ];
    let mut state = 0xD1B5_4A32_D192_ED03_u64;

    for case in 0..256 {
        let len = case % 65;
        let mut tokens = Vec::with_capacity(len);
        for index in 0..len {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let slot = usize::try_from(state % kinds.len() as u64).unwrap_or(0);
            tokens.push(Token {
                kind: kinds[slot].clone(),
                span: Span::empty(source_id, index),
            });
        }
        let _ = parse(&tokens);
    }
}

#[test]
fn source_line_columns_exist_for_every_utf8_boundary() {
    for text in generated_sources() {
        let file = SourceFile::new(SourceId::new(13), "generated.hyd", text.clone());
        for offset in text
            .char_indices()
            .map(|(offset, _)| offset)
            .chain(std::iter::once(text.len()))
        {
            let span = file.span(offset, offset).expect("UTF-8 boundary is valid");
            assert_eq!(file.slice(span), Some(""));
            assert!(file.line_col(offset).is_some());
        }
    }
}

#[test]
fn compile_pipeline_is_deterministic_for_generated_sources() {
    for text in generated_sources() {
        let first = compile("generated.hyd", &text);
        let second = compile("generated.hyd", &text);

        assert_eq!(first.diagnostics, second.diagnostics);
        assert_eq!(format!("{:?}", first.hir), format!("{:?}", second.hir));
    }
}
