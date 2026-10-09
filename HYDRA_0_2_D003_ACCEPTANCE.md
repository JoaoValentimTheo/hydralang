# HYDRA 0.2 D003 — Normative Acceptance and Implementation Proof Obligations

**Status: ACCEPTED — NORMATIVE CONTRACT LOCKED; IMPLEMENTATION NOT STARTED.**

- Human authorization: 2026-10-09; specification and documentation only.
- Starting baseline: `5010e68d7924bf5f74c6350a41bc5140a48840c1` (`main`, synchronized with `origin/main`). The staging index was clean, but the resumed worktree already had uncommitted **D003 documentation in progress**: `docs/decisions/003-list.md`, `spec/GRAMMAR.md` and `spec/LEXICAL_GRAMMAR.md` modified, plus this acceptance document untracked. These changes were inspected, retained and reconciled; there was no reset or clean.
- Prior design proposal: `HYDRA_0_2_D003_DESIGN_GATE.md`, `docs/decisions/003-list.md`; design CI: run `37870992031` (5/5 SUCCESS at the design SHA, prior checkpoint).
- Frozen compatibility checkpoints: Hydra 0.1; D001 loop control; D002 tuples at `f91d9829c6964fed8e63b97b0a8a05f7a255e58c` (CI `37866987551`, 5/5).
- This contract governs **future behavior**. Existing 79 Rust tests and the 34-positive/55-negative classified corpus cover the **unchanged current compiler**, never the hypothetical List examples.

## Accepted family and exclusions

One family: **immutable homogeneous List**, usable as `[1, 2]`, `[x,]`, `[]`, `List<Int>`, `List<(Int, Bool)>`, `List<List<Int>>` and through readonly postfix `xs[i]`. A List's element type has exact structural identity; no implicit numeric conversion, covariance, subtype coercion or element conversion. `List` denotes a builtin **type-only** constructor with exactly one argument, recognized in type positions without a globally reserved keyword or changes to value identifiers. `List` is not a callable value constructor. Unknown types remain E3001, and syntactically well-formed wrong `List` type arity is E3017.

Explicitly excluded: user or function generics, generic constructors, general type application, traits, type classes, variance, mutable elements, mutating indexing, slicing, append/insert/remove/pop, concatenation, iteration, comprehensions, Sets, fixed Arrays, modules, patterns, hashes, membership, ordering, List serialization/printing and List-specific formatting. The prior standalone-constructor, Set, Array, module, ADT and generic-function candidates remain independently deferred; no D004 decision is implied.

## Empty literals: narrowly contextual expected type

An empty literal has no intrinsic element type (`let xs = []` -> checker **E3015**). It can construct a `List<T>` only when an *exact* expected List type is available in one of these positions:

1. Direct initializer of an explicitly annotated local: `let xs: List<Int> = []`.
2. Direct actual argument of a statically resolved function parameter declared `List<T>`: `consume([])`.
3. Tail result or explicit `return []` in a function **explicitly declaring** `-> List<T>`.
4. Assignment RHS to an existing mutable local whose static type is `List<T>`: `xs = []` (unchanged mutability and assignment rules).
5. An immediate structural element of a directly constructed List or Tuple literal with known expected aggregate type: `let xs: List<List<Int>> = [[], []]`; `let t: (List<Int>, Bool) = ([], true)`. This rule applies transitively **only while following directly constructed aggregate literal children**, each receiving its exact expected member type.

Expected types are explicitly scoped to the appropriate expression/checker invocation; they cannot leak to a later statement, unrelated call, function, error path, or other initializer. No inference from sibling expressions, a later assignment, arbitrary operator operands, unconstrained type variables or another `if` branch. An untyped `[]` directly inside a branch is not rescued by a typed list in a different branch or by an enclosing contextual annotation threaded through an `if`; branch joins of **already independently typed** identical List types are allowed. A preexisting normal `Never` join remains at the **whole-expression** boundary.

## `Never`, effects and normal value construction

Static `Never` is the bottom type for an expression without normal completion, never a runtime value. The checker must preserve the frozen D001 path distinction among normal fallthrough, `Return`, `Break`, `Continue` and possible divergence; runtime failure remains a separate diagnostic. List elements are evaluated once left-to-right, and the first non-normal effect/diagnostic prevents evaluation of all later elements and publication of any partial List. The checker still checks later unreachable source for type and placement errors without promoting unreachable effects into reachable paths.

