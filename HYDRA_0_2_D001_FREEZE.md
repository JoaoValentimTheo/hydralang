# Hydra 0.2 D001 — Post-implementation audit and semantic-family freeze

Date: 2026-10-08. Scope: **D001 (`break` / `continue`) only**.

## Identity and governance

- Accepted normative decision: `docs/decisions/001-loop-control.md`, introduced at `130ccf5f2ed06fd7bdcef5b62a00ccbc37323f64`; its `spec/` and `docs/` contract is locked.
- Accepted implementation baseline: `130ccf5f2ed06fd7bdcef5b62a00ccbc37323f64`.
- Implemented and audited semantic source: `f5bb9e6bd8b8befde6bdc79d290983b0ba136d7b`; this was also the clean `main == origin/main` audit starting SHA.
- Audit test-evidence commit: `8ab976c7abaa61b1c2e32e753834150a15575664` (additional regression tests only; no production changes).
- The audit freeze/state commit is the commit containing this record and the associated `AGENT_STATE.md` update. Its exact identity and matching remote CI run must be verified after the ordinary fast-forward push.
- Hydra 0.1: **PRESERVED**. Hydra 0.2: **IN PROGRESS**. D001: **ACCEPTED / IMPLEMENTED / POST-IMPLEMENTATION AUDIT PASSED / FROZEN**. D002: **NOT AUTHORIZED**. Next family: **NOT SELECTED**. H15 Collections: **DEFERRED**.

## Normative requirement → implementation → executable evidence

The rightmost column records observations on the audited tree; tests asserting compiler diagnostics verify codes and keyword spans, not just compilation failure.

| D001 requirement | Implementation | Executable independent evidence | Result |
| --- | --- | --- | --- |
| Reserved, value-less keywords; near identifiers preserved | `crates/hydra-lexer/src/lib.rs` tokenization; `crates/hydra-ast/src/lib.rs` spanned statements | Lexer keyword/near-name test; CLI invalid-name matrix; pass corpus | PASS |
| Bare statement boundaries, attached operand rejection (E1101) | `crates/hydra-parser/src/lib.rs` statement parser and `require_line_boundary` | Parser recovery/span unit test; CLI invalid-control matrix; negative corpus | PASS |
| Progress/EOF/recursive depth safety | Parser recovery guard, synthetic EOF, `MAX_PARSE_DEPTH = 128`, `MAX_EXPR_DEPTH = 256` | Parser EOF/recovery/depth tests; generated parser properties; corpus depth test | PASS |
| Resolver owns no loop placement error or new binding | `crates/hydra-resolve/src/lib.rs` statement traversal | CLI E3010/E3011 single-owner cases; resolver scope isolation unit tests | PASS |
| Loop depth per function; own `while` condition does not establish a target | `crates/hydra-check/src/lib.rs` function reset and `check_stmt` | CLI invalid own condition/function barrier; nested-condition positive corpus and runtime matrix | PASS |
| Nearest eligible body owns Break/Continue; condition targets outer body | Checker lexical depth; `crates/hydra-runtime/src/lib.rs` while condition/body dispatch | Nested-loop body and nested-condition CLI tests, including 3-level condition break; generated depth 1–24 property | PASS |
| E3010/E3011 keyword-primary spans, even in unreachable code and UTF-8 source | Checker statement diagnostics, source spans | CLI invalid matrix, unreachable-outside-loop cases, new UTF-8-offset cases | PASS |
| `Never` is normal-value bottom, independent of Return/Break/Continue | Checker `Outcome`, `check_block`, `check_expr`, `Type::join` | Checker unit tests, CLI never/return/initializer/assignment/branch/call matrix | PASS |
| Five reachable-effect dimensions and fallthrough-gated sequencing | `Outcome::{then,either}`, `block_outcome`, `stmt_outcome`, `expr_outcome` | Checker unit tests, dead-source type-check regression, conditional/mixed paths in runtime matrix | PASS |
| `while` keeps `Unit` on normal completion; propagates reachable Return and condition effects | Checker `stmt_outcome`, runtime while branch | Nested condition, mixed Return/Break/Continue, `Unit` and return-through-loop CLI cases | PASS |
| Strict evaluation propagates effects left-to-right, skipping later effects and writes | Interpreter `eval_stmt`, `eval_expr` and `eval_block` | Strict call/print/assign/binary/unary/return/if runtime matrix and corpus cases | PASS |
| Boolean `&&`/`||` preserve skipped RHS and conditional normal path | Checker `expr_outcome` and interpreter binary short circuit | Constant skipped/taken RHS plus new nonconstant `flag &&` / `flag ||` CLI cases | PASS |
| Explicit spanned HIR identity; no return/sentinel lowering | `crates/hydra-hir/src/lib.rs` and checker lowering | Runtime source cases, malformed-HIR runtime boundary tests | PASS |
| Function barrier: malformed HIR escapes fail E9004 at original effect span; call depth restored | Runtime `call_function`, `run_main` | Direct malformed-HIR E9004 test and new nested `If`/block/callee escape + subsequent valid call test | PASS |
| Same iteration-boundary fuel tick on normal completion and Continue; Break exits promptly | Runtime `eval_stmt` while body and `tick` | Infinite-Continue E4006 CLI regression; finite break/continue runtime matrix | PASS |
| Preserve Hydra 0.1 except for reserved `break`/`continue` name usages | Lexer/parser keyword decision; existing compiler/runtime | Existing 0.1 corpus, 14 public examples, full workspace regressions, invalid legacy-name tests | PASS |

