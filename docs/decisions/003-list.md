# D003 Decision — Immutable Homogeneous List

**Status: ACCEPTED — NORMATIVE CONTRACT LOCKED** (human authorization 2026-10-09).

**Implementation: NOT STARTED.** The authoritative accepted details and proof obligations are in `HYDRA_0_2_D003_ACCEPTANCE.md` and the D003 sections of `spec/`. Examples below describe future Hydra source and do not run on the current compiler. Frozen Hydra 0.1, D001 and D002 behavior is preserved.

**Historical proposal retained:** The original investigation, candidate scope, tentative recommendations, and open questions below document the **pre-acceptance design gate** at `5010e68d7924bf5f74c6350a41bc5140a48840c1`. They are historical evidence, not outstanding decisions or current normative rules. The 2026-10-09 accepted resolution appended at the end of this record and the acceptance ledger supersede any conflicting proposal wording.

## Problem and rationale

D002 tuples provide fixed-arity heterogeneous products. Hydra still lacks a homogeneous sequence of values of runtime-varying length. A bounded immutable List would allow passing and returning homogeneous sequences, choosing an item by a runtime-computed index, and comparing lists, without introducing mutation, iterators, general generic functions, modules or a collection framework. See `HYDRA_0_2_D003_DESIGN_GATE.md` for comparison and rejection evidence.

## Illustrative proposed source

```hydra
fn first(xs: List<Int>) -> Int {
    xs[0]
}

fn main() {
    let xs: List<Int> = [10, 20, 30]
    let ys = [1, 2, 3]
    let n = 1
    println(xs[n])
    println(first(ys))
    let empty: List<Int> = []
    let paired: (List<Int>, Bool) = (xs, true)
    println(paired.0[2])
}
```

All examples require D003 acceptance and implementation; the existing parser/checker rejects this syntax. No source-printing for whole lists is proposed.

## Scope proposed for acceptance

1. A *single* builtin parametric type spelling `List<T>`, where `T` is a valid existing concrete type (including a frozen D002 tuple type). Its constructor identity is a reserved builtin type namespace entry with arity one, resolved by the checker in a type position. `List` in an ordinary value position is not a callable constructor. No user-defined constructor, type parameter binding or generalized type application is enabled.
2. Homogeneous finite literal `[e0, e1, ...]` and `[]`; commas separate elements, with a permitted trailing comma on nonempty lists. `[x]` has one element, `[x, y]` has two. No semicolon delimiter. List literal element expressions evaluate strictly left to right, exactly once, and all are statically checked, including unreachable expressions.
3. The element type is inferred as one exact concrete type from normally completing elements; no numeric coercion or type promotion. Whole-expression `Never` and frozen D001 Return/Break/Continue effects are preserved. An unreachable element still receives static checking but contributes no reachable runtime effect. For no normal-completing elements, list construction itself has whole-expression `Never`; no fabricated `List<Never>` runtime value is produced.
4. An empty list has no standalone inferred element type. **Recommended, subject to human approval:** only an explicit expected `List<T>` type may type `[]`. This includes an annotated binding; whether return and call-argument contexts also count is an OPEN DECISION below. A bare `let xs = []` is statically ambiguous. An empty list is distinct from `Unit` and from an empty tuple.
5. `List<T>` has invariant structural identity: two list types agree only when their element types agree exactly. `List<Int>` differs from `List<Float>`, `List<(Int, Bool)>` from `List<(Bool, Int)>`. Whole-expression `Never` joins into a normal List type via existing bottom-type rules; incompatible normal list types do not join or coerce.
6. Postfix `base[index]` evaluates the base once, then the index once; only a normally valued `List<T>` base may be indexed. Index must have type `Int`; result type is `T`. It composes with calls, parentheses and D002 projection: `make_list()[0]`, `t.0[i]`, `xs[i].0`. Runtime bounds are checked for dynamic indices. **Recommended:** valid indices are 0 through length-1; negative indices are errors. No `xs[i] = v`, mutable field reference, slicing or optional index-return operation.
7. Storage is a shared, immutable, length-bounded aggregate analogous to D002 `Rc<[Value]>`. Constructing a list snapshots completed values; immutable list clones share storage, local rebinding does not alter a captured list, and indexing returns a value without producing an assignable location. Cycles are not expressible in the proposed source language. No mutation, append, concatenation, capacity API or implicit growth.
8. `==` and `!=` are allowed only on identical static List types and use ordered, length-sensitive structural equality. `!=` complements semantic `==`; recursive list/tuple comparisons must use bounded iterative pair memoization, including mixed shared-DAG aggregates. Pointer equality alone cannot imply value equality: shared Float NaN descendants remain unequal under IEEE 754. Other comparisons on lists are invalid. No hash, ordering, membership, formatting or builtin whole-list printing is introduced.
9. Parameter passing, return values and local assignment use existing statically declared/inferred `List<T>` types and preserve D001 function barriers. No new control effect or overload protocol exists.

