# Agent State

- Current HEAD: initial repository commit pending
- Current milestone: H0-H10 foundation implemented and locally validated; H11+ remains
- Branch: `main`
- Rust edition: 2024
- Declared and locally verified MSRV: 1.85.0
- Pipeline: source -> lexer -> parser/AST -> resolution -> type checking -> typed HIR -> interpreter -> CLI
- Implemented types: Int, Float, Bool, String, Unit, Never
- Implemented language core: literals, immutable/mutable bindings, arithmetic, comparisons, boolean operators, functions/calls, if/else, while, return, blocks, local inference, explicit parameter/return types
- Builtins: `print`, `println`, defined through the central `hydra-stdlib` registry
- Runtime policy: checked i64 integer arithmetic, explicit division-by-zero/overflow diagnostics, 512 call-depth limit, 1,000,000-step execution budget
- Diagnostics: structured E1xxx/E2xxx/E3xxx/E4xxx/E9xxx families with source spans
- Tests: 10 Rust tests passing across source, lexer, parser, resolver, checker, and CLI/core execution
- Validation: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace --all-features`, `cargo +1.85.0 check --workspace`, and CLI check/run of `examples/hello.hyd` pass locally
- Known limitations: no golden program corpus yet; no property-test framework; no fuzz targets; no collections, modules, algebraic data types, generics, closures, native/WASM backends, package tooling, or REPL
- Open risks: parser/runtime depth hardening is not yet adversarially audited; negative diagnostic coverage is still small; documentation reconciliation is not final
- Next engineering action: publish the initial GitHub repository/CI checkpoint, then build H11 integration corpus, H12 property tests, and H13 fuzz foundations before H17/H18
