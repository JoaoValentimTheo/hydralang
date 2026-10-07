# Hydra fuzzing

Hydra uses `cargo-fuzz` with libFuzzer targets for the lexer, parser, full compile pipeline, and runtime. The targets assert structural invariants in addition to looking for panics: source spans must remain valid UTF-8 boundaries, lexer output must end in EOF, compilation must either produce HIR or diagnostics, and runtime diagnostics must refer to valid source spans.

Build every target:

```text
cargo +nightly fuzz build
```

Run a bounded smoke pass, for example:

```text
cargo +nightly fuzz run lexer -- -runs=1000
cargo +nightly fuzz run parser -- -runs=1000
cargo +nightly fuzz run compile -- -runs=500
cargo +nightly fuzz run runtime -- -runs=100
```

The lexer target caps individual UTF-8 inputs at 64 KiB, parser and compiler at 16 KiB, and runtime at 4 KiB. The runtime target still relies on Hydra's deterministic 1,000,000-step execution budget and 128-frame call-depth limit.

Small checked-in seed sets under `fuzz/seeds/<target>/` cover valid programs, malformed syntax, Unicode, no-semicolon syntax, short-circuit behavior, `Never`, and integer-boundary forms. Pass that directory as an explicit corpus when reproducing CI locally; libFuzzer remains responsible for mutation and discovery.
