use hydra_cli::compile;
use hydra_diagnostics::Phase;
use hydra_runtime::execute;

fn run(source: &str) -> String {
    let result = compile("d003.hyd", source);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    let result = execute(&result.hir.expect("checked HIR"));
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    result.output
}

fn reject(source: &str, code: &str, phase: Phase) {
    let result = compile("d003_bad.hyd", source);
    assert!(result.hir.is_none(), "unexpected acceptance: {source}");
    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| d.code == code && d.phase == phase),
        "expected {code} ({phase:?}), got {:?}",
        result.diagnostics
    );
    for diagnostic in result.diagnostics {
        assert!(diagnostic.primary.start <= diagnostic.primary.end);
        assert!(source.is_char_boundary(diagnostic.primary.start));
        assert!(source.is_char_boundary(diagnostic.primary.end));
    }
}

fn runtime_error(source: &str, code: &str) {
    let result = compile("d003_runtime.hyd", source);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    let run = execute(&result.hir.expect("checked HIR"));
    assert!(
        run.diagnostics.iter().any(|d| d.code == code),
        "{:?}",
        run.diagnostics
    );
}

#[test]
fn d003_literals_index_chains_and_snapshots() {
    let source = r#"
fn choose() -> List<(Int, Bool)> { [(7, true), (9, false),] }
fn main() {
    let numbers = [10, 20, 30]
    let singleton = [5]
    println(numbers[0])
    println(numbers[2])
    println(singleton[0])
    println(choose()[1].0)
    let pair = (choose(), true)
    println(pair.0[0].1)
    let mut original: List<Int> = [1, 2]
    let snapshot = original
    original = [3, 4]
    println(snapshot[0])
    println(original[0])
    let List = 21
    println(List)
}
"#;
    assert_eq!(run(source), "10\n30\n5\n9\ntrue\n1\n3\n21\n");
}

#[test]
fn d003_empty_lists_in_all_five_contexts_and_context_isolation() {
    let source = r#"
fn take(xs: List<Int>) -> Int { 42 }
fn blank() -> List<Int> { [] }
fn also_blank() -> List<Int> { return [] }
fn main() {
    let typed: List<Int> = []
    let nested: List<List<Int>> = [[], []]
    let other: List<List<Int>> = [[], []]
    let pair: (List<Int>, Bool) = ([], true)
    let mut assigned: List<Int> = [1]
    assigned = []
    println(take([]))
    println(pair.1)
    println(nested == other)
    println(typed == blank())
    println(also_blank() == assigned)
}
"#;
    assert_eq!(run(source), "42\ntrue\ntrue\ntrue\ntrue\n");
    reject(
        "fn main() {\n let ok: List<Int> = []\n let bad = []\n}\n",
        "E3015",
        Phase::Type,
    );
    reject(
        "fn main() {\n let bad = []\n let ok: List<Int> = []\n}\n",
        "E3015",
        Phase::Type,
    );
    reject(
        "fn first() -> List<Int> { [] }\nfn main() { let bad = [] }\n",
        "E3015",
        Phase::Type,
    );
    reject(
        "fn main() { let x: List<Int> = if true { [] } else { [1] } }",
        "E3015",
        Phase::Type,
    );
}

#[test]
fn d003_index_diagnostics_and_type_errors() {
    reject("fn main() { let xs = [] }", "E3015", Phase::Type);
    reject("fn main() { let xs = [1, true] }", "E3002", Phase::Type);
    reject("fn main() { let xs: List<> = [] }", "E3017", Phase::Type);
    reject(
        "fn main() { let xs: List<Int, Bool> = [] }",
        "E3017",
        Phase::Type,
    );
    reject(
        "fn main() { let xs: Unknown<Int> = [] }",
        "E1101",
        Phase::Parser,
    );
    reject("fn main() { let xs: Unknown = 1 }", "E3001", Phase::Type);
    reject("fn main() { let xs = 1[0] }", "E3018", Phase::Type);
    reject("fn main() { let xs = [1][true] }", "E3019", Phase::Type);
    reject(
        "fn main() {\n let xs = [1]\n xs[0] = 2\n}",
        "E1104",
        Phase::Parser,
    );
    reject("fn main() { println([1]) }", "E3007", Phase::Type);
    reject("fn main() { let xs = [1] < [2] }", "E3003", Phase::Type);
    runtime_error("fn main() { let xs = [1, 2]\n println(xs[-1]) }", "E4007");
    runtime_error("fn main() { let xs = [1, 2]\n println(xs[2]) }", "E4007");
    runtime_error(
        "fn main() { let xs: List<Int> = []\n println(xs[0]) }",
        "E4007",
    );
}

#[test]
fn d003_bounds_error_spans_complete_index_operation() {
    let source = "fn main() { let xs = [1]\n println(xs[2]) }";
    let compiled = compile("bounds.hyd", source);
    assert!(
        compiled.diagnostics.is_empty(),
        "{:?}",
        compiled.diagnostics
    );
    let result = execute(&compiled.hir.expect("valid typed HIR"));
    assert_eq!(result.diagnostics.len(), 1);
    let diagnostic = &result.diagnostics[0];
    assert_eq!(diagnostic.code, "E4007");
    assert_eq!(diagnostic.phase, Phase::Runtime);
    assert_eq!(
        &source[diagnostic.primary.start..diagnostic.primary.end],
        "xs[2]"
    );
}

