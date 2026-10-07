#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    if text.len() > 16 * 1024 {
        return;
    }
    let compiled = hydra_cli::compile("fuzz.hyd", text);
    assert_eq!(compiled.hir.is_some(), compiled.diagnostics.is_empty());
    for diagnostic in &compiled.diagnostics {
        assert!(diagnostic.primary.start <= diagnostic.primary.end);
        assert!(diagnostic.primary.end <= text.len());
        assert!(text.is_char_boundary(diagnostic.primary.start));
        assert!(text.is_char_boundary(diagnostic.primary.end));
    }
});
