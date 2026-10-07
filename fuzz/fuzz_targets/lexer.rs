#![no_main]

use hydra_source::SourceId;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    if text.len() > 64 * 1024 {
        return;
    }

    let source = SourceId::new(0);
    let result = hydra_lexer::lex(source, text);
    assert!(!result.tokens.is_empty());

    let mut previous_end = 0;
    for token in &result.tokens {
        assert_eq!(token.span.source, source);
        assert!(token.span.start <= token.span.end);
        assert!(token.span.end <= text.len());
        assert!(text.is_char_boundary(token.span.start));
        assert!(text.is_char_boundary(token.span.end));
        assert!(token.span.start >= previous_end);
        previous_end = token.span.end;
    }

    let eof = result.tokens.last().expect("lexer emits EOF");
    assert!(matches!(eof.kind, hydra_lexer::TokenKind::Eof));
    assert_eq!(eof.span.start, text.len());
    assert_eq!(eof.span.end, text.len());

    for diagnostic in &result.diagnostics {
        assert_eq!(diagnostic.primary.source, source);
        assert!(diagnostic.primary.start <= diagnostic.primary.end);
        assert!(diagnostic.primary.end <= text.len());
        assert!(text.is_char_boundary(diagnostic.primary.start));
        assert!(text.is_char_boundary(diagnostic.primary.end));
    }
});
