# Hydra 0.1 Post-Foundation Audit

This ledger tracks the semantic hardening campaign after completion of the Hydra 0.1 foundation. It does not alter the meaning or status of `HYDRA_0_1_LEDGER.md`. H15 Collections remains deferred.

| Milestone | Status | Evidence |
| --- | --- | --- |
| PF0 — Baseline verification | COMPLETE | Started from `main` at `32d0cdacf5af2c5de389077170a5f1ccd127aefc`; local HEAD equals `origin/main`; worktree clean; no tags or GitHub releases; Rust workspace uses edition 2024, MSRV 1.85, `MIT OR Apache-2.0`; CI contains fmt/clippy, workspace tests, and MSRV jobs; 22 unit/integration `#[test]` functions in `crates/`, 9 positive corpus programs, 22 negative corpus programs, 1 public example, and 4 fuzz targets |
| PF1 — Semantic surface inventory | COMPLETE | Cross-layer audit reconciled specification, lexer, parser/AST, resolver, checker, HIR, runtime, CLI, tests, examples, and documentation for the shipped 0.1 surface |
| PF2 — Public examples expansion | COMPLETE | 14 public `.hyd` examples; integration test discovers the exact set, compiles to HIR, executes each, and checks exact output |
| PF3 — Positive program corpus expansion | COMPLETE | 22 positive corpus programs cover primitives, expressions, multiline syntax, control flow, functions, recursion, numeric edges, scopes, and resource-boundary success cases |
| PF4 — Negative diagnostic corpus expansion | COMPLETE | 41 negative corpus programs cover lexer, parser, resolution, type, and runtime failures with expected phase/code assertions |
| PF5 — Expression and precedence matrix | COMPLETE | Regression corpus verifies normal arithmetic/comparison precedence, left associativity, unary precedence, and `&&` tighter than `||`; semicolon and same-line statement rejection are covered |
| PF6 — Control-flow and Never audit | COMPLETE | Fixed `Never` propagation through strict unary/binary/assignment/condition/call contexts while preserving short-circuit behavior; checker and executable regressions pass |
| PF7 — Function and recursion audit | COMPLETE | Forward calls, parameter/return contracts, Unit returns, recursion, mutual recursion, left-to-right argument evaluation, and diverging arguments are covered |
| PF8 — Numeric semantics audit | COMPLETE | Checked i64 arithmetic, overflow/division/remainder errors, constructible i64::MIN policy, direct-min parser policy, and IEEE Float inf/-inf/NaN behavior are covered |
| PF9 — String and Boolean semantics audit | COMPLETE | Empty/Unicode/escaped strings, string equality, Boolean operators, comparisons, and short-circuit runtime guards are covered |
| PF10 — Resolver/scope/mutability audit | COMPLETE | Lexical shadowing, nested mutation, scope leaks, builtin shadowing, duplicate bindings, and local→function→builtin lookup order are covered |
| PF11 — Parser recovery and malformed-input audit | COMPLETE | Missing EOF, malformed EOF nesting, recovery, syntax-depth guard, generated token streams, and fuzzed malformed grammar paths terminate without panic/hang |
| PF12 — Runtime resource-boundary audit | COMPLETE | Call-depth success/failure boundary and 1,000,000-step execution budget are executable regressions; finite-loop/early-return behavior remains valid |
| PF13 — Diagnostic inventory reconciliation | COMPLETE | Mechanical inventory finds 32 production `E####` codes and 32 documented codes with no missing or stale entries; E9005 is emitted by the CLI invariant path |
| PF14 — Property-test strengthening | COMPLETE | 7 deterministic property tests cover lexer progress/spans, parser termination/determinism, UTF-8 positions, compile determinism, structured programs, and stable edge diagnostics |
| PF15 — Fuzz target review and smoke matrix | COMPLETE | All 4 targets build on nightly; final bounded smoke executed lexer 256, parser 256, compile 128, runtime 64 with no crash, hang, or invariant violation; 8 intentional seeds retained |
| PF16 — CI hardening | COMPLETE | `actions/checkout` SHA-pinned to v4.2.2 commit; stable fmt/clippy/tests and MSRV 1.85 preserved; bounded nightly fuzz-smoke job added |
| PF17 — Documentation/spec reconciliation | COMPLETE | README/spec/docs reconciled for precedence, newline/no-semicolon grammar, Never, assignment Unit, resolver precedence, integer/Float policies, builtins, and resource limits |
| PF18 — Final falsification pass | COMPLETE | 14/14 final checks passed across all required combinations, including nested return/if, while return, recursion+overflow, short-circuit+runtime error, Unicode diagnostics, nested mutability, diverging args, same local names, malformed EOF, parse depth, call depth, and execution budget |
| PF19 — Post-foundation verdict | COMPLETE | Required local gates, examples, fuzz build/smoke, and final falsification are green; GitHub Actions run `37687338037` succeeded on pushed audit HEAD `dbeb810e460c726c5e23336d03896f9844df7874` with fmt/clippy, tests, MSRV 1.85, and bounded fuzz smoke all green |

## Baseline facts

- Branch: `main`
- Starting HEAD: `32d0cdacf5af2c5de389077170a5f1ccd127aefc`
- Starting `origin/main`: `32d0cdacf5af2c5de389077170a5f1ccd127aefc`
- Starting worktree: clean
- Tags at start: none
- GitHub releases at start: none
- CI baseline: `.github/workflows/ci.yml` with fmt/clippy, workspace tests, and MSRV 1.85 jobs
- Workspace members: 12 compiler/runtime/CLI crates listed in the root `Cargo.toml`; `fuzz` is intentionally excluded from the main workspace
- Rust edition: 2024
- MSRV: 1.85
- Declared license: `MIT OR Apache-2.0`; `LICENSE-MIT` and `LICENSE-APACHE` are present
- GitHub license classifier at start: `Other`
- Public examples at start: `examples/hello.hyd`
- Fuzz targets at start: lexer, parser, compile, runtime
- Baseline `cargo test --workspace --all-features`: PASS

## Confirmed defects fixed during this campaign

- Checker: `Never` did not propagate through several strict expression contexts. The checker now propagates bottom type through eager unary, binary, assignment, condition, user-call, and builtin-call contexts while retaining Boolean short-circuit semantics.
- Parser: the `()` Unit literal span covered only the opening parenthesis. It now joins the opening and closing token spans.
- Runtime: malformed typed HIR could call `print`/`println` with extra arguments and silently ignore them. Builtin HIR arity is now exact and violations report E9004.

## Final local validation before remote closeout

- `cargo fmt --all -- --check`: PASS
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS
- `cargo test --workspace --all-features`: PASS
- `cargo +1.85.0 check --workspace`: PASS
- `cargo +nightly fuzz build`: PASS
- bounded fuzz smoke: lexer 256, parser 256, compile 128, runtime 64; no crashes/hangs
- final PF18 falsification matrix: 14/14 checks passed

## Remote closeout evidence

- pushed audit HEAD: `dbeb810e460c726c5e23336d03896f9844df7874`
- GitHub Actions run: `37687338037`
- conclusion: SUCCESS
- jobs: `fmt + clippy`, `test`, `MSRV 1.85`, and `bounded fuzz smoke` all succeeded

## Scope guard

This campaign audits and hardens the existing Hydra 0.1 semantics. Collections and all other 0.2 feature families remain deferred. No tag, release, crates.io publication, or binary release belongs to this campaign.
