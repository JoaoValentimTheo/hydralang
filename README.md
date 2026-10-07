# Hydra

Hydra is an independently implemented, statically typed programming language written in Rust.

Hydra 0.1 is focused on a small, coherent compiler pipeline:

```text
source -> lexer -> parser/AST -> resolution -> type checking -> typed HIR -> interpreter
```

The repository is pre-release. No compatibility guarantee is made until the Hydra 0.1 release candidate gate is satisfied.

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

The parser limits syntax nesting to 128 levels. The reference interpreter limits call depth to 128 frames and each execution to 1,000,000 evaluation steps. See `spec/` for the normative 0.1 language contract and `docs/ERROR_CODES.md` for structured diagnostics.

## License

Licensed under either of Apache License 2.0 or MIT at your option.
