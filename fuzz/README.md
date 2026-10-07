# Hydra fuzzing

Hydra uses `cargo-fuzz` with libFuzzer targets for the lexer, parser, full compile pipeline, and runtime. The targets assert structural invariants in addition to looking for panics: source spans must remain valid UTF-8 boundaries, lexer output must end in EOF, compilation must either produce HIR or diagnostics, and runtime diagnostics must refer to valid source spans.

The reproducible fuzz toolchain for the 0.1 baseline is `nightly-2026-10-02` with `cargo-fuzz 0.13.2`. CI pins both. Advance the nightly only in a dedicated maintenance change that rebuilds all four targets, runs the bounded smoke suite, and records the new date here and in CI.

Build every target:

```text
cargo +nightly-2026-10-02 fuzz build
```

Run a bounded smoke pass, for example:

```text
cargo +nightly-2026-10-02 fuzz run lexer -- -runs=1000
cargo +nightly-2026-10-02 fuzz run parser -- -runs=1000
cargo +nightly-2026-10-02 fuzz run compile -- -runs=500
cargo +nightly-2026-10-02 fuzz run runtime -- -runs=100
```

The lexer target caps individual UTF-8 inputs at 64 KiB, parser and compiler at 16 KiB, and runtime at 4 KiB. The runtime target still relies on Hydra's deterministic 1,000,000-step execution budget and 128-frame call-depth limit.

Small checked-in seed sets under `fuzz/seeds/<target>/` cover valid programs, malformed syntax, Unicode, no-semicolon syntax, short-circuit behavior, `Never`, and integer-boundary forms. Pass that directory as an explicit corpus when reproducing CI locally; libFuzzer remains responsible for mutation and discovery.
