# Hydra 0.1 Ledger

| Milestone | Status | Evidence |
| --- | --- | --- |
| H0 — Repository constitution | COMPLETE | Rust workspace, constitution/spec docs, licenses, contribution guide, state ledger, GitHub repository, and GitHub Actions CI exist |
| H1 — Kof reference analysis | COMPLETE | `docs/KOF_REFERENCE_NOTES.md`; read-only Kof4j review with ADOPT/ADAPT/AVOID/DEFER decisions |
| H2 — Source model and diagnostics | COMPLETE | `hydra-source` + `hydra-diagnostics`; Unicode/source-span tests and malformed-span safety test pass |
| H3 — Lexer | COMPLETE | Deterministic lexer with diagnostics, newline/EOF model, comments, literals/operators, and explicit unexpected-input progress test |
| H4 — Parser and AST | COMPLETE | Typed AST + Pratt parser for the 0.1 core; function/block/recovery tests pass |
| H5 — Name resolution | COMPLETE | Lexical scopes, stable IDs, deterministic maps, duplicate/undefined/immutability checks; immutable-assignment regression test passes |
| H6 — Primitive type system | COMPLETE | Int/Float/Bool/String/Unit/Never, local inference, type joins, explicit function signatures, operator checking |
| H7 — Typed HIR | COMPLETE | Backend-neutral typed HIR with resolved function/symbol identities; checker lowering test passes |
| H8 — Interpreter/runtime | COMPLETE | HIR interpreter, checked integer arithmetic, structured runtime errors, step budget and call-depth guard; fib execution passes |
| H9 — Functions and control flow | COMPLETE | Functions/calls, recursion, if/else, while, return, blocks, mutable assignment; recursive Fibonacci execution passes |
| H10 — CLI | COMPLETE | `hydra check`, `hydra run`, `hydra --version`, `hydra help`; local check/run of `examples/hello.hyd` passes |
| H11 — Integration corpus | COMPLETE | Classified `tests/programs` corpus covers passing programs plus lexer/parser/resolution/type/runtime failures with expected codes |
| H12 — Property tests | COMPLETE | Deterministic generated-input suites cover lexer progress/spans, parser termination with and without EOF, UTF-8 source boundaries, and compile determinism |
| H13 — Fuzz foundations | COMPLETE | `cargo-fuzz` package has buildable lexer/parser/compile/runtime targets; bounded smoke runs completed without crashes |
| H14 — Standard library foundation | COMPLETE | Central builtin/signature registry provides `print` and `println` to resolver/checker/runtime |
| H15 — Collections | NOT STARTED | Intentionally deferred; the foundation program makes collections conditional on core stability and they are not part of the 0.1 feature floor |
| H16 — Documentation reconciliation | COMPLETE | Language, lexical grammar, grammar, type system, execution model, complete diagnostic inventory, architecture, roadmap, Kof notes, state, and ledger reconciled with implementation |
| H17 — Hydra 0.1 adversarial review | COMPLETE | Deep parser input, missing EOF, Unicode identifiers/strings, numeric/runtime guards, negative diagnostics, empty source, duplicate definitions/parameters, return/operand/arity typing, immutable assignment, `Never` joins, recursion/step limits, deterministic properties, and all four fuzz targets passed; two control-flow defects were found and regression-fixed |
| H18 — Hydra 0.1 release candidate | COMPLETE | Closeout commit `4809388` was pushed to `main`; GitHub Actions CI run `37678326927` completed successfully after all local release-candidate gates passed |

## Current validation floor

- `cargo fmt --all -- --check` — PASS
- `cargo clippy --workspace --all-targets -- -D warnings` — PASS
- `cargo test --workspace --all-features` — PASS
- `cargo +1.85.0 check --workspace` — PASS
- `cargo +nightly fuzz build` — PASS
- bounded fuzz smoke — PASS: lexer 1000, parser 1000, compile 500, runtime 100 runs
- `hydra check examples/hello.hyd` — PASS
- `hydra run examples/hello.hyd` — PASS, output: `Hydra`

Hydra 0.1 foundation is complete. No tag or release was created as part of this gate.