## Exhaustive state and control-flow review

- Parser: statement keyword consumption precedes boundary validation; `E1101` recovery advances; depth and EOF guards remain. Both new token kinds are recognized before identifier fallback.
- Resolver/checker: no resolver legality error; per-function type and loop state resets; the `while` condition is checked at the enclosing loop depth; only its body increments that depth; subsequent source is checked after non-fallthrough while its effects are gated from reachable summary paths.
- Runtime `Flow::{Value,Return,Break,Continue}`: producers in `eval_stmt`; strict expression operands and block/statement propagation in `eval_expr`/`eval_block`; while **condition** propagates all non-values; while **body** consumes Break/Continue, propagates Return, and ticks after normal/Continue iteration; `call_function` rejects uncaught Break/Continue with E9004 and decrements call depth on both errors and successful execution. `run_main` reports the returned diagnostic. No control effect is silently coerced to Unit/Return.
- Checked all use sites of `TokenKind`, `Stmt`, `HirStmt`, `Flow`, `Type::Never`, `Outcome`, and former single-termination assumptions; no D001-invalid `stmt_diverges`/termination shortcut remains.

## Reconciled counts and validation evidence

- **47 Rust unit/integration tests** after the audit test-only commit (46 before). This is the number of named Rust tests, not a sum with embedded case tables or corpus files.
- Classified corpus: **30 positive `.hyd` files**, **49 negative `.hyd` files**; 8 positive and 8 negative files were introduced for D001. Negative cases are asserted against expected diagnostic codes, and positive cases against exact outputs.
- **8 property-test functions**; the D001 generator covers nesting depths 1–24. Generated inputs are cases within those functions, not additional Rust test functions.
- CLI adversarial case tables: **35 valid runtime cases** and **16 invalid diagnostic cases** (31/14 before audit). Two other D001 Rust tests independently check unreachable type errors and million-step Continue exhaustion.
- New runtime malformed-HIR regression checks **both** Break and Continue through nested blocks/If in a called function, the exact E9004 source span, call-depth restoration, and a successful subsequent call.
- Six hand-selected D001 fuzz seeds across the existing four targets. Additional bounded pinned fuzz smoke preceding the test-only changes: lexer **1,500**, parser **1,500**, compile **850**, runtime **250** = **4,100** executions; all four targets built and ran successfully with `nightly-2026-10-02` / `cargo-fuzz 0.13.2`. After the test-only commit, an additional pinned run passed: lexer **256**, parser **256**, compile **128**, runtime **64** = **704** executions. Original D001 implementation smoke was independently 1,000 / 1,000 / 500 / 100. Generated untracked mutation artifacts were removed; these counts are fuzz iterations, not tests.
- Native macOS `./scripts/ci-macos.sh`: PASS on Darwin arm64, macOS 27.0.1; stable fmt, warning-free Clippy, workspace tests, MSRV 1.85, public CLI hello check/run. The audited production/test code is identified by commit `8ab976c7abaa61b1c2e32e753834150a15575664`; the freeze commit changes documentation/state only.
- Initial implementation GitHub Actions run **37816599694**: 5/5 success (fmt+Clippy, Linux, Windows, MSRV 1.85, pinned fuzz). The freeze SHA must independently pass the same five jobs before final remote closure; the original run alone is not evidence for the freeze SHA.

## Dependency/lockfile forensics

`fuzz/Cargo.lock` gained precisely the `"hydra-stdlib"` dependency entry under workspace packages `hydra-check` and `hydra-runtime`; both crates already declared that local dependency in their manifests before D001. The D001 baseline-to-implementation diff changes no manifest, external dependency version, checksum, or dependency requirement. The lockfile delta reconciles workspace package metadata, not third-party dependency drift. No library/dependency upgrade is attributed to D001.

## Findings register and final verdict

| Classification | Finding | Disposition |
| --- | --- | --- |
| CONFIRMED D001 DEFECT | None demonstrated in production after adversarial audit | Zero production fixes |
| TEST DEFECT / evidence gap | Variable-LHS short circuit, 3-level condition ownership, UTF-8 spans, nested malformed-HIR callee escape not explicitly covered by original D001 tests | Closed by test-only commit `8ab976c`; focused runs green |
| DOCUMENTATION DEFECT | No inaccurate post-implementation assertion requiring a change to locked normative material | None fixed |
| NON-ISSUE / EXPECTED CONTRACT | Locked acceptance/decision prose refers historically to future implementation at acceptance time | Left unchanged to preserve authoritative record |
| NON-ISSUE / EXPECTED CONTRACT | Two additions to `fuzz/Cargo.lock` workspace dependency metadata | Reconciled above; no manifest drift |
| OUT-OF-SCOPE FUTURE DESIGN | D002, next feature family and H15 Collections | Deferred; no design or code started |

The proposed bulk ad-hoc adversarial generator was **not executed**; it is not counted as test evidence. Independently reviewed existing property, corpus, parser-depth, bounded fuzz, and focused new regressions provide the recorded coverage. Test runs demonstrate conformance for the cases exercised; they are not a proof over every possible program.

**Local post-implementation D001 audit: PASSED. Semantic family: FROZEN, subject to verification that the final pushed freeze SHA passes all five GitHub Actions jobs.** The Hydra 0.2 milestone itself remains in progress; this record authorizes no other work, release, or tag.