## Lexical and grammar boundary

The lexer already recognizes `[` and `]` tokens, but the parser has no list expressions or indexing. The grammar proposal adds bracket literal as a primary expression, postfix brackets containing exactly one index expression, and builtin `List<T>` only in type positions. In these positions `<` and `>` must be parsed contextually; they remain comparisons in expressions. Nested `List<List<Int>>` must close deterministically without introducing a shift token. Type/name lookup must reject unknown constructors and wrong arity in one owning phase.

Parentheses retain Unit, grouping and tuple forms. Braces retain blocks. The dot remains D002 projection and decimal-dot float syntax remains untouched. Existing assignment-target grammar is unchanged except that indexing is explicitly **not** a place. Newlines remain statement boundaries; a newline before postfix `[` needs explicit precedence/recovery tests to avoid swallowing the next statement. Parser `current`, syntax depth and constructed-expression depth restore on all normal/error/recovery paths; missing `]` recovers with forward progress and accurate UTF-8 spans.

## Resource policy proposal — pending exact numeric acceptance

Keep the current syntax recursion guard (128), expression-depth guard (256), tuple arity (64), tuple structural depth (64), call-depth (128) and interpreter fuel (1,000,000). **Proposed, not accepted:** finite maximum list-literal element count, finite list-inclusive aggregate type nesting bound and bounded per-node list construction/allocation. Suggested review baselines are 256 literal elements and combined Tuple/List type depth 64; the exact values and whether that additional combined bound applies to mixed nesting require human acceptance. They must be checked before allocating unbounded host memory. These new bounds must not change the acceptance of any *previously valid* tuple-only program.

Construction charges the existing E4006 execution budget per evaluated element and before expensive aggregate work; equality charges per visited pair/field under its existing fuel. List indexing charges normal base/index expression execution and the access itself. Nested value/type validation must use iterative or provably bounded graph-aware traversals, with proportional fuel accounting where runtime. Length and depth checks precede allocation; do not recursively clone, hash or format shared aggregate DAGs. Source List construction permits sharing but not cycles. Runtime should not claim safety for arbitrarily large externally fabricated cyclic HIR/value graphs beyond the existing internal trust boundary, while all *checked source-derived* HIR must be safe under approved limits.

## Execution, effects, and state transitions

| Transition | Initial state → normal completion | Error / abnormal effect | Restoration invariant |
| --- | --- | --- | --- |
| Lex/parse List literal/type/index | Read `[` or builtin type argument → collect bounded children → completed AST with spans | Unexpected delimiter, EOF, arity/depth → parser diagnostic and bounded synchronization | `current` progresses; parse depth restored; no expression leaked as partially valid |
| Resolver | Walk child expressions, lexical lookup → existing symbol IDs | Undefined identifiers → existing resolver diagnostic | Function/scope stacks restored as in D001/D002 |
| Checker | Resolve exact `T` and context → homogeneous literal and typed index → typed HIR | Unknown/ambiguous element type, mismatch, non-List base, non-Int index → checker diagnostic | Per-function type/effect states reset; unreachable code still checked |
| Runtime construct | Evaluate e0..en sequentially → immutable shared List value after full success | `Flow::Return/Break/Continue`, divergence or diagnostic abort before later children; no partial value escapes | No field aliases; original effect forwarded; fuel charged |
| Runtime index | Evaluate base then index → checked element read → normal value | Propagate base/index effect first; invalid internal shape E9004; source-derived dynamic out of bounds runtime diagnostic | No mutation, pending operation abandoned, all call depth/frame transitions restored |
| Equality | Evaluate LHS then RHS → bounded ordered graph-aware comparison | Propagate evaluation effects; E4006 on fuel exhaustion; invalid HIR/value E9004 | No stale memo state after comparison; no host stack recursion |