Every normally completed nonempty List has compatible elements of one exact static `T`. A normal `Never` **value** is impossible and cannot become a stored element or justify implicit element conversion. A wholly non-normal construction has whole-expression `Never`, not a fabricated `List<Never>` value. An explicitly annotated *empty* `List<Never>` is legal where `Never` itself is already a legal type annotation and has zero elements. D001 function and while-body effect barriers remain unchanged.

## Grammar, indexing and allocation

`[]`, `[x]`, `[x,y]`, `[x,y,]` and nested literals are bracket primaries. A postfix `base[index]` contains exactly one expression, associates left with calls and D002 `.DIGITS` projections, and cannot attach across a statement-ending newline. `List<List<Int>>` closes through two contextual `>` delimiters in *type* parsing, without shift operators or interference with expression `<`/`>`. Preserve `()` Unit, grouping, D002 tuples, float `1.25`, no semicolon separators, and UTF-8 spans/recovery progress.

Indexing executes base exactly once and then index exactly once, forwarding each non-normal effect before proceeding. The normally valued base must have `List<T>` (checker E3018) and index must have `Int` (checker E3019); result has type `T`. Valid runtime indices satisfy **0 <= index < len**. Negative, `len`, beyond-len and indexing an empty List with normally evaluated operands report **E4007** with the originating index operation's span. Indexing is readonly and produces no assignable location; `xs[0] = 10` reports parser E1104.

A literal contains **at most 256 elements**, counted as expressions, not source bytes/tokens; 257 yields parser E1101. Existing parser nesting 128 (E1105), expression-tree depth 256 (E1106), tuple arity 64 (E1101), call depth 128 (E4003), and execution fuel **1,000,000 steps** (E4006) remain exactly as before.

**Combined aggregate type depth** on each type path is:

```text
depth(Int|Float|Bool|String|Unit|Never) = 0
depth(List<T>) = 1 + depth(T)
depth((T1, ..., Tn)) = 1 + max(depth(T1), ..., depth(Tn))
depth(grouped T) = depth(T)
```

The empty type `()` is `Unit` and therefore depth zero. The maximum depth is **64**, including nested List/Tuple mixes. Exactly 64 succeeds; 65 in a written type is parser **E1105**; 65 in an inferred type **containing List** is checker **E3016**; an inferred **tuple-only** depth 65 remains **E3014** under frozen D002. Validation must cover locals, annotations, parameters, results, calls, assignments, joins, tuple projections, list indexes and aggregate fields. No successfully checked source-derived executable typed HIR may violate this bound. The combined rule must not reject any previously accepted tuple-only program.

List storage is immutable, reference-shared, snapshots its fully constructed elements, and avoids recursive deep cloning. A future runtime may use `Rc<[Value]>` or an equivalent proven bounded representation. Source List construction introduces no cycles. Work must be charged from the **existing fuel** proportional to construction elements, indexing, nested validations and equality comparisons before expensive growth. This bounds **individual literals** and charged interpreter operations; it does not guarantee a global hard memory cap or protection against arbitrarily forged internal HIR. An internal malformed type/value, corrupt nested field, invalid HIR index or invalid function argument/return must terminate with source-spanned **E9004**, not panic or silently succeed; valid-source out-of-range index remains ordinary **E4007**.

Equal static List types support only `==` and `!=`: ordered, length-sensitive, structural, elementwise equality; mixed List/Tuple shared DAGs are compared iteratively with pair identity memoization and fuel charging proportional to traversed nodes/edges. First-visit children must be checked even for identical pointers; shared Float NaNs retain `NaN != NaN`. No false memoization may mask a different scalar descendant. `!=` complements the actual semantic `==`. No recursive source/host formatting or implicit large graph cloning is allowed.

## Diagnostic allocation and phase boundaries

The `docs/ERROR_CODES.md` registry was checked at baseline: the following codes had no existing meanings; D003 reserves them **normatively**, but no diagnostic emitter has been implemented.

| Code | Owner | Accepted source condition |
| --- | --- | --- |
| E3015 | Checker | Untyped empty List literal with no permitted exact expected `List<T>` |
| E3016 | Checker | Inferred aggregate type **containing List** exceeds depth 64 |
| E3017 | Checker | Syntactically well-formed builtin `List` type with wrong type-argument arity |
| E3018 | Checker | Normally valued non-List base of an index expression |
| E3019 | Checker | Index expression has a normal type other than `Int` |
| E4007 | Runtime | Normally evaluated List index negative or outside `0..len` |

