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
        let _ = hydra_runtime::execute(&hir);
    }
});
