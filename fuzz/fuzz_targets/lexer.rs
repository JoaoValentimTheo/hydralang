#![no_main]

use hydra_source::SourceId;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    let result = hydra_lexer::lex(SourceId::new(0), text);
    for token in result.tokens {
        let _ = text.get(token.span.start..token.span.end);
    }
});
