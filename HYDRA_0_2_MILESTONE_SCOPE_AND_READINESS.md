# Hydra 0.2 — Milestone Scope and Engineering Readiness

**Assessment:** 2026-10-09. **Branch:** `main`. **Starting SHA:** `a7108d07e5895f2290632607630d369502dde417`. **Disposition:** scope sufficient for a separate **human** technical milestone-closure decision; Hydra 0.2 remains **IN PROGRESS**. This record does not close the milestone, authorize D004, assign a new version, tag or publish a release.

## Decision and evidence boundary

**Recommended outcome: HYDRA 0.2 SCOPE SUFFICIENT — HUMAN MILESTONE CLOSURE DECISION REQUIRED.** D001 loop control, D002 structural tuples and D003 immutable homogeneous lists form a coherent, executable extension of the frozen Hydra 0.1 foundation. The accepted contracts compose across parsing, static types and control effects, typed HIR, runtime and CLI. No reproducible contract violation or demonstrated hard dependency on a fourth family was found in this finite review.

This is a technical-readiness recommendation based on accepted specifications, prior audit ledgers, inspected implementation/tests, fresh local validation and published exact-SHA CI. Green tests and bounded fuzz runs are empirical evidence, not formal proof, a universal resource-safety guarantee or certification for distribution. The D003 adversarial audit used three sequential evidence checklists, not three separately executing reviewer agents or independent human auditors.

## Frozen family checkpoints and compatibility