D001 `Break` and `Continue` may only be consumed by the nearest eligible while *body*. List construction or indexing never consumes these effects; a call remains a function boundary. `Type::Never` never identifies a control effect. Runtime failures are diagnostics, not normal values.

## Diagnostic ownership — proposed, no allocated codes

- Lexer: unexpected characters via the current lexer-owned codes; list punctuation is already tokenized.
- Parser: missing/extra delimiters, malformed postfix, forbidden indexed assignment, nesting/literal count. Existing parser codes may be reused **only** if their existing meaning fits; otherwise propose a new code after human acceptance.
- Resolver: undefined value names via E2004; builtin type-constructor recognition belongs to the type-name/checker policy, not lexical value resolution.
- Checker: unknown type/constructor or wrong type arity, homogeneous mismatch, context-free empty literal, non-List index base and non-Int index. Existing E3001/E3002/E3003 may be applicable; any new code is **provisional**, subject to collision check against `docs/ERROR_CODES.md` (E3014 currently allocated).
- Runtime: source-valid dynamic out-of-range index needs an explicit **provisional E4xxx** decision, distinct from internal E9004. Fuel exhaustion stays E4006. A corrupt typed HIR, impossible type/value mismatch, malformed nested field, or invalid static HIR representation is internal E9004, never a substitute for a static source error.
- Preserve dot/index and UTF-8 primary-span ownership, one source error per owning phase; no redundant diagnostic cascades.

## Acceptance test obligations (future, not executed)

- Fast positive: typed and inferred literals, empty with approved context, calls/returns, tuple/List interleaving, zero/last index, chained projection, 1.25 float disambiguation, equality/inequality including shared NaN, return/break/continue short-circuit of construction and indexing, snapshot/rebinding.
- Fast negative: bare `[]`, heterogeneous elements, mismatched list types, wrong type arity, unknown constructor, invalid and negative index, out of range, indexed assignment, parser EOF/recovery, oversize/deep source, forbidden list comparison/whole-list builtin print, malformed typed HIR E9004.
- Classified positive/negative corpus: a small unique program per public grammar/typing/diagnostic boundary, plus unchanged 0.1, D001 and D002 corpus.
- Bounded deterministic properties: length/element roundtrip, equality symmetry where IEEE permits, `!=` complement, shared-DAG linear-node behavior with fuel, nested type identity, no effect leakage and fuel exhaustion. Reuse the four existing pinned fuzz targets with a few distinct seeds, fixed CI budgets; optional heavy stress experiments stay opt-in.

## Questions requiring explicit human acceptance

1. Does the language accept empty `[]` only at annotated local declarations, or also when an enclosing function argument, return or assignment supplies an exact `List<T>` expected type? Do not silently introduce general bidirectional inference.
2. Confirm builtin-only spelling `List<T>` versus an alternative non-generic type syntax; define parser handling of nested closing `>` and reserved type-name collision.
3. Confirm nonnegative zero-based dynamic indexing, runtime out-of-bounds policy and public diagnostic ownership/code, including indexing an empty list.
4. Approve numerical list-length, mixed aggregate depth and allocation/fuel charging bounds; prove no regressions for previously valid tuple-only programs.
5. Confirm homogeneous inference for elements with `Never` outcomes and contextual treatment of fully non-normal list expressions (especially branch joins).

