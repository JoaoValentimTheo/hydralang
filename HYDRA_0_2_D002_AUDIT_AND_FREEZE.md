# HYDRA 0.2 D002 — Independent adversarial audit and conditional semantic freeze

## Identity, scope and freeze condition

- Audit date: **2026-10-08**; branch: `main`.
- Audited implementation baseline: **`22f66b354b557d4cd58321ae6bb17cd4457da99a`**, identical to `origin/main` with a clean tracked/index/untracked worktree before this audit.
- Baseline implementation GitHub Actions: [run 37855256644](https://github.com/JoaoValentimTheo/hydralang/actions/runs/37855256644), five jobs successful on the implementation SHA. This is historical prerequisite evidence, **not** final freeze-commit CI evidence.
- Governing contract: `docs/decisions/002-tuples.md`, `spec/{LEXICAL_GRAMMAR,GRAMMAR,TYPE_SYSTEM,EXECUTION_MODEL}.md`, `docs/ERROR_CODES.md`, `docs/STATE_MACHINE.md`; historical approval: `HYDRA_0_2_D002_ACCEPTANCE.md`; implementation evidence: `HYDRA_0_2_D002_IMPLEMENTATION.md`.
- Frozen predecessor: D001, documented in `HYDRA_0_2_D001_FREEZE.md`. The 0.1 baseline and D001 decisions remain unchanged. No changes to production code, tests, dependencies, fuzz seeds or CI configuration were required by this independent review.
- **Decision: local adversarial audit PASSED; D002 technical freeze becomes effective only after the final documentation commit is pushed as an ordinary fast-forward and all five GitHub Actions jobs report SUCCESS with `head_sha` exactly equal to that final commit.** Until then its status is **AUDITED / FREEZE PENDING FINAL CI**. The final SHA, run URL and conclusions must be captured from the actual remote run in the closure report; this prepublication record does not claim them.
- Freeze is a semantic-governance checkpoint, not a version tag, release, publication of Hydra 0.2, or a claim of exhaustive correctness. D003 and List/Set/Array, patterns, modules and generics remain deferred.

## Methodology and contract-to-implementation traceability

Three independent-review roles were executed sequentially: **A**, lexer/parser/resolver/checker/HIR/diagnostic ownership; **B**, runtime type/value integrity, NaN, shared-DAG equality, resource budgets and D001 effects; **C**, normative records, corpus, property suites, compatibility, tools and performance. Findings were evaluated against the already accepted Option A; no new semantic contract was introduced. Direct CLI falsification used temporary files, 15-second subprocess timeouts, and explicit expected diagnostics or values. Source and tests were inspected in addition to executing the standard matrix.

| Normative invariant | Owning implementation | Independent behavior / test evidence | Status |
| --- | --- | --- | --- |
| `()` is Unit; `(x)` groups; `(x,)`, `(x,y,)` form ordered tuples; grouped/tuple type distinction | `hydra-parser`, `hydra-ast` | `d002_construction_projection_annotations_and_equality`, positive `d002_structural_roundtrip.hyd` | PASS |
| `1.25` stays Float; `t.0.1`, `t . 0`, `.00` remain chained/index projections with local lexer state | `hydra-lexer`, parser | `tuple_projection_digits_do_not_consume_following_dot_or_change_floats`; `dot_mode_is_reset_by_newline_and_independent_numbers`; direct CLI probes | PASS |
| Postfix projection precedence, newline barrier, malformed syntax, synthetic EOF and parser progress | `hydra-parser` | `d002_diagnostics_have_precise_utf8_spans_and_recover_deterministically`; direct newline-before-dot probe | PASS |
| Maximum tuple arity 64; 65 is parser E1101; syntax 128, expression 256 | `hydra-parser`, AST depth walker | `d002_arity_and_nesting_exact_boundaries`, `d002_generated_arity_annotation_projection_and_bounds`, parser recovery regressions | PASS |
| Written tuple-type depth 64 accepted, 65 parser E1105 | `hydra-parser` | `d002_arity_and_nesting_exact_boundaries`, `d002_nested_function_argument_return_and_projection_respect_type_depth` | PASS |
| Inferred tuple depth 64 accepted, 65 checker E3014 with construction span; shared type DAG bounded | `hydra-types`, `hydra-check` | `d002_inferred_depth_64_succeeds_and_65_is_checker_owned`, `d002_inferred_tuple_dag_does_not_expand_types_exponentially` | PASS |
| Tuple children/bases resolve in existing scopes; no new writable projection target | `hydra-resolve`, `hydra-parser` | End-to-end CLI construction, `d002_projection_assignment.hyd` negative corpus | PASS |
| Ordered structural type identity, arity sensitivity, hash consistency, exact inference and parameters/returns; no coercion | `hydra-types`, `hydra-check` | Type identity/hash unit cases, `d002_construction_projection_annotations_and_equality`, `d002_type_order.hyd` negative corpus and direct swapped-type probe | PASS |
| `Never` normal-value bottom remains distinct from frozen D001 Return/Break/Continue outcomes, including unreachable checking | `hydra-check`, `hydra-hir` | `d002_strict_fields_stop_after_return_break_continue_or_error`, existing frozen D001 regression suite | PASS |
| Static non-tuple E3012, out-of-bounds E3013; E3002/E3003/E3004 mismatches, E3007 whole-tuple print | `hydra-check`, `hydra-stdlib` | `d002_diagnostics_and_limits`, precise UTF-8 span tests, classified negative corpus, direct `.02` and `<` probes | PASS |
| Completed element snapshots, immutable shared tuple storage; left-to-right construction stops at first effect/error | `hydra-hir`, `hydra-runtime` | `d002_snapshots_float_and_strict_d001_effects`, `d002_strict_fields_stop_after_return_break_continue_or_error`, positive `d002_snapshot.hyd` | PASS |
| Static positional projection with full base shape validation; internal corrupt tuple fields, arguments, return types, nested HIR report E9004 | `hydra-runtime` | `malformed_hir_projection_rejects_corrupt_unselected_fields`; malformed-HIR tuple, function boundary and depth runtime unit regressions | PASS |
| Structural ordered `==` and complementary `!=`, IEEE NaN even with physically identical/shared tuple nodes | `hydra-runtime` | `tuple_alias_nan_never_becomes_reflexively_equal`, `d002_nan_equality_symmetry_complement_and_nested_sharing`, direct shared-nested-NaN probe | PASS |
| Pair-identity-memoized iterative shared-DAG equality, short-circuit and charged comparison/field work; no recursive tuple formatting | `hydra-runtime` | `shared_dag_equality_short_circuits_without_exponential_expansion`, `tuple_equality_consumes_existing_fuel_and_not_equal_is_complement`, runtime host-display regression | PASS |
| Existing one-million-step budget raises E4006, call-depth 128 and restoration after errors | `hydra-runtime` | Fuel exhaustion, malformed callee/effect and stack-boundary runtime regressions; `ci-macos.sh` | PASS |
| D001 effects, Hydra 0.1 arithmetic/Float/grouping/scopes/calls/statement boundaries preserved | All compiler/runtime phases, CLI | Full workspace tests, 34 positive / 55 negative classified programs, public CLI hello check/run, direct semicolon/Float probes | PASS |

Traceability statuses refer to bounded observed tests and source inspection, not proofs over all hypothetical programs. In particular, 64/65 tuple nesting is enforced statically for source-derived checked HIR, with E9004 a defensive internal runtime failure for corrupted HIR rather than a source diagnostic.

## Falsification probes and findings

Eight additional temporary-file CLI probes, executed with a 15-second timeout per process, all matched the contract: (1) spaced and chained projection with an independent `1.25` float; (2) leading-zero projection `.00`; (3) shared nested NaN returned `== false` and `!= true`; (4) out-of-range `.02` reported E3013; (5) swapped structural type order reported E3002; (6) statement semicolon reported E1001; (7) newline before a dot reported parser E1101; (8) tuple ordering `<` reported E3003. They are **eight exploratory CLI probes**, not eight additional committed tests.

| Finding / hypothesis | Severity | Evidence | Resolution | Status |
| --- | --- | --- | --- | --- |
| Critical D002 defect | CRITICAL | No such defect reproduced by bounded adversarial source/runtime tests | No production change warranted | No confirmed finding |
| Incorrect D002 semantics, explosive shared-DAG equality, NaN short-circuit, or malformed HIR escape | HIGH | Existing targeted runtime tests, type-DAG tests and direct NaN probe passed | Hypotheses falsified for tested cases; no production change | No confirmed finding |
| Diagnostic ownership, tuple-type identity, invalid projection, Float ambiguity, effects, compatibility | MEDIUM | Classified corpus, 10 D002 CLI tests, diagnostics/UTF-8 tests and eight direct probes passed | No correction indicated | No confirmed finding |
| Acceptance wording says implementation not started | LOW (historical wording) | Acceptance checkpoint predates separate implementation authorization; the decision file includes an explicit later Option A note | Preserve the original decision; current status belongs in this audit and `AGENT_STATE.md` | Explained; not a semantic defect |

No further speculative refactoring, new tests, or expanded fuzz campaign was justified by this finite audit. Existing test cases already exercise the relevant threats with deterministic oracles. The implementation ledger records previously fixed D002 prepublication defects; their regressions were re-executed and remain green. They are **not** new adversarial audit findings.

## Reproducible local validation and bounded testing

All figures below refer to the implementation baseline above on **macOS 27.0.1 arm64**, Rust stable **1.99.0** and supported MSRV **1.85.0** on 2026-10-08. This documentation-only audit introduces no source/test changes; final CI must independently validate the published documentation SHA.

| Gate / unit of evidence | Observation |
| --- | --- |
| `cargo +stable fmt --all -- --check` | PASS |
| `cargo +stable clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo +stable test --workspace --all-features` | **79 Rust test functions passed / 0 failed**, 0 doctests; `/usr/bin/time -p` real **2.75 s** |
| D002-specific CLI tuple integration tests | **10** test functions within the 79, not additive |
| Deterministic property tests | **10** test functions within the 79; generated examples are not separately counted as Rust tests |
| Classified corpus | **34 positive** and **55 negative** `.hyd` programs, exercised within existing Rust corpus functions; not additive to 79 |
| `cargo +1.85.0 check --workspace` | PASS |
| `./scripts/ci-macos.sh` | PASS, including Rust tests and `examples/hello.hyd` CLI check/run (`Hydra`) |
| `cargo +nightly-2026-10-02 fuzz build` | PASS on all four existing fuzz targets; pinned `cargo-fuzz 0.13.2` |
| Pinned bounded fuzz smoke, copied isolated seed trees | Lexer **256**, parser **256**, compile **128**, runtime **64** = **704 executions**, all PASS; `-runs=N -max_total_time=25`; seed copies and logs at `/tmp/hydra-d002-audit-fuzz.QiwYNA/` |
| Stored fuzz seed inventory | **11 D002** seeds in existing four target seed directories; no new seeds added during audit |
| Eight independent bounded CLI probes | **8/8** matched expected value/diagnostic (15-second subprocess timeout per probe) |
| `git diff --check`, `git diff --cached --check` on starting checkout | PASS; starting worktree/index clean |

Same-command Rust-test baseline wall times before this audit were approximately **2.39–2.66 s**; current **2.75 s** is a small run-to-run difference without a test-suite modification. No significant test-suite growth was introduced. The four fuzz smoke counts measure *libFuzzer executions*, not comprehensive coverage, property cases, corpus programs or Rust test functions. Each smoke uses bounded inputs/seeds and ends after a fixed number of runs. Heavy adversarial stress belongs to a separate explicitly invoked, resource-limited gate; none was added to default CI.

## Security, state-machine compatibility and remaining limitations

- The runtime stores constructed tuples in immutable `Rc<[Value]>`; cloning a tuple root shares storage. Equality visits tuple-node pairs iteratively and memoizes pair identity only after scheduling descendants, including scalar NaN comparisons. Tuple pair/field work consumes interpreter fuel; lack of a pointer-equality shortcut for NaN-bearing aggregates is essential.
- Invalid runtime tuple/value types are checked across nested fields, including *unselected* projection fields and declared function parameters/returns, with E9004 on detected malformed typed HIR. Source-level projection OOB is checker-owned E3013.
- The parser bounds recursive syntax (128), expression-tree depth (256), tuple arity (64) and explicit tuple-type nesting (64); the checker separately bounds inferred tuple-type nesting at 64. D001's distinct outcomes and strict control-effect propagation remain in use. Interpreter calls are bounded at depth 128 and execution at 1,000,000 steps.
- `docs/STATE_MACHINE.md` retains the inherited parser progress/depth restoration, per-function resolver/checker context resets, runtime `Flow` effect unwinding and call-depth restoration. No new general FSM architecture was introduced.
- Bounded fuzz smoke and ordinary tests do **not** establish exhaustive language or memory-safety proof. Source programs cannot build cycles in tuple storage; arbitrary externally fabricated deeply cyclic or malformed HIR lies outside the source-language trust boundary, and no universal host allocation/stack-safety guarantee is asserted. Host resource limits remain distinct from deterministic interpreter fuel.
- The only D002 amendment is the already human-approved Option A: explicit depth 65 E1105, inferred depth 65 E3014, malformed typed HIR E9004. Prior acceptance, D001 freeze records and implementation-commit identity are retained verbatim as historical evidence.

## Final freeze gate

The local evidence satisfies the applicable source, correctness, performance, compatibility and scope gates without adding unrelated work. Publication shall consist only of this audit record and minimal current-state reconciliation. Before fast-forward push, review the complete staged diff, verify clean scope, fetch and confirm no remote advancement. **The binding freeze event is the successful GitHub Actions run on the exact resulting commit SHA with all five jobs (`fmt + clippy`, Linux `test`, Windows `test (windows-latest)`, `MSRV 1.85`, `bounded fuzz smoke`) reporting SUCCESS.** If any final job fails or exact-SHA verification is unavailable, D002 remains AUDITED / **UNFROZEN** until that precise blocker is resolved.

After verified exact-SHA success, status is **D002 — ACCEPTED / IMPLEMENTED / ADVERSARIALLY AUDITED / FROZEN**. Hydra 0.2 remains **IN PROGRESS**, with no D003, family expansion, tag or release authorization. Stop this bounded audit at that checkpoint.
