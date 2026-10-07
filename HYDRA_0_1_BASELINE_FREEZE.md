# Hydra 0.1 Baseline Freeze Ledger

This ledger records the Hydra 0.1 baseline-freeze, state-machine-contract, and governance campaign. It does not change the historical meaning of `HYDRA_0_1_LEDGER.md` or `HYDRA_0_1_POST_FOUNDATION.md`. Hydra 0.2 implementation was not started and H15 Collections remains deferred.

## Starting point

- Branch: `main`
- Authoritative pre-freeze checkpoint: `2a4ea08e3b3856437ba2ccac2ebb776e374ecf92`
- `origin/main` at campaign start: same commit
- Prior CI: run `37687636620`, successful
- Tags at campaign start: none
- GitHub releases at campaign start: none
- Workspace: 12 crates; version `0.1.0-dev`; Rust edition 2024; MSRV 1.85.0
- Program corpus: 22 pass programs and 41 classified failure programs
- Examples: 14 `.hyd` programs plus `examples/README.md`
- Fuzz targets: lexer, parser, compile pipeline, runtime

The frozen semantic implementation commit is recorded by the closeout update in `docs/BASELINE.md`. A commit cannot truthfully contain its own hash, so the implementation freeze and the hash-recording closeout are intentionally separate commits.

## Gate evidence

| Gate | Status | Evidence |
| --- | --- | --- |
| G0 authoritative baseline | COMPLETE | `main`, local HEAD and `origin/main` all matched `2a4ea08e...`; prior CI, tags, releases, workspace metadata, corpus, examples, fuzz targets, specs and ledgers were inspected before edits. |
| G1 agent state | COMPLETE | `AGENT_STATE.md` now describes the baseline-freeze campaign, corpus/examples, pinned fuzz toolchain, platform gate, structural risks, deferred families and next permitted action. |
| G2 state-machine contract | COMPLETE | `docs/STATE_MACHINE.md` documents parser progress/recovery/depth, resolver/checker per-function state, value/Unit/Never/Return/error distinctions, call lifecycle and execution budget. Regressions cover missing EOF recovery, cross-function state isolation and call-depth restoration. |
| G3 builtin identity | COMPLETE | `BuiltinId` is owned by `hydra-stdlib`; resolver/HIR/runtime use stable identity; the registry owns names/signatures; runtime dispatch is exhaustive by ID and validates HIR arity. Registry round-trip/coverage and malformed-HIR arity tests prevent silent drift. |
| G4 semantic baseline | COMPLETE | `docs/BASELINE.md` indexes the accepted 0.1 surface, primitive semantics, lookup/mutability/Never policy, builtins, structural/resource limits and deferred surface. |
| G5 versioning policy | COMPLETE | `docs/VERSIONING.md` separates the `0.1.0-dev` engineering baseline from any public compatibility promise and defines the required process for observable semantic changes. |
| G6 language-change protocol | COMPLETE | `docs/decisions/README.md` and `docs/decisions/000-template.md` require problem/alternatives/chosen semantics/phase impacts/diagnostics/tests/compatibility for language-surface decisions. |
| G7 roadmap | COMPLETE | `docs/ROADMAP.md` defines a design/decision gate followed by one coherent implementation family per campaign. |
| G8 README | COMPLETE | README distinguishes the validated engineering baseline from a public tag/release and documents both parser structural limits. |
| G9 fuzz reproducibility | COMPLETE | CI and `fuzz/README.md` pin `nightly-2026-10-02` and `cargo-fuzz 0.13.2`; advancing the nightly requires a dedicated maintenance change. |
| G10 platform policy | COMPLETE | `docs/PLATFORMS.md` defines Linux quality/tests/MSRV/fuzz and stable full-workspace tests on macOS and Windows; CI contains the corresponding host matrix. |
| G11 license presentation | COMPLETE LOCALLY | GitHub classified the pre-fix repository as `Other` / `NOASSERTION`. `LICENSE-APACHE` contained only the short notice; it was replaced with the canonical complete Apache License 2.0 text while preserving `MIT OR Apache-2.0`. Remote classification is rechecked after push. |
| G12 HIR extensibility | COMPLETE | `docs/HIR_EXTENSIBILITY.md` records confirmed pressure for control effects, collections, nominal identity/modules and optional future lower IR without implementing speculative abstractions. |
| G13 0.2 decision queue | COMPLETE | `docs/HYDRA_0_2_DECISION_QUEUE.md` is explicitly design-only and separates List, Tuple, Set and optional fixed Array semantics. H15 remains deferred. |
| G14 local validation | COMPLETE | All required local gates listed below passed before the freeze commit. Remote cross-platform CI is a final closeout condition after push. |
| G15 adversarial review | COMPLETE | Governance mechanisms below materially block the failure modes named by the campaign. |

