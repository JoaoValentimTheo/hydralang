# HYDRA 0.2 — D003 Architectural Design Gate

**Status: DESIGN INVESTIGATION / PROPOSAL ONLY — HUMAN ACCEPTANCE REQUIRED.**

## Baseline and scope

- Audited starting branch: `main`, local HEAD = `origin/main` = remote `main` = `f91d9829c6964fed8e63b97b0a8a05f7a255e58c`; clean index/worktree at gate start, 2026-10-08. No preexisting changes were discarded.
- Hydra 0.1: frozen; D001 `break`/`continue`: accepted, implemented, independently audited and frozen (`764b901ec8a0f0febfb3624c392d27f14b631ff4`).
- D002 structural tuples: accepted, implemented, independently audited and frozen at `f91d9829c6964fed8e63b97b0a8a05f7a255e58c`. Historical D002 post-implementation run `37866987551` passed five exact-SHA jobs; this is NOT D003 CI evidence. Preserve the human-approved Option A depth invariant and its E1105/E3014/E9004 ownership.
- Hydra 0.2: IN PROGRESS; D003 normative acceptance: **NOT GRANTED**; implementation: **NOT STARTED**. No other collection, language family, tag, release or deployment is authorized.

Inputs inspected: `AGENT_STATE.md`, `docs/ROADMAP.md`, `docs/HYDRA_0_2_DECISION_QUEUE.md`, `docs/VERSIONING.md`, `docs/STATE_MACHINE.md`, `docs/ERROR_CODES.md`, `docs/ARCHITECTURE.md`, `docs/HIR_EXTENSIBILITY.md`, frozen D001/D002 documents and decisions, normative `spec/LEXICAL_GRAMMAR.md`, `spec/GRAMMAR.md`, `spec/TYPE_SYSTEM.md` and `spec/EXECUTION_MODEL.md`, and existing lexer/AST/parser/types/checker/runtime sources. Historical conditional language in the D002 audit describes its earlier publication checkpoint; final D002 freeze was established by the successful remote CI.

## Existing architecture and constraints

```text
source -> lexer -> parser/AST -> resolver -> checker/types
       -> typed HIR -> reference interpreter -> CLI
```

Observed: `crates/hydra-lexer/src/lib.rs` already tokenizes brackets, while the parser/AST has no List or indexing. Parentheses encode Unit/grouping/structural tuples, braces encode blocks, postfix dot encodes D002 tuple projection and numeric dots encode Floats. `crates/hydra-types/src/lib.rs` models primitive types and `Tuple(Rc<[Type]>)`; `crates/hydra-runtime/src/lib.rs` models immutable `Tuple(Rc<[Value]>)`. Resolution IDs are program-local. Typed HIR carries static types and stable builtin IDs. Parser owns 128 nested syntax and 256 expression depth guards; tuple arity and tuple type depth are 64. Runtime caps call depth at 128 and evaluation at 1,000,000 steps (E4006). D001 Return/Break/Continue outcomes are distinct and follow strict evaluation. The current tests exercise source-derived tuple DAGs and NaN; future collection types must extend those invariants deliberately.

Evidence is specific to the current code and contract. None of the proposed List syntax or semantics is already implemented.

## Candidate key

- **A** = standalone type-constructor/type-application foundation.
- **B** = immutable homogeneous List.
- **C** = Set.
- **D** = fixed Array.
- **E** = modules/imports.
- **F** = algebraic data types.
- **G** = generic functions.

### Equal-criteria qualitative decision matrix (1–8)

