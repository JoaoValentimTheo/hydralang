use hydra_cli::compile;
use hydra_diagnostics::Phase;
use hydra_runtime::execute;
use hydra_source::SourceId;

fn run(source: &str) -> String {
    let compiled = compile("d002.hyd", source);
    assert!(
        compiled.diagnostics.is_empty(),
        "{:?}",
        compiled.diagnostics
    );
    let result = execute(&compiled.hir.expect("checked HIR"));
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    result.output
}

fn error(source: &str, code: &str) {
    let compiled = compile("d002_invalid.hyd", source);
    assert!(compiled.hir.is_none(), "expected rejection: {source}");
    assert!(
        compiled.diagnostics.iter().any(|d| d.code == code),
        "expected {code}, got {:?}",
        compiled.diagnostics
    );
}

#[test]
fn d002_inferred_tuple_dag_does_not_expand_types_exponentially() {
    // Source stays linear in the number of bindings, while the inferred
    // structural product has exponentially many logical leaves. Runtime
    // equality and HIR validation must use the shared representation.
    let mut source = String::from("fn main() {\n let t0 = (1,)\n");
    for depth in 1..=40 {
        source.push_str(&format!(
            " let t{depth} = (t{}, t{})\n",
            depth - 1,
            depth - 1
        ));
    }
    source.push_str(" println(t40 == t40)\n println(t40.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0)\n}\n");
    assert_eq!(run(&source), "true\n1\n");
}

fn nested_inferred_locals(last_depth: usize) -> String {
    let mut source = String::from("fn main() {\n let π = 1\n let t1 = (π,)\n");
    for depth in 2..=last_depth {
        source.push_str(&format!(
            " let t{depth} = (t{}, t{})\n",
            depth - 1,
            depth - 1
        ));
    }
    source.push_str(&format!(
        " println(t{last_depth}{})\n}}\n",
        ".0".repeat(last_depth)
    ));
    source
}

#[test]
fn d002_inferred_depth_64_succeeds_and_65_is_checker_owned() {
    let valid = nested_inferred_locals(64);
    assert_eq!(run(&valid), "1\n");

    let invalid = nested_inferred_locals(65);
    let first = compile("tuple-depth-65.hyd", &invalid);
    let second = compile("tuple-depth-65.hyd", &invalid);
    assert_eq!(first.diagnostics, second.diagnostics);
    assert!(first.hir.is_none(), "invalid inferred type escaped to HIR");
    let diagnostics: Vec<_> = first
        .diagnostics
        .iter()
        .filter(|d| d.code == "E3014")
        .collect();
    assert_eq!(diagnostics.len(), 1, "{:?}", first.diagnostics);
    let diagnostic = diagnostics[0];
    assert_eq!(diagnostic.phase, Phase::Type);
    assert!(diagnostic.message.contains("64"), "{}", diagnostic.message);
    assert!(diagnostic.message.contains("65"), "{}", diagnostic.message);
    assert_eq!(diagnostic.primary.source, SourceId::new(0));
    assert_eq!(
        &invalid[diagnostic.primary.start..diagnostic.primary.end],
        "(t64, t64)"
    );
    assert!(!first.diagnostics.iter().any(|d| d.code == "E9004"));
}

#[test]
fn d002_nested_function_argument_return_and_projection_respect_type_depth() {
    let explicit_64 = format!("{}Int{}", "(".repeat(64), ",)".repeat(64));
    let constructor_64 = format!("{}1{}", "(".repeat(64), ",)".repeat(64));
    let valid = format!(
        "fn identity(x: {explicit_64}) -> {explicit_64} {{ x }}\nfn main() {{\n let v = identity({constructor_64})\n println(v{})\n}}\n",
        ".0".repeat(64)
    );
    assert_eq!(run(&valid), "1\n");

    let invalid = format!(
        "fn identity(x: {explicit_64}) -> {explicit_64} {{ x }}\nfn main() {{\n let v = identity({constructor_64})\n let bad = (v,)\n println(7)\n}}\n"
    );
    let result = compile("tuple-call-depth.hyd", &invalid);
    assert!(result.hir.is_none());
    let diagnostic = result
        .diagnostics
        .iter()
        .find(|d| d.code == "E3014")
        .unwrap_or_else(|| panic!("{:?}", result.diagnostics));
    assert_eq!(diagnostic.phase, Phase::Type);
    assert_eq!(
        &invalid[diagnostic.primary.start..diagnostic.primary.end],
        "(v,)"
    );
}

