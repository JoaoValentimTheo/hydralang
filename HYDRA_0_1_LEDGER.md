# Hydra 0.1 Ledger

| Milestone | Status | Evidence |
| --- | --- | --- |
| H0 — Repository constitution | IN PROGRESS | Rust workspace, constitution/spec docs, licenses, contribution guide, state ledger, and GitHub Actions workflow exist; remote repository/CI run still pending |
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
| H11 — Integration corpus | IN PROGRESS | Executable hello example and recursive fib integration-style test exist; classified `tests/programs` corpus and negative corpus still missing |
| H12 — Property tests | NOT STARTED | No property-test framework or property suite yet |
| H13 — Fuzz foundations | NOT STARTED | No buildable fuzz package/targets yet |
| H14 — Standard library foundation | COMPLETE | Central builtin/signature registry provides `print` and `println` to resolver/checker/runtime |
| H15 — Collections | NOT STARTED | Deferred until the core validation floor is stronger |
| H16 — Documentation reconciliation | IN PROGRESS | Language/spec/grammar/type/execution/error/architecture/roadmap docs exist; final adversarial reconciliation remains |
| H17 — Hydra 0.1 adversarial review | NOT STARTED | Final malformed/deep/numeric/diagnostic attack pass not yet performed |
| H18 — Hydra 0.1 release candidate | NOT STARTED | H11-H13, H16-H17 and release-candidate gates remain |

## Current validation floor

- `cargo fmt --all -- --check` — PASS
- `cargo clippy --workspace --all-targets -- -D warnings` — PASS
- `cargo test --workspace --all-features` — PASS
- `cargo +1.85.0 check --workspace` — PASS
- `hydra check examples/hello.hyd` — PASS
- `hydra run examples/hello.hyd` — PASS, output: `Hydra`

Hydra 0.1 is not a release candidate yet.
