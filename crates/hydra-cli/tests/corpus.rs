use hydra_cli::compile;
use hydra_diagnostics::Phase;
use std::collections::BTreeMap;
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
fn every_public_example_compiles_and_executes_with_exact_output() {
    let expected = BTreeMap::from([
        ("arithmetic.hyd", "7\n9\n5\n2\n"),
        ("blocks.hyd", "42\n"),
        ("booleans.hyd", "true\nfalse\ntrue\n"),
        ("conditionals.hyd", "positive\n"),
        ("early_return.hyd", "7\n"),
        ("fibonacci.hyd", "0\n1\n1\n2\n3\n5\n8\n13\n"),
        ("functions.hyd", "42\n"),
        ("hello.hyd", "Hydra\n"),
        ("mutability.hyd", "2\n"),
        ("recursion.hyd", "120\n"),
        ("strings.hyd", "Hydra λ\n"),
        ("type_inference.hyd", "42\n3.5\ntrue\nhydra\n()\n"),
        ("variables.hyd", "42\nHydra\n"),
        ("while_loop.hyd", "0\n1\n2\n"),
    ]);
    let examples_dir = repository_root().join("examples");
    let mut discovered = fs::read_dir(&examples_dir)
        .expect("examples directory must be readable")
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            (path.extension().and_then(|ext| ext.to_str()) == Some("hyd"))
                .then(|| path.file_name()?.to_str().map(str::to_owned))
                .flatten()
        })
        .collect::<Vec<_>>();
    discovered.sort();
    assert_eq!(
        discovered,
        expected
            .keys()
            .map(|name| (*name).to_owned())
            .collect::<Vec<_>>(),
        "every public .hyd example must have an exact-output contract"
    );

    for (name, expected_output) in expected {
        let relative = format!("examples/{name}");
        let compiled = compile_program(&relative);
        assert!(
            compiled.diagnostics.is_empty(),
            "{relative}: compile diagnostics: {:?}",
            compiled.diagnostics
        );
        let hir = compiled
            .hir
            .as_ref()
            .unwrap_or_else(|| panic!("{relative}: HIR missing after successful compilation"));
        let run = hydra_runtime::execute(hir);
        assert!(
            run.diagnostics.is_empty(),
            "{relative}: runtime diagnostics: {:?}",
            run.diagnostics
        );
        assert_eq!(run.output, expected_output, "{relative}: unexpected output");
    }
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
            "tests/programs/pass/control_flow/never_builtin_argument.hyd",
            "",
        ),
        (
            "tests/programs/pass/control_flow/all_paths_return.hyd",
            "1\n",
        ),
        (
            "tests/programs/pass/types/primitives.hyd",
            "3.5\nfalse\nok\n",
        ),
        (
            "tests/programs/pass/expressions/precedence_associativity.hyd",
            "7\n9\n12\n2\ntrue\ntrue\ntrue\ntrue\n-2\n",
        ),
        ("tests/programs/pass/syntax/multiline_boundaries.hyd", "8\n"),
        (
            "tests/programs/pass/control_flow/never_strict_contexts.hyd",
            "11\n12\n13\n14\n15\n16\n17\n",
        ),
        (
            "tests/programs/pass/control_flow/short_circuit_runtime_guards.hyd",
            "false\ntrue\nfalse\ntrue\n",
        ),
        (
            "tests/programs/pass/control_flow/while_return.hyd",
            "7\n0\n",
        ),
        (
            "tests/programs/pass/functions/contracts.hyd",
            "42\n()\n6\n7\ntrue\n",
        ),
        (
            "tests/programs/pass/numeric/integer_edges.hyd",
            "0\n-1\n9223372036854775807\n-9223372036854775808\n9223372036854775806\n-9223372036854775807\n-1\n",
        ),
        (
            "tests/programs/pass/numeric/float_ieee.hyd",
            "3.5\n2.5\n3\n2.5\n1.5\ntrue\ntrue\n-1.5\ninf\n-inf\nNaN\nfalse\ntrue\n",
        ),
        (
            "tests/programs/pass/basic/string_boolean_semantics.hyd",
            "\nHydra λ\ntrue\ntrue\nline\nnext\nquote: \" slash: \\\ntrue\ntrue\nfalse\n",
        ),
        (
            "tests/programs/pass/resolution/scope_mutability.hyd",
            "2\n1\n15\n7\n9\n",
        ),
        ("tests/programs/pass/runtime/call_depth_limit_ok.hyd", "0\n"),
        ("tests/programs/pass/runtime/finite_loop.hyd", "1000\n"),
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
            Phase::Lexer,
        ),
        (
            "tests/programs/fail/lexer/unterminated_string.hyd",
            "E1002",
            Phase::Lexer,
        ),
        (
            "tests/programs/fail/lexer/unknown_escape.hyd",
            "E1003",
            Phase::Lexer,
        ),
        (
            "tests/programs/fail/lexer/semicolon_statement.hyd",
            "E1001",
            Phase::Lexer,
        ),
        (
            "tests/programs/fail/parser/incomplete_function.hyd",
            "E1101",
            Phase::Parser,
        ),
        (
            "tests/programs/fail/parser/out_of_range_integer.hyd",
            "E1102",
            Phase::Parser,
        ),
        (
            "tests/programs/fail/parser/invalid_assignment_target.hyd",
            "E1104",
            Phase::Parser,
        ),
        (
            "tests/programs/fail/parser/same_line_statements.hyd",
            "E1101",
            Phase::Parser,
        ),
        (
            "tests/programs/fail/parser/direct_min_literal.hyd",
            "E1102",
            Phase::Parser,
        ),
        (
            "tests/programs/fail/parser/malformed_eof_nested.hyd",
            "E1101",
            Phase::Parser,
        ),
        (
            "tests/programs/fail/parser/missing_function_name.hyd",
            "E1101",
            Phase::Parser,
        ),
        (
            "tests/programs/fail/resolution/duplicate_function.hyd",
            "E2001",
            Phase::Resolution,
        ),
        (
            "tests/programs/fail/resolution/duplicate_local.hyd",
            "E2002",
            Phase::Resolution,
        ),
        (
            "tests/programs/fail/resolution/duplicate_parameter.hyd",
            "E2002",
            Phase::Resolution,
        ),
        (
            "tests/programs/fail/resolution/undefined_name.hyd",
            "E2004",
            Phase::Resolution,
        ),
        (
            "tests/programs/fail/resolution/immutable_assignment.hyd",
            "E2005",
            Phase::Resolution,
        ),
        (
            "tests/programs/fail/resolution/scope_leak.hyd",
            "E2004",
            Phase::Resolution,
        ),
        (
            "tests/programs/fail/type/type_mismatch.hyd",
            "E3002",
            Phase::Type,
        ),
        (
            "tests/programs/fail/type/bad_return.hyd",
            "E3002",
            Phase::Type,
        ),
        (
            "tests/programs/fail/type/unknown_type.hyd",
            "E3001",
            Phase::Type,
        ),
        (
            "tests/programs/fail/type/invalid_operand.hyd",
            "E3003",
            Phase::Type,
        ),
        (
            "tests/programs/fail/type/wrong_arity.hyd",
            "E3009",
            Phase::Type,
        ),
        (
            "tests/programs/fail/type/chained_assignment.hyd",
            "E3002",
            Phase::Type,
        ),
        (
            "tests/programs/fail/type/wrong_condition_type.hyd",
            "E3002",
            Phase::Type,
        ),
        (
            "tests/programs/fail/type/incompatible_if_branches.hyd",
            "E3004",
            Phase::Type,
        ),
        (
            "tests/programs/fail/type/value_if_without_else.hyd",
            "E3005",
            Phase::Type,
        ),
        (
            "tests/programs/fail/type/wrong_argument_type.hyd",
            "E3002",
            Phase::Type,
        ),
        (
            "tests/programs/fail/type/invalid_unary.hyd",
            "E3003",
            Phase::Type,
        ),
    ];

    for (path, expected_code, expected_phase) in cases {
        let compiled = compile_program(path);
        assert!(
            compiled.hir.is_none(),
            "{path}: invalid program produced HIR"
        );
        assert!(
            compiled.diagnostics.iter().any(|diagnostic| {
                diagnostic.code == expected_code && diagnostic.phase == expected_phase
            }),
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
        (
            "tests/programs/fail/runtime/integer_overflow_add.hyd",
            "E4004",
        ),
        (
            "tests/programs/fail/runtime/integer_overflow_subtract.hyd",
            "E4004",
        ),
        (
            "tests/programs/fail/runtime/integer_overflow_multiply.hyd",
            "E4004",
        ),
        (
            "tests/programs/fail/runtime/integer_overflow_negate.hyd",
            "E4004",
        ),
        (
            "tests/programs/fail/runtime/integer_overflow_remainder.hyd",
            "E4004",
        ),
        ("tests/programs/fail/runtime/remainder_by_zero.hyd", "E4005"),
        ("tests/programs/fail/runtime/recursion_limit.hyd", "E4003"),
        (
            "tests/programs/fail/runtime/call_depth_limit_exceeded.hyd",
            "E4003",
        ),
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
                .any(|diagnostic| diagnostic.code == expected_code
                    && diagnostic.phase == Phase::Runtime),
            "{path}: expected {expected_code}, got {:?}",
            run.diagnostics
        );
    }
}

#[test]
fn syntax_depth_boundary_is_enforced() {
    fn nested_expression(depth: usize) -> String {
        format!("{}1{}", "(".repeat(depth), ")".repeat(depth))
    }

    let accepted = format!(
        "fn main() {{\n    let value = {}\n    println(value)\n}}\n",
        nested_expression(126)
    );
    let accepted = compile("depth-accepted.hyd", &accepted);
    assert!(
        accepted.diagnostics.is_empty(),
        "boundary program should compile: {:?}",
        accepted.diagnostics
    );
    assert!(accepted.hir.is_some());

    let rejected = format!(
        "fn main() {{\n    let value = {}\n    println(value)\n}}\n",
        nested_expression(127)
    );
    let rejected = compile("depth-rejected.hyd", &rejected);
    assert!(rejected.hir.is_none());
    assert!(
        rejected
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "E1105" && diagnostic.phase == Phase::Parser)
    );
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