#[test]
fn d002_construction_projection_annotations_and_equality() {
    let source = r#"
fn pair(x: Int) -> ((Int, String), Bool) {
    ((x, "ok"), true)
}
fn extract(p: ((Int, String), Bool)) -> Int {
    p.0.0
}
fn main() {
    let t = pair(42)
    println(extract(t))
    println(t.0.1)
    println(t.1)
    println((42,).0)
    println((7))
    println(())
    println((1, 2,) == (1, 2))
    println((1, 2) != (1, 3))
    let grouped: (Int) = 1
    println(grouped)
    let u: (Unit,) = ((),)
    println(u.0)
}
"#;
    assert_eq!(run(source), "42\nok\ntrue\n42\n7\n()\ntrue\ntrue\n1\n()\n");
}

#[test]
fn d002_snapshots_float_and_strict_d001_effects() {
    assert_eq!(
        run(
            "fn main() {\n let mut x = 1\n let t = (x, 2)\n x = 5\n println(t.0)\n println(1.25)\n}\n"
        ),
        "1\n1.25\n"
    );
    assert_eq!(
        run(
            "fn main() {\n while true {\n let t = (1, {\n break\n }, {\n println(99)\n 3\n })\n println(t.0)\n }\n println(7)\n}\n"
        ),
        "7\n"
    );
    assert_eq!(
        run(
            "fn main() {\n let mut x = 0\n while x < 2 {\n x = x + 1\n let t = (0, {\n continue\n }, {\n println(99)\n 3\n })\n println(t.0)\n }\n println(x)\n}\n"
        ),
        "2\n"
    );
}

#[test]
fn d002_diagnostics_and_limits() {
    error("fn main() {\n let t = (1, 2)\n println(t.2)\n}\n", "E3013");
    error("fn main() {\n println(7.0)\n println((7).0)\n}\n", "E3012");
    error("fn main() {\n let t = (1,)\n t.0 = 4\n}\n", "E1104");
    error("fn main() {\n let t = (1,)\n println(t.name)\n}\n", "E1101");
    error("fn main() {\n let t = (1,)\n println(t.-1)\n}\n", "E1101");
    error(
        "fn main() {\n let t = (1,)\n println(t.99999999999999999999999999999999)\n}\n",
        "E1102",
    );
    error("fn main() {\n println((1,))\n}\n", "E3007");
    error(
        "fn main() {\n let x: (String, Int) = (1, \"s\")\n}\n",
        "E3002",
    );
    let oversized = format!(
        "fn main() {{\n let x = ({})\n}}\n",
        vec!["1"; 65].join(", ")
    );
    error(&oversized, "E1101");
    let nested = format!(
        "fn main() {{\n let x: {}Int{} = 1\n}}\n",
        "(".repeat(65),
        ",)".repeat(65)
    );
    error(&nested, "E1105");
}

#[test]
fn d002_nan_equality_symmetry_complement_and_nested_sharing() {
    let source = r#"
fn main() {
    let n = 0.0 / 0.0
    let leaf = (n,)
    let a = ((leaf, leaf), 1)
    let b = ((leaf, leaf), 1)
    println(a == a)
    println(a == b)
    println(a != b)
    println(b == a)
    println((1, "x") == (1, "x"))
    println((1, "x") != (1, "x"))
    println((1, "x") == (2, "x"))
    println((1, "x") != (2, "x"))
}
"#;
    assert_eq!(
        run(source),
        "false\nfalse\ntrue\nfalse\ntrue\nfalse\nfalse\ntrue\n"
    );
}

