# Hydra fuzzing

Hydra uses `cargo-fuzz` with libFuzzer targets for the lexer, parser, full compile pipeline, and runtime.

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

The parser and compiler targets cap individual UTF-8 inputs at 16 KiB. The runtime target caps them at 4 KiB and still relies on Hydra's deterministic execution budget and call-depth limit.
