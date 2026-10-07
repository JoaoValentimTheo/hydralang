use hydra_cli::compile;
use std::fs;
use std::path::{Path, PathBuf};

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read_program(relative: &str) -> String {
    let path = repository_root().join(relative);
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

fn compile_program(relative: &str) -> hydra_cli::CompileResult {
    let source = read_program(relative);
    compile(relative, &source)
}

#[test]
fn passing_programs_execute_with_expected_output() {
    let cases = [
        ("tests/programs/pass/basic/hello.hyd", "Hydra\n42\ntrue\n"),
        (
            "tests/programs/pass/basic/unicode_string.hyd",
            "Olá λ 中 🙂\n",
        ),
        (
            "tests/programs/pass/basic/unicode_identifier.hyd",
            "Hydra\n42\n",
        ),
        (
            "tests/programs/pass/functions/fibonacci.hyd",
            "0\n1\n1\n2\n3\n5\n8\n13\n",
        ),
        (
            "tests/programs/pass/control_flow/classify.hyd",
            "negative\nzero\npositive\n",
        ),
        ("tests/programs/pass/control_flow/early_return.hyd", "7\n"),
        ("tests/programs/pass/control_flow/never_join.hyd", "20\n"),
        (
            "tests/programs/pass/control_flow/all_paths_return.hyd",
            "1\n",
        ),
        (
            "tests/programs/pass/types/primitives.hyd",
            "3.5\nfalse\nok\n",
        ),
    ];

    for (path, expected_output) in cases {
        let compiled = compile_program(path);
        assert!(
            compiled.diagnostics.is_empty(),
            "{path}: compile diagnostics: {:?}",
            compiled.diagnostics
        );
        let hir = compiled
            .hir
            .as_ref()
            .unwrap_or_else(|| panic!("{path}: HIR missing after successful compilation"));
        let run = hydra_runtime::execute(hir);
        assert!(
            run.diagnostics.is_empty(),
            "{path}: runtime diagnostics: {:?}",
            run.diagnostics
        );
        assert_eq!(run.output, expected_output, "{path}: unexpected output");
    }
}

#[test]
fn negative_compile_programs_report_the_expected_code() {
    let cases = [
        (
            "tests/programs/fail/lexer/unexpected_character.hyd",
            "E1001",
        ),
        ("tests/programs/fail/lexer/unterminated_string.hyd", "E1002"),
        ("tests/programs/fail/lexer/unknown_escape.hyd", "E1003"),
        (
            "tests/programs/fail/parser/incomplete_function.hyd",
            "E1101",
        ),
        (
            "tests/programs/fail/parser/out_of_range_integer.hyd",
            "E1102",
        ),
        (
            "tests/programs/fail/parser/invalid_assignment_target.hyd",
            "E1104",
        ),
        (
            "tests/programs/fail/resolution/duplicate_function.hyd",
            "E2001",
        ),
        (
            "tests/programs/fail/resolution/duplicate_local.hyd",
            "E2002",
        ),
        (
            "tests/programs/fail/resolution/duplicate_parameter.hyd",
            "E2002",
        ),
        ("tests/programs/fail/resolution/undefined_name.hyd", "E2004"),
        (
            "tests/programs/fail/resolution/immutable_assignment.hyd",
            "E2005",
        ),
        ("tests/programs/fail/type/type_mismatch.hyd", "E3002"),
        ("tests/programs/fail/type/bad_return.hyd", "E3002"),
        ("tests/programs/fail/type/unknown_type.hyd", "E3001"),
        ("tests/programs/fail/type/invalid_operand.hyd", "E3003"),
        ("tests/programs/fail/type/wrong_arity.hyd", "E3009"),
    ];

    for (path, expected_code) in cases {
        let compiled = compile_program(path);
        assert!(
            compiled.hir.is_none(),
            "{path}: invalid program produced HIR"
        );
        assert!(
            compiled
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == expected_code),
            "{path}: expected {expected_code}, got {:?}",
            compiled.diagnostics
        );
    }
}

#[test]
fn runtime_failure_programs_report_the_expected_code() {
    let cases = [
        ("tests/programs/fail/runtime/division_by_zero.hyd", "E4005"),
        ("tests/programs/fail/runtime/integer_overflow.hyd", "E4004"),
        ("tests/programs/fail/runtime/recursion_limit.hyd", "E4003"),
        ("tests/programs/fail/runtime/missing_main.hyd", "E4001"),
        ("tests/programs/fail/runtime/main_parameters.hyd", "E4002"),
        ("tests/programs/fail/runtime/step_budget.hyd", "E4006"),
    ];

    for (path, expected_code) in cases {
        let compiled = compile_program(path);
        assert!(
            compiled.diagnostics.is_empty(),
            "{path}: expected runtime failure, compile failed first: {:?}",
            compiled.diagnostics
        );
        let hir = compiled
            .hir
            .as_ref()
            .unwrap_or_else(|| panic!("{path}: HIR missing after successful compilation"));
        let run = hydra_runtime::execute(hir);
        assert!(
            run.diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == expected_code),
            "{path}: expected {expected_code}, got {:?}",
            run.diagnostics
        );
    }
}

#[test]
fn empty_source_reports_missing_main_at_runtime() {
    let compiled = compile("empty.hyd", "");
    assert!(
        compiled.diagnostics.is_empty(),
        "{:?}",
        compiled.diagnostics
    );
    let hir = compiled
        .hir
        .as_ref()
        .expect("empty source is syntactically valid and should produce empty HIR");
    let run = hydra_runtime::execute(hir);
    assert!(
        run.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "E4001")
    );
}
