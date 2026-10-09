# HYDRA 0.2 D003 — Immutable Homogeneous List Implementation

**Status:** IMPLEMENTED AND LOCALLY VALIDATED; INDEPENDENT ADVERSARIAL AUDIT REQUIRED. This report records production implementation evidence, not a D003 freeze or Hydra 0.2 release.

## Authorization and baseline

- Normative authority: `HYDRA_0_2_D003_ACCEPTANCE.md`, `docs/decisions/003-list.md` (accepted addendum), the D003 sections of `spec/`, and `docs/ERROR_CODES.md`.
- Initial local `main` and `origin/main`: `1d059cff6bb55614029ca6b9b9cc34ade0d6427b`. Work started from recovered, valuable, uncommitted compiler and regression-test changes; these changes were preserved.
- Historical Hydra 0.1, D001 and D002 freezes remain untouched. This implementation was separately authorized after the historical specification-only acceptance; the acceptance ledger is intentionally left unchanged.

## Acceptance-to-implementation matrix

| Obligation | Implementation | Executable evidence |
| --- | --- | --- |
| `List<T>` exact homogeneous structural type; arity one | `hydra-types`, parser type application, checker E3017, typed HIR | `d003_index_diagnostics_and_type_errors`, D003 negative corpus |
| `[]`, singleton, multiple items, trailing comma; 256-element ceiling | Parser List literals, checker exact element typing; element 257 parser E1101 | `d003_list_element_count_boundary`, positive corpus |
| Five narrowly scoped contextual empty List positions | Checker explicit expected-type entry points and immediate aggregate-child descent | `d003_empty_lists_in_all_five_contexts_and_context_isolation`, bare `[]` E3015 |
| Mixed List/Tuple structural depth up to 64 | Parser written-depth E1105; checker E3016 for inferred aggregates containing List, frozen E3014 for tuple-only | `d003_written_and_inferred_depth_boundary`, `d003_mixed_inferred_list_tuple_depth_64_and_65`, D002 tuple tests |
| Readonly postfix indexing and type diagnostics | AST, parser, resolver, checker, HIR; E3018/E3019, indexed assignment E1104 | `d003_literals_index_chains_and_snapshots`, `d003_index_diagnostics_and_type_errors` |
| Ordered evaluation and D001 Never/control-flow propagation | Checker path outcomes; runtime List construction and index flows | `d003_left_to_right_short_circuit_d001_effects`, frozen D001 tests |
| Immutable shared snapshot values, mixed equality, NaN and bounded DAG work | `Rc<[Value]>`, iterative visited-pair equality, existing interpreter fuel | `d003_equality_preserves_nan_in_shared_mixed_dags`, runtime DAG and malformed-HIR tests |
| Runtime bounds and malformed internal HIR | E4007 on complete source index-operation span; E9004 for invalid List/index/type shapes | `d003_bounds_error_spans_complete_index_operation`, runtime E9004 regressions and negative corpus |
| UTF-8 diagnostics and parser recovery | Source-spanned diagnostics and bounded bracket recovery | `d003_utf8_index_diagnostic_and_malformed_brackets_recover`, prior UTF-8 property tests |
| Existing languages and platform compatibility | Additive List paths; unchanged existing test/corpus contracts | Full workspace tests, macOS gate, Linux/Windows CI after publication |

## Regressions and notable safeguards

- The out-of-bounds runtime E4007 diagnostic points to the **complete** indexing expression (`xs[2]`), including its UTF-8-safe span.
- Empty Lists do not bypass internal aggregate type-depth validation simply because they have zero runtime children: a fabricated depth-65 List type yields E9004; depth 64 succeeds.
- Mixed `List`/`Tuple` shared DAG equality is iterative and memoized, charges the existing execution budget, and does not use pointer equality to incorrectly equate shared NaN leaves.
- Indexing rejects malformed internal typed HIR and rejects out-of-bounds source-derived index values; no implicit aggregate printing or mutation was introduced.
- Additional deterministic regressions verify UTF-8 index error ownership, malformed-bracket recovery and the mixed inferred depth 64/65 boundary.

## Validation ledger (2026-10-09)

- Rust: **92 test functions passing** in `cargo +stable test --workspace --all-features`, including **10** D003 CLI integration tests and runtime D003 safety/equality tests.
- Classified corpus: **37 passing + 64 failing = 101 programs**, comprising **3** new D003 passing programs and **9** new D003 negative programs (parser/type/runtime).
- Formatting: `cargo +stable fmt --all -- --check` — PASS.
- Static quality: `cargo +stable clippy --workspace --all-targets -- -D warnings` — PASS.
- MSRV: `cargo +1.85.0 check --workspace` — PASS.
- Native macOS: `./scripts/ci-macos.sh` — PASS, including CLI hello check/run.
- Bounded fuzzing: pinned `nightly-2026-10-02`, `cargo-fuzz 0.13.2`; lexer **256**, parser **256**, compile **128**, runtime **64** runs (**704 total**), with no crashes reported. Four authored D003 seed programs cover these targets.
- Git whitespace validation: `git diff --check` and staged `git diff --cached --check` — PASS.

The checked source-language contract enforces a List literal size of 256, a combined aggregate type depth of 64, an existing 128-frame call limit, and an existing 1,000,000-tick interpreter budget. Fuel accounting is **not** a universal bound on process memory. Fabricated cyclic HIR/value graphs outside the checked source trust boundary are not covered by the source-language guarantee.

## Scope, pending assurance and governance

No Set, Array, slice, indexed assignment, mutator, iteration, collection printing, general generics, or new semantic family is introduced. No version tag or release is authorized. The normal post-publication GitHub Actions verification must cover **all five jobs on the exact pushed commit SHA**. Publication and CI evidence belong to the final operator report and may be appended here after successful CI.

**Independent adversarial D003 audit remains a mandatory separate gate. D003 is NOT FROZEN. D004 is NOT AUTHORIZED.**