| Family | Frozen SHA and normative authority | Delivered semantics and evidence | Current state |
| --- | --- | --- | --- |
| D001 — value-less `break` / `continue` | `764b901ec8a0f0febfb3624c392d27f14b631ff4`; `docs/decisions/001-loop-control.md`, `HYDRA_0_2_D001_FREEZE.md` | Reserved keywords, nearest eligible `while` body, inner-condition/outer-body ownership, checker E3010/E3011, distinct Return/Break/Continue/noncompletion effects, function barriers, charged `continue` and internal E9004. Frozen D001 audit: 47 Rust tests; targeted effect/malformed-HIR cases. | **ACCEPTED / IMPLEMENTED / AUDITED / FROZEN** |
| D002 — immutable structural Tuple | `f91d9829c6964fed8e63b97b0a8a05f7a255e58c`; `docs/decisions/002-tuples.md`, `HYDRA_0_2_D002_AUDIT_AND_FREEZE.md` | Ordered heterogeneous fields, exact structural type identity, static `.N` projection, equality preserving NaN, sharing/depth guards and Option A inferred-depth E3014; D002 exact-SHA CI run `37866987551` passed 5/5. | **ACCEPTED / IMPLEMENTED / AUDITED / FROZEN** |
| D003 — immutable homogeneous `List<T>` | `a7108d07e5895f2290632607630d369502dde417`; `docs/decisions/003-list.md`, `HYDRA_0_2_D003_AUDIT_AND_FREEZE.md` | Type-only constructor, homogeneous literals, narrow contextual `[]`, dynamic read-only `[index]`, immutable snapshots/sharing, equality and mixed depth/fuel; malformed Never-index E9004 defect fixed and regression-tested. Final exact-SHA run [`37983625702`](https://github.com/JoaoValentimTheo/hydralang/actions/runs/37983625702) is **SUCCESS, five of five jobs** at the D003 freeze SHA. | **ACCEPTED / IMPLEMENTED / AUDITED / FROZEN** |

Historic acceptance and freeze ledgers correctly preserve wording conditional on later implementation or CI at the time they were written; they were deliberately not rewritten. D001 intentionally reserved identifiers formerly usable as `break`/`continue`, a documented pre-release compatibility break. D002's human-approved Option A amendment distinguishes written depth E1105, inferred tuple depth E3014 and malformed internal HIR E9004. D003's additional syntax is additive within its contract; prior frozen D001/D002 regressions remained green. A frozen family is a semantic checkpoint, not the entire language's completion.

## Implemented language and interface inventory

| Area | Included in the candidate 0.2 milestone | Boundary |
| --- | --- | --- |
| Core values | `Int` (checked signed i64), IEEE-754 `Float`, `Bool`, `String`, `Unit`, and bottom type `Never`; operators, comparisons, boolean short-circuit, strings | No null, implicit numeric coercion, generalized value formatting or polymorphic operators |
| Variables and functions | `let` / `let mut`, lexical scopes, shadowing, reassignment, local inference, explicit function parameter/return types, calls, `return`, expression-valued blocks and `if/else` | No closure or first-class function values, user generic functions, overloading or modules |
| Control | `while`, `return`, and D001 bare `break` / `continue` with checked effects and fuel | No labeled/value-carrying breaks, value-returning `loop` or generic effect system |
| Aggregates | D002 heterogeneous structural Tuple with `.N`; D003 immutable homogeneous `List<T>` literals/read-only dynamic indexing | No Set, fixed Array, List mutation, slice, iteration, append or aggregate-wide `print` |
| Types and syntax | Ordered exact Tuple types; invariant `List<T>` in type positions; 5 narrowly defined contextual expected-type sources for `[]`; Unit/grouping/tuple syntax distinguished | No arbitrary generics or generalized bidirectional inference; newline-separated statements, **no `;` separator** |
| Builtins and CLI | Stable builtin IDs for `print`/`println`; `hydra check` and `hydra run`, validated entry point, source diagnostics, fourteen existing executable examples | No package manager, REPL, LSP, compiler backend, WASM or native artifact toolchain |

Existing 0.1 examples cover primitive functionality and are run against expected output by the corpus suite. D001–D003 source examples are tracked in classified integration corpus rather than being retroactively described as 0.1 examples.

## Engineering readiness across phases

| Phase | Observed guarantee / evidence | Assessment |
| --- | --- | --- |
| Source and lexer | UTF-8 source spans, keyword reservation, bracket/comma/dot disambiguation; existing lexer properties and classified cases | Ready for accepted scope |
| Parser / AST | Newline statement barriers, no semicolon statements; synthetic EOF, recovery progress; expression/syntax nesting guards, List/Tuple source forms and spans | Ready for accepted scope |
| Resolver | Stable IDs, nearest-local / function / builtin lookup, lexical scope/function reset, D001 control traversal without duplicate placement diagnostics | Ready for accepted scope |
| Types / checker | Exact primitive/aggregate identities, D001 path effects independent of `Never`, contextual empty List isolation, homogeneity and aggregate nesting; public E3010–E3019 ownership | Ready for accepted scope |
| Typed HIR | Statically validated aggregate/control forms and builtin identity; malformed internal HIR rejected at runtime with E9004 | Ready for source-derived checked HIR |
| Runtime | Checked arithmetic, dynamic E4007 bounds, ordered single evaluation, immutable shared values, DAG-aware mixed List/Tuple equality, NaN preservation, fuel charging and frame restoration | Ready within documented execution/resource bounds |
| CLI | `check` and `run` commands, entry-point validation, diagnostics and exit paths; existing 14 examples + macOS hello check/run | Ready for defined developer workflow |

The D003 audit's malformed `Type::Never` index failure was a **confirmed HIGH production invariant defect** before the freeze. A red/green runtime regression demonstrates its correction: invalid typed HIR emits spanned E9004; valid source out-of-range indexing keeps E4007. The audit ledger documents the precise trigger and remediation. No **remaining confirmed milestone blocker** was reproduced by this scope review. This conclusion is limited to tested contracts and inspected boundaries; it is not a claim that undiscovered defects do not exist.

### Resource and diagnostic boundaries

Source-level bounds: List literal length **256**, Tuple arity **64**, combined Tuple/List type depth **64**, syntax nesting **128**, expression-tree depth **256**. Runtime: call depth **128**, charged execution fuel **1,000,000** steps. Arithmetic overflow **E4004**, integer zero division **E4005**, exhaustion **E4006**, valid-source List bounds **E4007**, internal invalid HIR **E9004**. Written type depth **E1105**; inferred tuple-only depth **E3014**; inferred mixed depth containing List **E3016**. D003 checker errors **E3015–E3019** cover ambiguous empty literals, depth, constructor arity, index base and index type. Public error-code meanings and phase ownership are documented in `docs/ERROR_CODES.md`.

Immutable `Rc` aggregate sharing and iterative/memoized comparisons avoid exponential revisiting of shared source-derived DAG structures. An interpreter fuel budget is **not** an all-encompassing host memory or allocation limit; externally fabricated cyclic/unbounded Rust HIR/value graphs are outside the documented source trust boundary. Parser and checker structural limits, deterministic diagnostics, UTF-8 span tests, function-context restoration and malformed-HIR regressions address the accepted exposure, with residual unverified host-resource risk retained as future hardening.

## Local and remote evidence (2026-10-09)

| Gate | Evidence on assessed source checkpoint |
| --- | --- |
| Formatting | `cargo +stable fmt --all -- --check`: **PASS** |
| Lint | `cargo +stable clippy --workspace --all-targets -- -D warnings`: **PASS** |
| Full suite | `cargo +stable test --workspace --all-features`: **PASS** (93 Rust test functions in frozen D003 ledger; includes 10 D003 CLI and 10 property test functions, not added a second time) |
| Source corpus | **37 positive + 64 negative = 101** programs, executed in the existing Rust test suite, not 101 additional Rust test functions |
| MSRV | `cargo +1.85.0 check --workspace`: **PASS**; edition 2024, declared MSRV 1.85 |
| Native host | `./scripts/ci-macos.sh`: **PASS** on macOS **27.0.1 arm64**, stable rustc **1.99.0**; repeated format/lint/tests/MSRV and `examples/hello.hyd` `check`/`run` output **Hydra** |
| Pre-existing bounded fuzz evidence | D003 audit: pinned four-target **256 lexer + 256 parser + 128 compile + 64 runtime = 704** executions without recorded crashes; **not rerun** in this docs-only assessment |
| D003 freeze CI | GitHub Actions run **37983625702**, commit **`a7108d07e5895f2290632607630d369502dde417`**: **five successes** (Linux test, Windows test, quality, MSRV, bounded fuzz smoke) |
| Documentation commit CI | The **resulting documentation-only commit** requires its *own* exact-SHA five-job GitHub Actions verification; record SHA/run/results in the final publication handoff, after push. This prepublication report cannot attest to a future run. |

The platform policy uses Linux GitHub Actions for quality/tests/MSRV/pinned fuzz, Windows Actions for all-feature tests and a **native maintainer macOS** script. GitHub-hosted macOS is not a requirement (`docs/PLATFORMS.md`). These are the declared validation platforms; they do not establish packaging or binary-distribution commitments. CLI validation includes a direct successful check and run of `examples/hello.hyd`; broader CLI semantics are covered by the committed integration suites. No additional speculative CLI probe or new test was necessary to close an identified evidence gap.

## Findings, debt, limitations and scope expansion

| Classification | Item | Effect on current gate |
| --- | --- | --- |
| **Confirmed milestone blocker** | **None reproduced** within the accepted D001–D003 scope after the frozen D003 correction | No D004 dependency established |
| **Documentation drift** | `AGENT_STATE.md`, `docs/ROADMAP.md`, `docs/HYDRA_0_2_DECISION_QUEUE.md` still called D003 freeze conditional, even after successful final exact-SHA CI | Reconcile current status to **ACCEPTED / IMPLEMENTED / AUDITED / FROZEN** while retaining Hydra 0.2 **IN PROGRESS** |
| **Non-blocking limitations** | Fixed List and Tuple bounds; readonly indexing and no aggregate-wide printing; no iterators, slicing or mutable collections; finite fuel and call depth | Explicit contract restrictions; not violated promises |
| **Future feature** | Set, fixed Array, modules/imports, ADTs, generic functions, unrestricted type constructors | Independently selectable under a new human design/acceptance gate; no demonstrated milestone prerequisite |
| **Future tooling/release work** | REPL, LSP, packages, optimizers, native/WASM backends, installers and release artifacts | Separate engineering or publication campaign |
| **Unverified risk / future hardening** | New hostile HIR constructions, host allocation behavior, undiscovered semantic edges, expanded OS packaging | Not established as a current production defect by the bounded evidence; defer unless reproducibly linked to accepted source contract |

**D004 necessity assessment:** None of Set, Array, modules, ADTs, generic functions or an additional type constructor is required for `while` loop control, structural Tuple, homogeneous List, static checking or the current interpreter/CLI. Each would add independent grammar, identity, evaluation, diagnostics, state transitions, regressions and CI surface. Set additionally entails block/brace syntax and equality/duplicate policy; fixed Array adds length identity; modules add cross-file namespaces, dependency cycles and initialization; ADTs add variant/constructor identities; generic functions add instantiation/substitution. Increasing the surface now would raise compatibility and validation risk without remedying an observed blocker. **Defer all D004 candidates pending a distinct human scope/design decision.** This is a scope argument, not a completed D004 design gate.

## Governance and requested human decision

Engineering milestone readiness and release readiness have separate owners and checks. `Cargo.toml` continues to declare **`0.1.0-dev`**; this audit authorizes no version bump, compatibility guarantee, release candidate, release, tag, deployment, packaging or channel rollout. The present status remains **Hydra 0.2 IN PROGRESS** until the human owner explicitly authorizes a technical closure campaign. That separate campaign must decide the closure checkpoint/acceptance ledger and whether any independent release program should ever follow.

**One requested decision:** Human owner to **approve or decline initiating a separate Hydra 0.2 technical milestone-closure gate with D001+D002+D003 as the complete accepted 0.2 semantic scope**. If declined, supply a specific reproducible contract gap or explicit business requirement for expanded scope. A vote to initiate technical closure is not consent to tag or ship software. This assessment stops after documentation synchronization and exact-SHA CI verification.

**Verdict: HYDRA 0.2 SCOPE SUFFICIENT — HUMAN MILESTONE CLOSURE DECISION REQUIRED**
