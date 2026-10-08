use hydra_cli::compile;
use hydra_runtime::execute;

fn run(source: &str) -> (String, Option<hydra_runtime::Value>) {
    let compiled = compile("loop_control.hyd", source);
    assert!(
        compiled.diagnostics.is_empty(),
        "{:?}",
        compiled.diagnostics
    );
    let result = execute(&compiled.hir.expect("valid control flow"));
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    (result.output, result.value)
}

#[test]
fn d001_runtime_matrix() {
    let cases = [
        (
            "fn main() {\n while true {\n break\n }\n println(1)\n}\n",
            "1\n",
        ),
        (
            "fn main() {\n let mut i = 0\n while i < 3 {\n i = i + 1\n continue\n println(99)\n }\n println(i)\n}\n",
            "3\n",
        ),
        (
            "fn main() {\n let mut x = 0\n while x < 2 {\n x = x + 1\n while true {\n break\n }\n continue\n println(99)\n }\n println(x)\n}\n",
            "2\n",
        ),
        (
            "fn main() {\n while true {\n if true {\n break\n }\n println(99)\n }\n println(1)\n}\n",
            "1\n",
        ),
        (
            "fn main() {\n let mut x = 0\n while x < 2 {\n x = x + 1\n if x == 1 {\n continue\n }\n println(x)\n }\n}\n",
            "2\n",
        ),
        (
            "fn main() {\n while true {\n if true {\n break\n } else {\n continue\n }\n println(99)\n }\n println(1)\n}\n",
            "1\n",
        ),
        (
            "fn main() {\n while true {\n let x = {\n break\n }\n println(x)\n }\n println(1)\n}\n",
            "1\n",
        ),
        (
            "fn main() {\n let mut x = 0\n while x < 2 {\n x = x + 1\n return {\n continue\n }\n }\n println(x)\n}\n",
            "2\n",
        ),
        (
            "fn main() {\n while true {\n while {\n break\n } {\n println(99)\n }\n println(99)\n }\n println(1)\n}\n",
            "1\n",
        ),
        (
            "fn main() {\n let mut x = 0\n while x < 2 {\n x = x + 1\n print({\n continue\n })\n println(99)\n }\n println(x)\n}\n",
            "2\n",
        ),
        (
            "fn main() {\n while true {\n 1 + {\n break\n }\n println(99)\n }\n println(1)\n}\n",
            "1\n",
        ),
        (
            "fn main() {\n let mut x = 0\n let mut y = 0\n while x < 2 {\n x = x + 1\n y = {\n continue\n }\n }\n println(y)\n}\n",
            "0\n",
        ),
        (
            "fn main() {\n let mut x = 0\n while x < 2 {\n x = x + 1\n println(false && {\n break\n })\n println(true || {\n continue\n })\n }\n println(x)\n}\n",
            "false\ntrue\nfalse\ntrue\n2\n",
        ),
        (
            "fn main() {\n while true {\n let x = if false {\n break\n } else {\n 5\n }\n println(x)\n break\n }\n}\n",
            "5\n",
        ),
        (
            "fn main() {\n while true {\n if false {\n return\n }\n if true {\n break\n }\n }\n println(1)\n}\n",
            "1\n",
        ),
        (
            "fn main() {\n while true {\n println(1)\n break\n }\n}\n",
            "1\n",
        ),
        (
            "fn main() {\n while true {\n let x = !{\n break\n }\n println(x)\n }\n println(1)\n}\n",
            "1\n",
        ),
        (
            "fn main() {\n while true {\n if {\n break\n } {\n println(99)\n }\n }\n println(1)\n}\n",
            "1\n",
        ),
        (
            "fn main() {\n while true {\n print({\n println(8)\n break\n })\n println(99)\n }\n println(1)\n}\n",
            "8\n1\n",
        ),
        (
            "fn get() -> Int {\n while true {\n return {\n break\n }\n }\n 7\n}\nfn main() {\n println(get())\n}\n",
            "7\n",
        ),
        (
            "fn get() -> Int {\n while true {\n return true && {\n break\n }\n }\n 7\n}\nfn main() {\n println(get())\n}\n",
            "7\n",
        ),
        (
            "fn main() {\n while true {\n let x = 1 + {\n break\n }\n println(x)\n }\n println(1)\n}\n",
            "1\n",
        ),
        (
            "fn main() {\n let mut x = 0\n while true {\n x = {\n break\n }\n println(99)\n }\n println(x)\n}\n",
            "0\n",
        ),
        (
            "fn main() {\n let mut i = 0\n while i < 2 {\n i = i + 1\n let x = {\n continue\n }\n println(x)\n }\n println(i)\n}\n",
            "2\n",
        ),
        (
            "fn main() {\n let mut i = 0\n while i < 2 {\n i = i + 1\n !{\n continue\n }\n println(99)\n }\n println(i)\n}\n",
            "2\n",
        ),
        (
            "fn main() {\n let mut i = 0\n while i < 2 {\n i = i + 1\n while {\n continue\n } {\n println(99)\n }\n println(99)\n }\n println(i)\n}\n",
            "2\n",
        ),
        (
            "fn id(x: Int) -> Int { x }\nfn main() {\n while true {\n println(id({\n break\n }))\n println(99)\n }\n println(1)\n}\n",
            "1\n",
        ),
        (
            "fn main() {\n let mut i = 0\n let mut j = 0\n while i < 2 {\n i = i + 1\n while j < 2 {\n j = j + 1\n continue\n println(99)\n }\n println(i)\n }\n}\n",
            "1\n2\n",
        ),
        (
            "fn choose() -> Int {\n while true {\n if false {\n break\n } else {\n return 5\n }\n }\n 7\n}\nfn main() {\n println(choose())\n}\n",
            "5\n",
        ),
        (
            "fn choose() -> Int {\n let mut i = 0\n while i < 2 {\n i = i + 1\n if i == 1 {\n continue\n }\n return 5\n }\n 7\n}\nfn main() {\n println(choose())\n}\n",
            "5\n",
        ),
        (
            "fn main() {\n while true {\n {\n break\n } + {\n println(99)\n 1\n }\n }\n println(1)\n}\n",
            "1\n",
        ),
    ];
    for (source, expected) in cases {
        assert_eq!(run(source).0, expected, "{source}");
    }
}

