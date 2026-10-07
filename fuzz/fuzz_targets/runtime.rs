#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    if text.len() > 4 * 1024 {
        return;
    }
    let compiled = hydra_cli::compile("fuzz.hyd", text);
    if let Some(hir) = compiled.hir {
        assert!(compiled.diagnostics.is_empty());
        let result = hydra_runtime::execute(&hir);
        for diagnostic in &result.diagnostics {
            assert!(diagnostic.primary.start <= diagnostic.primary.end);
            assert!(diagnostic.primary.end <= text.len());
            assert!(text.is_char_boundary(diagnostic.primary.start));
            assert!(text.is_char_boundary(diagnostic.primary.end));
        }
    }
});