| Required criterion | A | B | C | D | E | F | G |
| --- | --- | --- | --- | --- | --- | --- | --- |
| **1. Immediate user value** | Type spelling with no values: low | Homogeneous grouped data and dynamic reads: high | Deduplicated membership useful, needs API choice | Fixed homogeneous data useful but Tuple overlaps | Organize multi-file code: high at scale | Domain variants/models: high | Reuse algorithms over types: moderate |
| **2. End-to-end demonstration** | Cannot execute a new program alone | Literal, pass, index, return, equality | Literal, dedup, contains required | Literal, typed length, index | Cross-file CLI build/import | Declare, construct and inspect variants | Bind type variable, infer/call |
| **3. New semantic decisions** | Constructor identity, namespace, grammar | Empty inference, index, ownership, bounds, equality | Equality, dedup, membership, ordering, syntax, limits | Length identity, literal count, ownership, indexing | Files, visibility, cycles, initialization, CLI | Declaration, nominal identity, fields, variants, access | Binders, substitution, inference, instantiation |
| **4. Hard dependencies** | Type-position grammar/lookup only | Localized builtin type spelling, literal, index and checked storage | Equality, duplicate policy and membership or equivalent useful observation | Compile-time length representation and indexing | Multi-file source map and stable cross-file IDs | Nominal IDs, constructor and consumption/access rule | Type-variable binding and substitution |
| **5. Premature abstraction risk** | High: machinery without consumer | Low if builtin-only and one variant per phase | Medium: generic equality/hash temptation | Medium: generic const system temptation | High: package/build framework temptation | Medium: shared nominal framework temptation | High: full polymorphism engine |
| **6. Lexer/parser impact** | Type argument delimiters versus comparisons | Brackets available; postfix, literals, builtin type args | Braces conflict with blocks; alternative sigil needed | Brackets clash with List; length syntax new | `import`, qualified paths, file units | Declarations, constructors, field/variant access | Type binders, instantiation versus comparison |
| **7. Resolver/checker impact** | Recognize type constructor/arity | Exact `List<T>`, contextual empty, homogeneous inference, index typing | Element identity, duplicate comparison and membership | Element type plus type-level constant length | Multi-unit symbol/visibility resolution | Named type/variant IDs and fields | Type variables, inference/substitution and signature checking |
| **8. Typed HIR impact** | New type syntax, no executable op | List construct and index nodes with spans/type | Set construction and membership nodes | Fixed-length value and index nodes | Cross-module symbol refs and initialization | Nominal constructor/access/variant HIR | Instantiated type arguments/monomorphization decisions |

### Equal-criteria qualitative decision matrix (9–16)

| Required criterion | A | B | C | D | E | F | G |
| --- | --- | --- | --- | --- | --- | --- | --- |
| **9. Storage/equality/ownership** | No new runtime data | Immutable `Rc` sequence; mixed List/Tuple DAG comparator | Dedup storage; set equality, deterministic search; hash optional | Fixed-size allocation and length identity | Module state/initialization lifetime | Tagged variant/record layouts and equality | Type erasure/specialization policy, runtime mostly unchanged |
| **10. State machine / flow** | Type parse/lookup only | Left-to-right elements, index base then key, propagate D001 | Ordered input evaluation then dedup; branch effects | Construct then index, D001 propagation | Load/resolve/init order, cycle recovery | Constructor args and variant access effects | Type binding/instantiation plus ordinary calls |
| **11. Diagnostic owner** | Type arity/name checker | Parser structure; checker homogeneity; runtime dynamic bounds | Parser/block ambiguity; checker equality/member types | Checker constant length and bounds; runtime index | Loader/import resolver, cycle/visibility | Resolver nominal names, checker field/variant | Generic inference/arity checker |
| **12. Compatibility risk** | `<` grammar ambiguity in types | Existing brackets lexed; postfix newline boundaries need tests | `{}` blocks collide | Bracket grammar conflicts with future List | Entry/CLI/namespace model changes | New declaration keywords and symbol scopes | Call/type grammar and name ambiguity |
| **13. Memory/stack/cost** | Type expansion/comparison risk | Finite literal/aggregate depth, fuel and DAG equality | Dedup quadratic or hashing attacks; strict bound needed | Large constant length and allocation | Cycles and whole-project memory | Recursive nominal values/variants; shared DAG | Instantiation explosion, recursive substitution |
| **14. Necessary regressions** | Type grammar/name/arity only; no executable oracle | Focused literal/type/index/D001/D002/NaN tests | Duplicate/membership/syntax/determinism/adversarial cases | Length identity/index/shape/overflow cases | Multi-file graph, conflicts, error spans, cycles | Constructor/variant/equality/malformed HIR | Inference/instantiation/soundness/recursion matrix |
| **15. Default CI duration** | Low now, deferred cost later | Low if finite deterministic tests + bounded fuzz | Medium: duplicate complexity probes | Low-medium for length/bounds edge cases | Medium: many filesystem/CLI fixtures | Medium: combinatorial variants | High risk from type-solver permutations |
| **16. Future family foundation** | Potential reuse, no proof yet | Useful concrete consumer; list-specific infra reused cautiously | Equality and deterministic-set insights | Compact static sequence if separately desired | Enables scalable multi-file library architecture | Useful type modeling and later patterns | Future polymorphism, only after concrete need |