#[test]
fn d001_invalid_control_and_keyword_matrix() {
    let cases = [
        ("fn main() {\n break\n}\n", "E3010", "break"),
        ("fn main() {\n continue\n}\n", "E3011", "continue"),
        ("fn main() {\n return\n break\n}\n", "E3010", "break"),
        ("fn main() {\n return\n continue\n}\n", "E3011", "continue"),
        (
            "fn main() {\n while {\n break\n } {}\n}\n",
            "E3010",
            "break",
        ),
        (
            "fn helper() {\n break\n}\nfn main() {\n while true {\n helper()\n break\n }\n}\n",
            "E3010",
            "break",
        ),
        ("fn main() {\n break 1\n}\n", "E1101", "1"),
        ("fn main() {\n continue 1\n}\n", "E1101", "1"),
        ("fn main() {\n break(1)\n}\n", "E1101", "("),
        ("fn main() {\n continue(1)\n}\n", "E1101", "("),
        ("fn main() {\n let break = 1\n}\n", "E1101", "break"),
        ("fn main() {\n let continue = 1\n}\n", "E1101", "continue"),
        ("fn break() {}\nfn main() {}\n", "E1101", "break"),
        (
            "fn f(continue: Int) {}\nfn main() {}\n",
            "E1101",
            "continue",
        ),
    ];
    for (source, code, token) in cases {
        let result = compile("invalid.hyd", source);
        assert!(result.hir.is_none(), "should reject: {source}");
        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.code == code && &source[d.primary.start..d.primary.end] == token),
            "{source}: {:#?}",
            result.diagnostics
        );
    }
}

#[test]
fn d001_unreachable_source_still_typechecked() {
    let source = "fn main() {\n while true {\n break\n let x: Int = false\n }\n}\n";
    let compiled = compile("unreachable.hyd", source);
    assert!(
        compiled.diagnostics.iter().any(|d| d.code == "E3002"),
        "{:?}",
        compiled.diagnostics
    );
}

#[test]
fn d001_unconditional_continue_exhausts_fuel() {
    let compiled = compile("fuel.hyd", "fn main() {\n while true {\n continue\n }\n}\n");
    assert!(
        compiled.diagnostics.is_empty(),
        "{:?}",
        compiled.diagnostics
    );
    let result = execute(&compiled.hir.unwrap());
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].code, "E4006");
}