#[test]
fn d002_strict_fields_stop_after_return_break_continue_or_error() {
    let returning = r#"
fn exit_early() -> Int {
    let ignored = (1, { return 9 }, { println(99)
        3
    })
    0
}
fn main() { println(exit_early()) }
"#;
    assert_eq!(run(returning), "9\n");

    let ordering = r#"
fn main() {
    let mut n = 0
    let t = ({ n = n + 1
        n }, { n = n + 1
        n })
    println(t.0)
    println(t.1)
    println(n)
}
"#;
    assert_eq!(run(ordering), "1\n2\n2\n");

    let failed = compile(
        "error-in-field.hyd",
        "fn main() {\n let t = (1 / 0, { println(99)\n 2 })\n println(t.0)\n}\n",
    );
    assert!(failed.diagnostics.is_empty(), "{:?}", failed.diagnostics);
    let result = execute(&failed.hir.expect("checked runtime-error HIR"));
    assert_eq!(result.diagnostics[0].code, "E4005");
    assert_eq!(result.output, "");
    assert!(result.value.is_none());
}

#[test]
fn d002_arity_and_nesting_exact_boundaries() {
    for size in [1, 2, 63, 64] {
        let literals = vec!["1"; size].join(", ");
        let types = vec!["Int"; size].join(", ");
        let source = format!(
            "fn main() {{\n let v: ({types},) = ({literals},)\n println(v.{})\n}}\n",
            size - 1
        );
        assert_eq!(run(&source), "1\n", "arity {size}");
    }
    let types = vec!["Int"; 65].join(", ");
    error(
        &format!("fn main() {{\n let v: ({types}) = ()\n}}\n"),
        "E1101",
    );
    for nesting in [1, 8, 32, 64] {
        let tuple_type = format!("{}Int{}", "(".repeat(nesting), ",)".repeat(nesting));
        let tuple_value = format!("{}1{}", "(".repeat(nesting), ",)".repeat(nesting));
        let source = format!("fn main() {{\n let v: {tuple_type} = {tuple_value}\n}}\n");
        let compiled = compile("tuple-type-depth.hyd", &source);
        assert!(
            compiled.diagnostics.is_empty(),
            "depth {nesting}: {:?}",
            compiled.diagnostics
        );
    }
    error(
        &format!(
            "fn main() {{\n let v: {}Int{} = 1\n}}\n",
            "(".repeat(65),
            ",)".repeat(65)
        ),
        "E1105",
    );
}

#[test]
fn d002_diagnostics_have_precise_utf8_spans_and_recover_deterministically() {
    for (source, code, target) in [
        (
            "fn main() {\n let π = (1, 2)\n println(π.3)\n}\n",
            "E3013",
            ".3",
        ),
        ("fn main() {\n let π = 7\n println(π.0)\n}\n", "E3012", ".0"),
        (
            "fn main() {\n let π = (1,)\n println(π.-1)\n}\n",
            "E1101",
            ".-",
        ),
    ] {
        let first = compile("tuple-spans.hyd", source);
        let second = compile("tuple-spans.hyd", source);
        assert_eq!(first.diagnostics, second.diagnostics);
        let diag = first
            .diagnostics
            .iter()
            .find(|d| d.code == code)
            .unwrap_or_else(|| panic!("{code}: {:?}", first.diagnostics));
        assert_eq!(diag.primary.source, SourceId::new(0));
        assert!(source.is_char_boundary(diag.primary.start));
        assert!(source.is_char_boundary(diag.primary.end));
        assert_eq!(&source[diag.primary.start..diag.primary.end], target);
    }
    for invalid in ["(1,,2)", "(1,", "(1,2", "(1,).name", "(1,).(0)"] {
        let source = format!("fn main() {{\n let t = {invalid}\n println(7)\n}}\n");
        let first = compile("tuple-malformed.hyd", &source);
        let second = compile("tuple-malformed.hyd", &source);
        assert!(first.hir.is_none());
        assert_eq!(first.diagnostics, second.diagnostics, "{invalid}");
        assert!(!first.diagnostics.is_empty(), "{invalid}");
    }
}