#[test]
fn d003_utf8_index_diagnostic_and_malformed_brackets_recover() {
    let source = "fn main() {\n let π = [1]\n println(π[true])\n}\n";
    let first = compile("list-utf8.hyd", source);
    let second = compile("list-utf8.hyd", source);
    assert_eq!(first.diagnostics, second.diagnostics);
    let diagnostic = first
        .diagnostics
        .iter()
        .find(|d| d.code == "E3019")
        .unwrap_or_else(|| panic!("expected E3019: {:?}", first.diagnostics));
    assert_eq!(diagnostic.phase, Phase::Type);
    assert!(source.is_char_boundary(diagnostic.primary.start));
    assert!(source.is_char_boundary(diagnostic.primary.end));
    assert_eq!(
        &source[diagnostic.primary.start..diagnostic.primary.end],
        "true"
    );

    for malformed in ["[1,,2]", "[1,", "[1,2", "π[", "π[0,1]"] {
        let source =
            format!("fn main() {{\n let π = [1]\n let xs = {malformed}\n println(7)\n}}\n");
        let first = compile("list-malformed.hyd", &source);
        let second = compile("list-malformed.hyd", &source);
        assert!(first.hir.is_none(), "{malformed}");
        assert_eq!(first.diagnostics, second.diagnostics, "{malformed}");
        assert!(!first.diagnostics.is_empty(), "{malformed}");
        for diagnostic in first.diagnostics {
            assert!(source.is_char_boundary(diagnostic.primary.start));
            assert!(source.is_char_boundary(diagnostic.primary.end));
        }
    }
}

#[test]
fn d003_equality_preserves_nan_in_shared_mixed_dags() {
    let source = r#"
fn main() {
    let n = 0.0 / 0.0
    let leaf = [n]
    let pair = (leaf, leaf)
    let a = [pair, pair]
    println(a == a)
    println(a != a)
    let b = [pair, pair]
    println(a == b)
    println([1, 2] == [1, 2])
    println([1, 2] != [1, 3])
    println([1] == [1, 2])
}
"#;
    assert_eq!(run(source), "false\ntrue\nfalse\ntrue\ntrue\nfalse\n");
}

#[test]
fn d003_list_element_count_boundary() {
    let list_256 = vec!["1"; 256].join(", ");
    assert_eq!(
        run(&format!("fn main() {{ println([{}][255]) }}", list_256)),
        "1\n"
    );
    let list_257 = vec!["1"; 257].join(", ");
    reject(
        &format!("fn main() {{ let xs = [{}] }}", list_257),
        "E1101",
        Phase::Parser,
    );
}

#[test]
fn d003_written_and_inferred_depth_boundary() {
    let written_64 = format!("{}Int{}", "List<".repeat(64), ">".repeat(64));
    let written_65 = format!("{}Int{}", "List<".repeat(65), ">".repeat(65));
    assert_eq!(
        run(&format!(
            "fn main() {{ let xs: {written_64} = {}1{} }}",
            "[".repeat(64),
            "]".repeat(64)
        )),
        ""
    );
    reject(
        &format!("fn main() {{ let xs: {written_65} = [] }}"),
        "E1105",
        Phase::Parser,
    );

    let mut source = String::from("fn main() {\n let x0 = 1\n");
    for depth in 1..=64 {
        source.push_str(&format!(" let x{depth} = [x{}]\n", depth - 1));
    }
    source.push_str(&format!(" println(x64{})\n}}\n", "[0]".repeat(64)));
    assert_eq!(run(&source), "1\n");
    let invalid = source.replace(" println(x64", " let x65 = [x64]\n println(x64");
    reject(&invalid, "E3016", Phase::Type);
}

#[test]
fn d003_mixed_inferred_list_tuple_depth_64_and_65() {
    let mut source = String::from("fn main() {\n let x0 = 1\n");
    for depth in 1..=64 {
        if depth % 2 == 0 {
            source.push_str(&format!(" let x{depth} = [x{}]\n", depth - 1));
        } else {
            source.push_str(&format!(" let x{depth} = (x{},)\n", depth - 1));
        }
    }
    assert_eq!(run(&format!("{source} println(1)\n}}\n")), "1\n");
    reject(
        &format!("{source} let x65 = (x64,)\n}}\n"),
        "E3016",
        Phase::Type,
    );
}

#[test]
fn d003_left_to_right_short_circuit_d001_effects() {
    assert_eq!(
        run(
            "fn main() {\n while true {\n let xs = [1, {\n break\n }, {\n println(99)\n 3\n }]\n }\n println(7)\n}\n"
        ),
        "7\n"
    );
    assert_eq!(
        run(
            "fn main() {\n let mut n = 0\n while n < 2 {\n n = n + 1\n let xs = [1, {\n continue\n }, {\n println(99)\n 3\n }]\n }\n println(n)\n}\n"
        ),
        "2\n"
    );
    assert_eq!(
        run(
            "fn answer() -> Int {\n let xs = [1, {\n return 9\n }, {\n println(99)\n 3\n }]\n 0\n}\nfn main() { println(answer()) }\n"
        ),
        "9\n"
    );
}