## Defects and architectural findings

The campaign found one critical source-derived structural defect. A very long left-associative expression could construct a deeply left-heavy AST while remaining shallow in Pratt-parser recursion. A 20,000-addition reproducer could reach later recursive phases and abort the host process. Hydra now enforces an iterative `MAX_EXPR_DEPTH = 256` check and reports E1106 before the AST reaches those phases. This is distinct from the existing `MAX_PARSE_DEPTH = 128` recursive syntax guard (E1105). The regression `rejects_pathological_left_deep_expression_before_host_stack_exhaustion` protects the boundary.

The builtin boundary also carried avoidable string-identity drift risk. The minimal refactor introduces `BuiltinId::{Print, Println}`, keeps name/signature ownership in `hydra-stdlib`, lowers stable identity through resolver and typed HIR, and dispatches exhaustively by ID in the runtime. No plugin framework or duplicated registry was introduced.

Resolver/checker/runtime state transitions were hardened with targeted regressions: lexical/function state cannot leak to a subsequent function, a checker error or diverging function cannot poison the next function, malformed token streams without physical EOF terminate, and runtime call depth returns to zero on both successful and diagnostic exits.

The final adversarial review also found a contract-order mismatch in user-function calls: the runtime checked the call-depth limit before function lookup and HIR arity validation even though the documented lifecycle requires lookup -> arity invariant -> depth guard. The implementation was reordered to match the state-machine contract. Builtin execution also now extracts arguments inside each exhaustive `BuiltinId` arm after authoritative signature validation, avoiding a hidden global assumption that every future builtin has one argument.

## Local validation

The following completed successfully on the campaign worktree:

```text
cargo +stable fmt --all --check
cargo +stable clippy --workspace --all-targets -- -D warnings
cargo +stable test --workspace --all-features
cargo +1.85.0 check --workspace
cargo fuzz --version                         # cargo-fuzz 0.13.2
cargo +nightly-2026-10-02 fuzz build
cargo +nightly-2026-10-02 fuzz run lexer fuzz/seeds/lexer -- -runs=256
cargo +nightly-2026-10-02 fuzz run parser fuzz/seeds/parser -- -runs=256
cargo +nightly-2026-10-02 fuzz run compile fuzz/seeds/compile -- -runs=128
cargo +nightly-2026-10-02 fuzz run runtime fuzz/seeds/runtime -- -runs=64
cargo run -p hydra-cli -- check examples/hello.hyd
cargo run -p hydra-cli -- run examples/hello.hyd
git diff --check
```

Focused regressions for E1106, missing-EOF recovery, resolver/checker function isolation, call-depth restoration, and builtin registry identity also passed independently before the full matrix.

Local libFuzzer runs add discovered inputs to a supplied corpus directory. Those generated inputs and the incidental local `fuzz/Cargo.lock` change were removed/restored after validation because they were validation artifacts, not reviewed corpus additions.

## G15 adversarial answers

- A future feature cannot legitimately change accepted semantics without the versioning rule, a decision record, specification/regression updates, compatibility classification and full validation.
- Stdlib/runtime identity drift is constrained by one `BuiltinId` universe, authoritative signatures, HIR identity and exhaustive runtime dispatch; arity disagreement is diagnosed as invalid HIR.
- Parser, resolver/checker, control-flow, call and execution-budget state transitions have an explicit contract plus regressions at the previously weak boundaries.
- The README/versioning/baseline documents explicitly distinguish an engineering baseline from a public release; no tag/release is implied.
- The roadmap and 0.2 decision queue require one coherent feature family per implementation campaign.
- CI fuzz behavior no longer follows a moving nightly; both nightly date and `cargo-fuzz` version are pinned.
- `AGENT_STATE.md`, `docs/BASELINE.md` and this campaign ledger provide repository-local state checkpoints; a future semantic campaign must update the governing documents as part of its gate.

## Scope result

No List, Set, Tuple, Array, collection implementation, break/continue, structs/enums, modules/imports, generics, or other Hydra 0.2 language family was implemented. The only compiler changes are robustness/state-machine hardening and the stable builtin-identity boundary required by this freeze. H15 Collections remains deferred.

The final synchronized HEAD, final remote CI run and final GitHub license-classification observation are reported after the closeout commit and push; they are deliberately not fabricated into a commit that precedes those observations.
