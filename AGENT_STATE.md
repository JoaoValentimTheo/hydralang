# Agent State

- Current repository checkpoint: branch `main`; the repository HEAD is the commit containing the latest version of this state file
- Current milestone: Hydra 0.1 foundation H0-H14 and H16-H18 complete; H15 collections remains intentionally deferred
- Branch: `main`
- Rust edition: 2024
- Declared and locally verified MSRV: 1.85.0
- Pipeline: source -> lexer -> parser/AST -> resolution -> type checking -> typed HIR -> interpreter -> CLI
- Implemented types: Int, Float, Bool, String, Unit, Never
- Implemented language core: literals, immutable/mutable bindings, arithmetic, comparisons, boolean operators, functions/calls, if/else, while, return, blocks, local inference, explicit parameter/return types
- Builtins: `print`, `println`, defined through the central `hydra-stdlib` registry
- Runtime policy: checked i64 integer arithmetic, explicit division-by-zero/overflow diagnostics, 128 call-depth limit, 1,000,000-step execution budget
- Diagnostics: stable E1xxx/E2xxx/E3xxx/E4xxx/E9xxx inventory documented in `docs/ERROR_CODES.md`, with source spans on compiler/runtime diagnostics
- Test architecture: unit tests plus classified pass/compile-fail/runtime-fail program corpus and deterministic property suites over generated UTF-8/token streams
- Fuzzing: buildable `cargo-fuzz` targets for lexer, parser, complete compile pipeline, and runtime; bounded smoke runs completed without crashes before final closeout
- Robustness changes from adversarial review: parser synthetic EOF and 128-level syntax-depth guard; 128-frame runtime call guard; deterministic execution-step budget; corrected `Never` propagation through early returns and fully diverging statements
- Validation floor: fmt, clippy with warnings denied, workspace all-features tests, MSRV check, fuzz build/smokes, and CLI hello check/run
- Known limitations: no collections, modules/imports, algebraic data types, generics, closures/first-class functions, native/WASM backends, package tooling, LSP, REPL, or optimizer
- Open risks: the 0.1 parser and interpreter intentionally use conservative fixed depth/resource limits; resource policy may need tuning as the language grows
- Next engineering action: begin a new milestone only after choosing the next language surface; collections remain an explicit deferred decision rather than unfinished 0.1 foundation work