These are qualitative engineering judgments, **not numerical scores**. Evidence for lower List syntax cost is the already tokenized brackets and existing tuple-style immutable storage/effect pipeline. List still needs type application and bounded runtime behavior; that cost is explicit. Modules and ADTs have high direct value but require several independent namespace/identity choices at once. Nothing here assumes a feature must follow Tuple merely by convention.

## Decision and alternatives

**Recommend exactly one D003 family: bounded immutable homogeneous List, proposal at `docs/decisions/003-list.md`.** It has executable source-visible value; its implementation could extend existing AST/type/HIR and shared immutable runtime conventions without new module identity, nominal declarations or a general generic type system. A readonly index makes variable-length sequences useful without mutable alias policy. This recommendation is conditional on human choice of the open semantic questions.

A offers a syntax foundation without a value producer; reject as standalone. C requires explicit duplicate/membership and source-brace disambiguation even if hashing is replaced with bounded linear search; defer. D adds type-level fixed length, overlapping Tuple for small fixed groups and List for homogeneous values; defer. E makes independent source mapping, cross-file symbol IDs, cycle policy and CLI decisions; viable later, too many cross-layer choices for this gate. F is valuable but needs named constructor identity, field/variant access and runtime tags; it need not drag in pattern matching/generics yet, but remains larger. G can exist without generic ADTs, but binds type parameters and needs a substitution/instantiation policy with limited present benefit.

## Dependency map: hard versus optional

```text
D003 List (one proposed family)
  HARD -> bracket literal parsing + postfix read-only index
       -> builtin-only List<T> type spelling and exact concrete type lookup
       -> element type inference + explicit empty-list typing policy
       -> AST -> resolver -> checker -> typed HIR support
       -> bounded shared immutable Value and invariant checks
       -> deterministic bounds/equality/NaN/fuel diagnostics
       -> compact source/corpus/property/fuzz verification
  OPTIONAL -> general type constructors, user generics, Set, Array
           -> mutation/append, iterators, comprehensions, slice views
           -> hash protocol, generic formatting, modules, packages
```

The standalone reusable type-constructor foundation is NOT a hard dependency: a builtin `List<T>` spelling with a single fixed-arity checker constructor suffices. A bounds-safe indexing operation is mandatory for the proposed useful subset; negative-index conveniences are not. Equality is part of proposed value usability, but hashing and total ordering are not. Empty-list semantics **must** be decided, not delegated to implementation.

## Cross-layer implementation impact (hypothetical; no work authorized)

