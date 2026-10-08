# HYDRA 0.2 D002 — normative acceptance ledger

- Human decision: **APPROVED on 2026-10-08**, following independent review of the lexer, parser, AST, type syntax, resolver, checker, frozen D001 effect model, typed HIR, interpreter, equality, limits and versioning.
- Acceptance starting `main` / `origin/main`: `7cb446edc52de315d8a99a7fd4c15917539ad56d`.
- Baseline remote CI: GitHub Actions run `37829274647`, **5/5 successful**, for the pre-acceptance SHA.
- D001: **ACCEPTED / IMPLEMENTED / AUDITED / FROZEN** at `764b901ec8a0f0febfb3624c392d27f14b631ff4`. No changes to the frozen D001 implementation or contract.
- D002: **ACCEPTED / NORMATIVE CONTRACT LOCKED / NOT IMPLEMENTED**.
- Hydra 0.2: **IN PROGRESS**, not released; D003 not selected; non-Tuple H15 collections deferred.

## Accepted contract

Only ordered, immutable, fixed-length, heterogeneous **structural tuples**, structural tuple type annotations, read-only zero-based constant positional projection, and structural equality/inequality are accepted. `()` is Unit, `(x)` grouping, `(x,)` a singleton tuple, `(x, y)` a two-element tuple, and nonempty tuples may have trailing commas. Equivalent distinctions apply to `(Int)`, `(Int,)`, `(Int, String)` and nested tuple types. Parenthesized semicolon separators remain invalid. No new keywords or reserved identifiers are introduced.

Projection `t.0`, `t.1` and `t.0.1` has the same postfix precedence as calls and associates left to right. Decimal indices are unsigned constants, with no dynamic indexing or mutation. Float tokenization such as `1.25` is unchanged; after a projection dot, a digit run lexes as an integer without merging the following `.digit`, so `t.0.1` is `Identifier Dot Int Dot Int`. Existing 0.1/D001 statement boundaries and grouping semantics remain unchanged.

Tuple type identity is ordered and arity-sensitive, with independently inferred element types, no coercion or width subtyping. Source-order evaluation is strict; if an element returns, breaks, continues, diverges or faults, no later element runs and no partial aggregate escapes. The frozen D001 flow/effect analysis remains authoritative. Whole-expression `Never` is different from a well-formed uninhabited tuple type containing `Never`. Projection from whole-expression `Never` propagates the effect without a new checker error.

Tuple storage is immutable, snapshotting element values and sharing aggregate storage across cloning, reads, calls, returns and projection. Equality is fieldwise in order, short-circuiting on the first inequality, and `!=` is its negation; static tuple types must match. Floating-point IEEE NaN semantics are preserved even for physically shared nodes: pointer equality cannot shortcut first-visit descendant comparisons. An iterative/otherwise bounded tuple-pair worklist with identity memoization must prevent exponential recomputation on shared DAGs. Tuple-pair and field-comparison work consumes the existing **1,000,000-step execution budget**, raising E4006 upon exhaustion. No independent comparison budget is accepted.

Tuple arity is at most **64** (E1101 for 65); tuple type nesting is at most **64 layers along a path** (E1105). Existing parser nesting **128** (E1105) and expression-tree depth **256** (E1106) stay unchanged. A too-large projection integer reports E1102, malformed syntax E1101, and assigning a projection E1104. The checker owns **E3012** (normally valued non-tuple projection) and **E3013** (index outside static tuple arity), each with dot-through-index primary span; existing E3002/E3003/E3004 cover type/operator mismatches. Whole-tuple arguments to `print`/`println` remain unsupported (E3007), with no source tuple printing. Malformed checker/HIR invariants use E9003 and runtime typed-HIR/value invariant failures use source-spanned E9004 without panics or unchecked indexing.

## Scope, compatibility and implementation boundary

Compatibility: **COMPATIBLE EXTENSION** relative to frozen D001. No previously valid float, literal, grouping, function call, assignment, keyword, statement boundary or normal program behavior may change. No destructuring/patterns, tuple-field mutation, named fields, ordering, hashing, iteration, slicing, generic indexing, type constructors, List/Set/Array, generics, modules or D003 are accepted.

The normative records changed under this acceptance are `docs/decisions/002-tuples.md`, `spec/GRAMMAR.md`, `spec/TYPE_SYSTEM.md`, `spec/EXECUTION_MODEL.md`, `spec/LEXICAL_GRAMMAR.md`, `docs/ERROR_CODES.md` and `docs/STATE_MACHINE.md`; status is reconciled in `AGENT_STATE.md`, `docs/HYDRA_0_2_DECISION_QUEUE.md` and `docs/ROADMAP.md`. The historical selection gate `HYDRA_0_2_D002_DESIGN_GATE.md` remains a historical record of the earlier proposal.

**Implementation has not started.** This acceptance authorizes documentation only. No Rust implementation, test, corpus, fuzz, Cargo, workflow, release, tag or publication changes are included. A separate explicit human instruction is required for D002 production work, with tests for IEEE NaN/shared DAG equality, D001 effect propagation, numeric lexing, guard boundaries, malformed HIR and cross-platform conformance.
