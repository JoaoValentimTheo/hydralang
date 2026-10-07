use hydra_diagnostics::Diagnostic;
use hydra_hir::HirProgram;
use hydra_source::SourceMap;

#[derive(Debug)]
pub struct CompileResult {
    pub sources: SourceMap,
    pub hir: Option<HirProgram>,
    pub diagnostics: Vec<Diagnostic>,
}

#[must_use]
pub fn compile(name: &str, text: &str) -> CompileResult {
    let mut sources = SourceMap::new();
    let source = sources.add(name, text);

    let lexed = hydra_lexer::lex(source, text);
    if !lexed.diagnostics.is_empty() {
        return CompileResult {
            sources,
            hir: None,
            diagnostics: lexed.diagnostics,
        };
    }

    let parsed = hydra_parser::parse(&lexed.tokens);
    if !parsed.diagnostics.is_empty() {
        return CompileResult {
            sources,
            hir: None,
            diagnostics: parsed.diagnostics,
        };
    }

    let resolved = hydra_resolve::resolve(&parsed.program);
    if !resolved.diagnostics.is_empty() {
        return CompileResult {
            sources,
            hir: None,
            diagnostics: resolved.diagnostics,
        };
    }

    let checked = hydra_check::check(&parsed.program, &resolved.resolution);
    CompileResult {
        sources,
        hir: checked.hir,
        diagnostics: checked.diagnostics,
    }
}

#[cfg(test)]
mod tests {
    use super::compile;

    #[test]
    fn compiles_and_executes_fibonacci_core() {
        let source = r#"
fn fib(n: Int) -> Int {
    if n <= 1 {
        n
    } else {
        fib(n - 1) + fib(n - 2)
    }
}

fn main() {
    let mut i = 0
    while i < 6 {
        println(fib(i))
        i = i + 1
    }
}
"#;
        let compiled = compile("fib.hyd", source);
        assert!(
            compiled.diagnostics.is_empty(),
            "{:?}",
            compiled.diagnostics
        );
        let Some(hir) = compiled.hir else {
            panic!("HIR must exist when diagnostics are empty");
        };
        let result = hydra_runtime::execute(&hir);
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
        assert_eq!(result.output, "0\n1\n1\n2\n3\n5\n");
    }

    #[test]
    fn reports_type_mismatch_without_runtime() {
        let source = "fn main() {\n let x: Int = \"no\"\n}\n";
        let compiled = compile("bad.hyd", source);
        assert!(compiled.diagnostics.iter().any(|d| d.code == "E3002"));
        assert!(compiled.hir.is_none());
    }
}
