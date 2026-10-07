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
    let lexed = hydra_lexer::lex(SourceId::new(0), text);
    let _ = hydra_parser::parse(&lexed.tokens);
});
