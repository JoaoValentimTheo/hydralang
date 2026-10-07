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

## License

Licensed under either of Apache License 2.0 or MIT at your option.