Proposed rejection of the alternatives: type constructors alone have no executable benefit; Set creates duplicate/membership and block-syntax decisions; fixed Array requires length-bearing type identity and overlaps existing tuple/List use; modules need cross-file namespace/source identity work; ADTs need nominal identity/constructor rules; user generic functions need binding/substitution/instantiation infrastructure without an accepted consumer. These remain independent families. **Historical gate condition (now fulfilled for specification only):** a human must accept a finalized decision before normative specification may proceed.

## Human acceptance and binding resolution — 2026-10-09

The owner accepts **exactly one** D003 family: **immutable homogeneous List**. This acceptance authorizes its normative specification only; it does **not** authorize Rust implementation, tests, fuzz seeds, releases or tags. The historical design analysis above remains a record of alternatives, not implementation evidence.

- **Type and syntax:** builtin, type-position-only `List<T>` with **exactly one** argument, invariant structural identity and no general type application, user generics, value-level List constructor or globally reserved `List` keyword. `[]`, `[x]`, `[x,y,]`, and readonly postfix `base[index]` are accepted future syntax. Nested closing `>>` is two type-position delimiters; relational operators in expressions remain unchanged.
- **Empty literal:** bare `[]` is E3015. An exact expected `List<T>` is admitted **only** from an annotated declaration, statically typed argument position, declared function return (tail expression or explicit `return`), assignment to an existing typed mutable local, or immediate structural List/Tuple literal children when the parent has a precise expected type. Context is lexical and scoped to that checked expression; no generic bidirectional inference, cross-branch inference or spillover to independent expressions.
- **Types and effects:** list elements have exactly one compatible static type on normal completion; no coercion. `Type::Never` denotes absence of normal completion, **not** a stored runtime element or an identification of D001 Return/Break/Continue. An explicitly annotated empty `List<Never>` is allowed when `Never` is a valid annotation. Construction proceeds in source order and propagates the first non-normal outcome, statically checking subsequent unreachable elements without treating their effects as reachable.
- **Indexing:** evaluate base once, then index once; `Int` only; checked zero-based `0 <= index < length`; negative and out-of-bounds indices report runtime E4007; no indexed assignment (E1104), slicing, mutations, append or optional results. Composition with calls and D002 projection uses the same postfix precedence and cannot cross a statement-leading newline.
- **Structural guards:** List literal element maximum **256** (257: parser E1101); combined List/Tuple type depth **64** with primitive depth 0 and one plus maximum child depth for each aggregate. Written depth 65: parser E1105; inferred depth 65 containing List: checker E3016; frozen tuple-only inferred depth 65: E3014. Tuple-only programs valid before D003 stay valid; tuple arity 64, syntax nesting 128 and expression-tree depth 256 are unchanged.
- **Runtime:** immutable shared aggregate storage and snapshot value semantics; allocate only after bounded elements are evaluated, with existing execution fuel charged for construction, indexing, type/value validation and mixed List/Tuple equality. Existing 1,000,000-step E4006 and 128-frame call-depth limits remain. Equality uses ordered, length-sensitive, graph-aware work and preserves NaN inequality even for physically shared aggregates; `!=` complements `==`. E9004 remains internal-only for malformed typed HIR. Fuel is not promised as a universal memory cap.
- **Diagnostics:** after checking `docs/ERROR_CODES.md` at the acceptance baseline, the unallocated codes **E3015–E3019** (checker) and **E4007** (runtime) are reserved for D003 with precisely the meanings in the acceptance ledger. Existing E1101/E1104/E1105/E1106/E3001/E3002/E3003/E3004/E3007/E3014/E4006/E9004 keep their established owners.
- **Compatibility:** additive syntax only; no intentional change to accepted 0.1/D001/D002 source semantics. Type/value namespaces remain distinct. No general List printing, serialization, ordering, hash, membership, iterators, Sets, Arrays, modules or generic functions.

The accepted detailed contract, unresolved implementation proof obligations and future testing matrix are maintained in `HYDRA_0_2_D003_ACCEPTANCE.md` and the normative D003 sections of the language specifications. **D003 is accepted and locked, but remains unimplemented.**