| Phase | Minimum future change | Explicit invariant |
| --- | --- | --- |
| Spec and accepted decision | After human acceptance only, define one coherent grammar/type/execution/diagnostic contract | No contradictory local phase rules |
| Lexer | Reuse `[`/`]`; confirm numeric/projection and newline tokenization unaffected | No new separator or leaking lexical state |
| Parser/AST | List literal, postfix indexing, builtin type-arg grammar with exact span recovery | Syntax 128, expr 256, progress and depth restoration preserved |
| Resolver | Walk literal and index child expressions without new scopes; type-name check stays checker-owned | Per-function scope/ID reset unchanged |
| Types/checker | Structural invariant `List<T>`, empty expected context, normal type/effect tracking, type depth/size checks | D001 effects distinct from `Never`; D002 typed tuple limits remain authoritative |
| Typed HIR | Distinct list construct and index operations, explicit static types/spans | Malformed HIR rejected with internal E9004 |
| Runtime | Immutable shared element values, index safety, iterative mixed List/Tuple value/type comparisons, fuel accounting | No partial aggregate escape, clone explosion, false NaN equality or host panic |
| Diagnostics | Parser structure; checker source types; E4xxx dynamic index; E9004 internal; E4006 fuel | One owning phase and bounded UTF-8 primary span |
| Tests/corpus | Small independent positives/negatives, bounded properties and existing fuzz targets | Preserve 0.1/D001/D002; ordinary CI duration remains controlled |

## State machine impact / restoration obligations

Use the existing `docs/STATE_MACHINE.md` parser/checker/interpreter lifecycle, not a generic FSM. Parsing starts with `[` or a type-position `List<`, consumes a bounded sequence and either completes or reports a source error while making recovery progress, then restores recursive depth. Resolver/checker traverse children in evaluation order with local contexts, preserving D001 reachability (unreachable later expressions checked but their effects not treated as reachable). Construction starts with no published value, evaluates element by element, charges work, and publishes immutable shared storage only after every reachable element normally completes; Return/Break/Continue and errors abort immediately. Index evaluates base before index, then performs a bounds-checked read. Equality walks pair-identity DAGs while checking scalar leaves (including same-pointer NaN) under E4006 fuel. At every exit, function loop barriers, frame teardown, and call-depth restoration remain as before.

Potential pitfalls: type parse nested `>>` closers versus expression comparisons; `[x]` literal versus `x[i]` postfix; postfix after newline; braces remain blocks; parentheses remain tuples; `t.0[i]` and `xs[i].0` must preserve projection/float tokenization. Assignment to a list element stays invalid. Semicolons remain illegal as statement separators.

## Resource, compatibility and testing budget

Preserve existing 128 syntax / 256 expression / 64 tuple arity / 64 tuple depth / 128 call frames / 1,000,000 runtime steps. Propose, for human consideration, 256 maximum list-literal elements and 64 maximum combined List/Tuple structural type depth, without changing acceptance for any prior tuple-only program. An aggregate can share nested subgraphs, so logical expansion size is not an admissible work bound. Construction must charge per element before costly work/allocation; list+tuple equality and internal type/value validation must use iterative graph-aware traversal with fuel or a proven tight static bound. Do not use pointer identity as a scalar equality shortcut; IEEE NaN still differs from itself. No recursive formatting, deep-clone on normal aliasing, unchecked runtime indexes or indefinite fuzz/test loops. Dynamic out-of-range indexing must be a runtime source diagnostic, distinct from malformed HIR E9004. Exact sizes and rules remain open to human decision.

Compatibility classification: additive grammar for previously unused bracket literals/indexing and contextual type syntax, with possible ambiguity at expression/newline boundaries requiring regression proof. No changes to existing parenthesized tuple, block, Float, assignment, semicolon, effect, builtin, or parameter behavior are proposed. Previously valid 0.1/D001/D002 source must retain its results/diagnostics except deliberately newly recognized syntax which previously failed. Neither general user generics nor collection mutation is implicit in `List<T>`.

Future assurance would use fast focused deterministic regressions, a few classified CLI corpus programs, bounded properties and pinned four-target fuzz smoke; stress experiments opt-in only. Existing D002 counts (79 Rust, 34 positive/55 negative classified corpus, pinned 256/256/128/64 bounded target runs) are **historical evidence**, NOT new D003 tests or D003 performance measurements. Documentation-only design does not add fuzz seeds or change CI durations by introducing new test code.

## Reconciliation of stale current-state claims