Preserved owners: E1101 malformed List syntax/257 elements; E1104 invalid indexed assignment; E1105 written depth/nesting; E1106 expression depth; E3001 unknown type; E3002 element/type mismatch; E3003 unsupported List operators; E3004 incompatible branches; E3007 primitive-only builtin printing; E3014 tuple-only inferred depth; E4006 fuel; E9004 malformed internal HIR. E3009 continues to mean **value-call** arity mismatches, never `List` type arguments. Recovery and all public primary spans must remain bounded and valid on UTF-8 character boundaries; no redundant cross-phase diagnostics.

## Future parser/checker/runtime state obligations

The **accepted, unimplemented** D003 additions to `docs/STATE_MACHINE.md` are authoritative alongside this ledger. Parser bracket/type-argument parsing must restore syntax/expression-depth context and guarantee progress/EOF safety after malformed delimiters. Resolver traverses element/base/index expressions under existing lexical scopes and adds no names or writable fields. Checker pushes and removes exact expected List types only for the five allowed contexts, resets per function and error path, and checks all elements while preserving D001 reachability. Typed HIR must carry static List/index types and source spans. Runtime constructs completed shared values, checks index and mixed structural equality, spends existing fuel and restores call/frame depth across success/effects/errors; it defensively rejects invalid typed HIR with E9004.

## Required future implementation verification (not executed by this campaign)

| Class | Distinct executable proof obligations |
| --- | --- |
| Positive source/corpus | Typed/inferred singleton, multiple and 256-element literals; all five contextual empty cases; nested List/tuple in both directions; functions/returns; first/last index; postfix call/projection/index; immutable snapshot/rebinding; equality/inequality; shared NaN; D001 Return/Break/Continue short-circuit and branch-conditional effects |
| Negative source/corpus | Bare `[]`, heterogeneity, wrong List arity, unknown type, non-List/non-Int index, negative/out-of-bounds/empty index, indexed assignment, 257 literal elements, written and inferred depth 65, illicit `if` branch inference, bad bracket/recovery, List printing/ordering, malformed typed HIR |
| Boundaries and properties | 256/257 element boundary; tuple arity 64/65; mixed type depth 64/65 in types/returns/calls/projections; identity and round trips; symmetry where scalar rules apply, `!=` complement; NaN physical sharing; linear-node mixed DAG comparison; fuel E4006; source-spanned E9004; diagnostic determinism/UTF-8; parser recovery/state restoration |
| Regression/platform | Frozen Hydra 0.1/D001/D002 corpus/behavior; `1.25` vs `t.0.1`; independent newlines; fmt, clippy, workspace/all-features Rust tests, Rust 1.85 check, native macOS CLI, Linux/Windows CI; pinned four-target bounded fuzz smoke |

Keep a small set of independent deterministic regressions and classified corpus examples. Ordinary CI must stay bounded; large stress experiments require explicit opt-in. Do not treat test counts as correctness proofs or conflate Rust functions, corpus files and fuzz executions.

## Compatibility, governance and evidence

Classification: **additive language extension** over frozen Hydra 0.1/D001/D002. Accepting formerly invalid List syntax is intentional; existing valid programs, float lexing, projection semantics, statement boundaries, type identity, scope/mutability, builtins, and D001 effects must not change. The implementation campaign must explicitly test possible bracket/newline, angle-bracket/comparison, postfix/float and typed-context collisions.

Only documentation changes are authorized for acceptance. No `.rs` files, executable tests, corpus, fuzz data, Cargo files, workflows, build scripts or version identifiers may change. D003 implementation requires separate human authorization; D004, other families, tags, releases and Hydra 0.2 publication remain unauthorized.

**Verification/publication record:** Initial branch and local/remote SHA matched; the index was clean but worktree contained the preserved in-progress D003 documentation described above. The acceptance commit SHA, corresponding exact-SHA five-job workflow results and final local/remote synchronization are reported in the completion report after publication, because a commit cannot truthfully contain its own SHA and future workflow conclusion without changing its identity. Required local gates are `cargo +stable fmt --all -- --check`, `cargo +stable clippy --workspace --all-targets -- -D warnings`, `cargo +stable test --workspace --all-features`, `cargo +1.85.0 check --workspace`, `./scripts/ci-macos.sh` and `git diff --check`; actual results are recorded after execution in the completion report. Existing 79 Rust tests, classified corpus and prior CI must not be presented as tests of unimplemented D003 behavior.
