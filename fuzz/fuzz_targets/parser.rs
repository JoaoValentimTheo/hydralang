#![no_main]

use hydra_source::SourceId;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    if text.len() > 16 * 1024 {
        return;
    }
    let source = SourceId::new(0);
    let lexed = hydra_lexer::lex(source, text);
    let parsed = hydra_parser::parse(&lexed.tokens);
    for diagnostic in &parsed.diagnostics {
        assert_eq!(diagnostic.primary.source, source);
        assert!(diagnostic.primary.start <= diagnostic.primary.end);
        assert!(diagnostic.primary.end <= text.len());
        assert!(text.is_char_boundary(diagnostic.primary.start));
        assert!(text.is_char_boundary(diagnostic.primary.end));
    }
});