- `docs/ROADMAP.md` incorrectly said, in current tense, “D002 implementation has not started” / “no tuple behavior ... in the executable compiler”; now records the frozen implementation, exact SHA and CI.
- `docs/HYDRA_0_2_DECISION_QUEUE.md` incorrectly said “D002 IMPLEMENTATION NOT STARTED”, and its top-level next action still required the implementation campaign; now records D002 freeze and D003 as a proposal only.
- `AGENT_STATE.md` had conditional pre-publication D002 freeze statements in its current status and limitations; now distinguishes fulfilled exact-SHA D002 freeze from historical audit conditions and records D003 design-only recommendation.
- Historical `HYDRA_0_2_D002_DESIGN_GATE.md`, `HYDRA_0_2_D002_ACCEPTANCE.md`, `HYDRA_0_2_D002_AUDIT_AND_FREEZE.md` and D001/D002 normative contracts remain unchanged. No version bump or `spec/` revision is part of this design exercise.

## Human decisions before any D003 normative acceptance

1. Approve/reject immutable homogeneous List as the one D003 family.
2. Decide empty-list typing context, including expected return/call/assignment positions and branch joins.
3. Approve builtin-only `List<T>` grammar/name/arity and nested closing delimiters without opening user generics.
4. Accept nonnegative zero-based index semantics and precise public runtime bounds diagnostic/code.
5. Accept list literal length, mixed List/Tuple depth, construction/fuel and allocation policies, including interaction with frozen D002 tuple bounds.
6. Confirm list equality/NaN and static `Never` effect rules exactly as proposed or request a separate semantic amendment before implementation.

## Exact authorized change list and gate validation

Only these five Markdown paths may be modified or added:

1. `HYDRA_0_2_D003_DESIGN_GATE.md` (this evidence record).
2. `docs/decisions/003-list.md` (unaccepted semantic proposal).
3. `docs/ROADMAP.md` (current-state reconciliation).
4. `docs/HYDRA_0_2_DECISION_QUEUE.md` (current decision queue).
5. `AGENT_STATE.md` (current checkpoint state).

Existing production code, tests, corpus, fuzz, Cargo, build scripts, normative `spec/`, diagnostic registry, CI workflows, frozen records and tags/releases remain unchanged.

### Prepublication local validation (2026-10-08; unchanged production sources)

| Gate | Observed result | Measured elapsed time |
| --- | --- | --- |
| `cargo +stable fmt --all -- --check` | PASS | 0.31 s |
| `cargo +stable clippy --workspace --all-targets -- -D warnings` | PASS | 0.46 s |
| `cargo +stable test --workspace --all-features` | PASS: 79 Rust test functions, 0 failures; documentation tests: 0 | 2.17 s |
| `cargo +1.85.0 check --workspace` | PASS, Rust 1.85.0 | 0.29 s |
| `./scripts/ci-macos.sh` | PASS on Darwin arm64, macOS 27.0.1; repeated 79 tests, fmt, Clippy, MSRV check and CLI `check`/`run examples/hello.hyd` (output `Hydra`) | 3.64 s |
| `git diff --check` | PASS | Not separately timed |

The classified corpus contains **34 positive** and **55 negative** `.hyd` programs; its checks passed in the six Rust corpus test functions above. Ten existing property-test functions and ten tuple-specific integration test functions passed; those are subsets of the 79 Rust tests, not additional totals. No D003 tests, fuzz seeds or compilation of hypothetical List examples were added. No new fuzz executions were required for the documentation-only gate. Existing CI fuzz-smoke budgets are unchanged. These wall-clock times are a local warm-cache snapshot, **not** a before/after CI comparison or assurance of equal times on other machines; the unchanged code and test inventory imply no newly introduced test workload.

Historical D002 CI: [run 37866987551](https://github.com/JoaoValentimTheo/hydralang/actions/runs/37866987551) at `f91d9829c6964fed8e63b97b0a8a05f7a255e58c` (5/5 success). **Final D003 commit exact-SHA GitHub Actions verification must be established after publication**; that result is recorded in the engineering response and CI itself because a commit cannot embed its own eventual SHA without changing that SHA.

**Design-only result:** Candidate B recommended; D003 selected for **proposal only**. Contract acceptance, implementation and D004 remain unauthorized.
