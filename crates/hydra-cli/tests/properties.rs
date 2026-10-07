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

fn assert_source_span(span: Span, source_id: SourceId, text: &str) {
    assert_eq!(span.source, source_id);
    assert!(span.start <= span.end);
    assert!(span.end <= text.len());
    assert!(text.is_char_boundary(span.start));
    assert!(text.is_char_boundary(span.end));
}

fn structured_programs() -> [(&'static str, &'static str); 6] {
    [
        (
            r#"fn main() {
    println(1 + 2 * 3)
    println((1 + 2) * 3)
    println(true || false && false)
}
"#,
            "7\n9\ntrue\n",
        ),
        (
            r#"fn main() {
    let mut x = 1
    println(x = 2)
    println(x)
}
"#,
            "()\n2\n",
        ),
        (
            r#"fn println(value: Int) -> Int {
    value + 1
}

fn main() {
    print(println(41))
}
"#,
            "42",
        ),
        (
            r#"fn choose() -> Int {
    println({
        return 7
    })
}

fn main() {
    println(choose())
}
"#,
            "7\n",
        ),
        (
            r#"fn main() {
    println(-9223372036854775807 - 1)
    println(1.0 / 0.0)
    println(0.0 / 0.0 == 0.0 / 0.0)
}
"#,
            "-9223372036854775808\ninf\nfalse\n",
        ),
        (
            r#"fn main() {
    println(false && (1 / 0 == 0))
    println(true || (1 / 0 == 0))
}
"#,
            "false\ntrue\n",
        ),
    ]
}

#[test]
fn lexer_makes_progress_and_emits_valid_spans_for_generated_utf8() {
    let source_id = SourceId::new(7);

    for text in generated_sources() {
        let lexed = lex(source_id, &text);
        assert!(!lexed.tokens.is_empty());

        let mut previous_end = 0;
        for token in &lexed.tokens {
            assert_source_span(token.span, source_id, &text);
            assert!(token.span.start >= previous_end);
            previous_end = token.span.end;
        }

        let eof = lexed.tokens.last().expect("lexer always appends EOF");
        assert!(matches!(eof.kind, TokenKind::Eof));
        assert_eq!(eof.span.start, text.len());
        assert_eq!(eof.span.end, text.len());

        for diagnostic in &lexed.diagnostics {
            assert_source_span(diagnostic.primary, source_id, &text);
        }
    }
}

#[test]
fn parser_terminates_for_generated_lexer_output() {
    let source_id = SourceId::new(11);

    for text in generated_sources() {
        let lexed = lex(source_id, &text);
        let parsed = parse(&lexed.tokens);
        let parsed_again = parse(&lexed.tokens);
        assert_eq!(parsed.diagnostics, parsed_again.diagnostics);
        assert_eq!(
            format!("{:?}", parsed.program),
            format!("{:?}", parsed_again.program)
        );
        for diagnostic in &parsed.diagnostics {
            assert_source_span(diagnostic.primary, source_id, &text);
        }
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
        assert_eq!(first.hir.is_some(), first.diagnostics.is_empty());
        for diagnostic in &first.diagnostics {
            assert_source_span(diagnostic.primary, SourceId::new(0), &text);
        }
    }
}

#[test]
fn structured_valid_programs_compile_and_execute_deterministically() {
    for (source, expected_output) in structured_programs() {
        let first = compile("structured.hyd", source);
        let second = compile("structured.hyd", source);
        assert_eq!(first.diagnostics, second.diagnostics);
        assert_eq!(format!("{:?}", first.hir), format!("{:?}", second.hir));
        assert!(first.diagnostics.is_empty(), "{:?}", first.diagnostics);

        let first_run = hydra_runtime::execute(first.hir.as_ref().expect("valid HIR"));
        let second_run = hydra_runtime::execute(second.hir.as_ref().expect("valid HIR"));
        assert_eq!(first_run.diagnostics, second_run.diagnostics);
        assert_eq!(first_run.output, second_run.output);
        assert!(
            first_run.diagnostics.is_empty(),
            "{:?}",
            first_run.diagnostics
        );
        assert_eq!(first_run.output, expected_output);
    }
}

#[test]
fn deliberate_edge_policies_have_stable_diagnostics() {
    let cases = [
        (
            "fn main() {\n    println(-9223372036854775808)\n}\n",
            "E1102",
        ),
        ("fn main() {\n    println(1); println(2)\n}\n", "E1001"),
        ("fn main() {\n    let a = 1 let b = 2\n}\n", "E1101"),
        (
            "fn f() -> Int {\n    return 1\n    1 + true\n}\nfn main() {\n    println(f())\n}\n",
            "E3003",
        ),
    ];

    for (source, expected_code) in cases {
        let first = compile("edge.hyd", source);
        let second = compile("edge.hyd", source);
        assert_eq!(first.diagnostics, second.diagnostics);
        assert!(first.hir.is_none());
        assert!(
            first
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == expected_code),
            "expected {expected_code}, got {:?}",
            first.diagnostics
        );
        for diagnostic in &first.diagnostics {
            assert_source_span(diagnostic.primary, SourceId::new(0), source);
        }
    }
}
