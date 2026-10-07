# Hydra

Hydra is an independently implemented, statically typed programming language written in Rust.

Hydra 0.1 is focused on a small, coherent compiler pipeline:

```text
source -> lexer -> parser/AST -> resolution -> type checking -> typed HIR -> interpreter
```

The repository is pre-release. Hydra 0.1 has a validated engineering baseline, but there is no public Hydra 0.1 tag or GitHub release and no published compatibility commitment. Intentional language changes must follow `docs/VERSIONING.md` and the repository decision-record process.

## Development

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Run a program with:

```bash
cargo run -p hydra-cli -- run examples/hello.hyd
```

Check without executing:

```bash
cargo run -p hydra-cli -- check examples/hello.hyd
```

Hydra does not use semicolons as ordinary statement separators. Mutability is explicit with `let mut`.

The 0.1 semantic contract is intentionally small and explicit. `Int` is checked signed 64-bit integer arithmetic; `Float` follows Rust `f64`/IEEE behavior, including infinities and NaN. Ordinary assignment evaluates to `Unit`, `Never` is the bottom type for diverging control flow, and lexical lookup prefers locals, then user functions, then builtins. The only 0.1 builtins are `print(value)` and `println(value)`.

The parser limits recursive syntax nesting to 128 levels and constructed expression-tree depth to 256 levels. The reference interpreter limits call depth to 128 frames and each execution to 1,000,000 evaluation steps. See `spec/` for the normative 0.1 language contract, `docs/BASELINE.md` for the frozen engineering baseline, and `docs/ERROR_CODES.md` for structured diagnostics.

## License

Licensed under either of Apache License 2.0 or MIT at your option.
