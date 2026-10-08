# D002: Structural tuples with constant positional projection

- Status: **Accepted** — normative contract locked; implementation not started
- Date: 2026-10-08
- Compatibility: **compatible extension** relative to the frozen D001 baseline
- Supersedes: none
- Selected family: **Tuple**, and no other Hydra 0.2 family
- Human approval: **GRANTED on 2026-10-08 after independent architectural review** of the existing lexer, parser, AST, type syntax, resolver/checker, D001 effect model, HIR, interpreter, equality, resource guards and versioning policy. This acceptance authorizes specification changes only; Rust implementation requires a separate human decision.

## Problem

Hydra can return and pass only primitive values. Programs cannot construct a fixed heterogeneous product such as `(42, "ready")`, annotate its type `(Int, String)`, or recover a constituent. The current `TypeExpr` is a single name, semantic `Type` has six primitive variants, and typed HIR/runtime contain no aggregate. A useful product type can be defined independently of parameterized type constructors, lists, patterns, nominal declarations, modules, or generics.

## Alternatives

1. **Selected:** structural tuples with a fixed compile-time arity, static numeric projection, and structural equality.
2. Type-constructor foundation first: enables future `List<T>` but has no accepted executable parameterized consumer today, so would establish a speculative type application grammar and identity scheme.
3. Tuple literals without projection: constructible but inconvenient to consume without accepting pattern matching or new destructuring semantics.
4. Bracket indexing for tuples: workable, but would commit `[]` to indexing semantics ahead of a list/array decision; use a tuple-specific constant field projection instead.
5. Nominal products or labeled records: require declaration, field-name, and nominal-identity contracts not necessary for positional products.

## Chosen semantics

### Surface syntax and grammar

The following grammar is **accepted and normative for Hydra 0.2 D002**. It is not yet implemented by the current parser:

```text
tuple-expression := "(" expression "," (expression ("," expression)* ","?)? ")"
tuple-type       := "(" type "," (type ("," type)* ","?)? ")"
projection       := postfix-expression "." DECIMAL_INDEX

"()"             = Unit literal, never a zero-element Tuple
"(x)"            = grouping, not a Tuple
"(x,)"           = one-element Tuple
"(x, y)"         = two-element Tuple
"(x, y,)"        = same two-element Tuple; trailing comma allowed
"(Int)"          = grouped type Int
"(Int,)"         = one-element structural Tuple type
"(Int, String)"   = two-element structural Tuple type
"()" in type position = Unit, equivalent to spelling "Unit"
```

More precisely: parenthesized comma-separated expressions/types become tuples **iff** at least one comma occurs; otherwise parentheses group one expression/type, and empty parentheses denote Unit. A nonempty tuple must have at least one element. A one-element tuple requires the trailing comma. Two or more elements permit an optional trailing comma. Commas are separators only inside parentheses; they do not separate statements. Parenthesized line breaks follow the existing grouping/call layout policy. Semicolons remain illegal.

`DECIMAL_INDEX` is a non-negative ASCII decimal integer token; `t.0`, `(make_pair()).1`, `t.0.1`, and `t . 0` are projections with highest postfix precedence, at the same level as calls and left-associated. Both the dot and its integer must stay on the same logical source line as the base expression; no implicit cross-statement newline binding. Spaces on that line are allowed. Leading zeroes denote the same index (e.g. `.00` means `.0`). Negative, fractional, identifier and dynamic indices (`t.-1`, `t.name`, `t.(i)`) are not supported. The lexeme `t.1.5` is **two** integer projections, parsed `(t.1).5`, and is valid only if the intermediate value is a tuple with field 5; it is never a fractional field index.

**Lexer requirement:** current number scanning consumes a decimal fraction when a digit run is followed by `.` and another digit. After a `Dot` projection token, scan the following digit run as `Int` without absorbing a subsequent `.digit`, so `t.0.1` lexes `Identifier Dot Int Dot Int`. Existing float literals such as `1.25` must retain their current tokenization and values. No new reserved keywords or source-level name namespaces.

Postfix projections are read-only. `t.0 = 4` is an invalid assignment target (existing E1104). No tuple destructuring, pattern matching, tuple element updates, named fields, computed/dynamic indices, slicing, iteration, or general collection indexing is admitted. A projected value may itself be a tuple and accept another projection.

### Structural type identity, inference, and compatibility

Introduce a semantic `Tuple([T0, ..., Tn-1])` type with **ordered** element types and arity in its identity. `(Int, String)` differs from `(String, Int)`, `(Int,)`, and `Int`. `Tuple` is structural, has no declaration/constructor identity, and does not use generic `Name<...>` syntax. Types are constructed recursively from the existing primitive names and tuple type expressions; nested tuples are allowed within limits below. Type names remain recognized by the existing checker; `Type::from_name` remains primitive-only.

`let t = (1, "x")` infers `(Int, String)`. Function arguments/returns and local annotations may spell tuple types, e.g. `fn pair(x: Int) -> (Int, Bool) { (x, true) }`. Literal elements are type-checked and evaluated **left to right**. A tuple has a normal value only if all elements complete normally; an element that has only `Never` normal type makes the aggregate expression's normal type `Never`, while preserving the exact D001 return/break/continue/divergence effects. Tuple types containing `Never` are well-formed but uninhabited in that position, not magically coerced to other tuple types. No elementwise numeric coercion, covariance, subtyping, or tuple-width conversion. `Never` at the *whole-expression* level retains existing bottom-type compatibility.

`Type::join` continues to join two **identical** normal tuple types or `Never` with a tuple type, and rejects two different normally returning tuple shapes; D001 path outcomes must not be inferred from `Never` alone. Function signature, assignment, `if` branches, and return checking use the same structural identity. A tuple projection on a normal tuple type at a valid constant index has precisely that element's static type; a projection whose base has no normal value also has normal type `Never` and propagates its effects. No projection of a normal non-tuple type is valid. `()`, `Unit`, and `(Unit)` describe the same existing Unit type; `(Unit,)` is a different tuple type.

### Execution, ownership, equality, and output

`Tuple` is an **immutable, fixed-length value**. Each element is evaluated once from left to right; on a non-normal D001 effect, subsequent elements are skipped and no partial tuple escapes. Tuple construction snapshots the values produced at that time. Reassigning a mutable local after construction does not mutate a captured field; projected fields cannot be assigned. Values follow Hydra's current value-copy behavior, but implementations must preserve sharing of immutable aggregate storage on clones (e.g. `Rc<[Value]>` or an equivalent persistent representation). Never deep-copy an arbitrarily expanded tuple graph merely to read a local or pass an argument. Cyclic tuple values cannot be created by valid source.

`==` and `!=` operate only on operands with matching static types, just as in the existing checker. Tuple equality is **recursive, position-by-position equality**, comparing corresponding fields in ascending index order and stopping at the first unequal field; `!=` is its logical negation. Scalar equality stays unchanged, including IEEE `Float` behavior (`NaN != NaN`). In particular, **pointer identity alone cannot imply tuple equality** because a tuple may contain `NaN`. No hashing, ordering (`<`, `>` etc.), sorting or total-equality contract is introduced. `Never` whole-expression semantics remain as in D001.

`print` / `println` retain their existing primitive-only argument contract: a tuple as a whole is rejected with E3007; users can print a projected primitive field, e.g. `println(t.0)`. This consciously avoids specifying an aggregate text/serialization protocol or an output expansion policy as a hidden second family. Host-side `Value::Display` for tuples, if needed by Rust consumers, must be **bounded and nonrecursive**, using an opaque marker such as `<tuple>`; it is not a source-level tuple formatting facility. Source-visible display for primitive values is unchanged.

### Diagnostics and invalid forms

- Existing **E1101** for malformed tuple syntax, missing element/comma/closing `)`, invalid dot target syntax, or a non-integer projection suffix (`t.name`, `t.-1`, `t.(0)`). Standard parser synchronization and progress guarantees apply.
- Existing **E1102** for a dot integer literal that does not fit the supported unsigned projection-index representation; reject it instead of truncating or panicking.
- Existing **E1104** for assigning to a projection rather than a mutable name.
- Existing **E1105/E1106** for parser structural/expression nesting constraints; the tuple AST/type parser and expression-depth walker must explicitly participate.
- Accepted **E3012**, checker: positional projection on a *normally valued* non-tuple type. Primary span: dot through index.
- Accepted **E3013**, checker: statically out-of-range positional projection. Primary span: dot through index. Include tuple arity and requested index.
- Existing **E3002/E3003/E3004** for structural type mismatch, equality operand mismatch/non-equality operator, and incompatible branch types, respectively. Existing **E3007** for passing an entire tuple to `print`/`println`; no new builtin overload.
- Existing **E9003/E9004** for impossible checked-HIR lowering and malformed runtime tuple/projection invariants, with the source span. Do not permit malformed HIR to cause panics or unchecked indexing.

### Resource and safety boundaries

Source tuple arity is at most **64 fields**, and tuple-type nesting is at most **64 tuple layers** along any path. Violations are source-level parser **E1101** (arity) or **E1105** (type nesting), with valid spans and recovery. The existing 128 syntax recursion-depth and 256 expression-tree-depth guards still apply, counting **every tuple child and projection**. The implementation must make type tree traversal and runtime equality resistant to deep recursion even for directly constructed malformed internal values: use bounded/iterative walkers, checked allocations, and E9004 for invalid typed HIR.

Sharing makes logical expansion potentially exponential (`t=(t,t)` built repeatedly), despite linear physical tuple-node growth. Equality must therefore use an **iterative worklist and a visited pair-of-tuple-node identity set**, without assuming equal pointer identity implies semantic equality. A repeated pair may be skipped only after its descendant work has been scheduled; a previously unseen pair (even with identical pointers) must inspect its scalar descendants to preserve `NaN != NaN`. Each tuple-pair/field comparison consumes fuel. It must charge the existing **1,000,000-step runtime budget** for comparison work and terminate with **E4006** if exhausted; no equality path may perform unchecked exponential traversal. Host-side `PartialEq`/`Debug` must not accidentally reintroduce recursive traversal or bypass these rules for source-visible comparisons. Tuple construction charges at least per constructed field (and existing expression ticks); large distinct values remain bounded by fuel and actual allocation work. Formatting is intentionally opaque and constant-cost for tuples, so the source language has no recursive formatting expansion. No new budget is silently imposed on 0.1 scalar operations. Allocation failure must not be transformed into an incorrect successful program result. Implementation must prove a bounded response to adversarial shared-DAG construction/equality and maintain the existing call-depth and effect propagation gates.

## Rejected alternatives

Type constructors without an accepted value family have no executable consumer and would prematurely decide builtin versus user-defined constructor resolution and generic application parsing. `List<T>`, `Set<T>`, and `Array<T,N>` add substantial distinct storage, update, indexing, equality, and type-argument/constant-length questions. Modules change compilation-unit and identity rules; ADTs introduce nominal identity and declarations; user generics introduce substitution and inference. None is a hard prerequisite of a structural tuple. Destructuring and patterns are intentionally separate future decisions: constant projection already makes tuples usable.

## Compiler impact

- **Lexer/grammar:** recognize dot-index token sequence without swallowing chained projections as floats; parse tuples and structural type parentheses with unchanged `()`/grouping semantics; preserve newline boundaries, recursion and recovery.
- **AST:** change `TypeExpr` from a single-name struct to a recursive name/grouping/tuple representation with source spans; add `ExprKind::Tuple` and `ExprKind::TupleProjection { base, index }`, not a generic collection node.
- **Resolver/name binding:** recurse through tuple children and projection bases; resolve no new names, type constructors, or fields; preserve per-function scope resets and source spans.
- **Type system/checker:** add `Type::Tuple` structural equality/display; recursively lower annotated tuple types with guards; infer tuple literals, check constant access/equality, preserve bottom joins and path-sensitive D001 outcomes; keep printable builtins primitive-only.
- **Typed HIR:** explicit tuple construction and constant tuple projection expressions, each with types, spans, and resolved children; no MIR or generic indexing node.
- **Runtime/backends:** immutable shared tuple values, safe constant field reads, left-to-right evaluation, D001 effect propagation through element evaluation, bounded iterative structural equality, E9004 malformed-HIR guards; no source tuple printing or mutability.
- **Diagnostics:** implement accepted E3012/E3013 in the separately authorized implementation campaign; retain existing parser, type, runtime, and internal code ownership without reusing unrelated codes.

## Testing obligations

Positive: `()`, `(x)` grouping, `(x,)`, `(x,y)`, trailing commas, nested structural types, local inference, parameter/return roundtrip, chained projection `t.0.1`, left-to-right single evaluation, snapshot after local reassignment, same-type tuple equality and inequality, `Unit` nesting, UTF-8 spans, printing a projected primitive.

Negative: malformed separators, missing close, stray projection, noninteger/dynamic/negative index, enormous index, non-tuple target E3012, missing field E3013, invalid assignment target E1104, different tuple shapes in `if`/signature/equality, tuple passed to `print` E3007, arity 65, nesting 65, type-expression depth guards, corrupted HIR projection.

Properties: type annotation round-trip/structural identity across nesting, deterministic diagnostics and spans, existing 0.1 and frozen D001 regressions, `Float`/`NaN` semantics in nested shared tuples, `!=` complement to `==`, no runtime evaluation after break/continue/return during element construction, no unchecked recursion or exponential comparisons on shared DAGs. Run existing lexer/parser/compile/runtime fuzz targets with selected tuple, projection, malformed delimiters, depth and arity seeds, under pinned bounded policy. Validate native macOS, Linux, Windows, MSRV, and full workspace quality gates.

## Compatibility impact

**Compatible extension** against frozen D001 behavior. No identifiers become reserved, existing float literals keep their tokenization, and old `()` / `(x)` / grouped type-name behavior is preserved. Newly admitted parenthesized comma expressions/types and tuple-only numeric projections were previously rejected, so neither valid old-source semantics nor its outputs change. Exact diagnostic wording for previously invalid source may change under the existing development policy. Any implementation that reinterprets a previously valid float, call, grouping, assignment, or statement-boundary program violates this decision and requires human review.

## Normative acceptance and implementation boundary

Human acceptance on **2026-10-08** locks this D002 contract. The accompanying normative changes are `spec/GRAMMAR.md` (syntax, limits, dot-number disambiguation), `spec/TYPE_SYSTEM.md` (structural types, inference, effects, equality and printing), `spec/EXECUTION_MODEL.md` (immutable sharing, evaluation order, bounded comparison and malformed HIR), `docs/ERROR_CODES.md` (E3012/E3013 and retained diagnostics) and `docs/STATE_MACHINE.md` (phase transitions and guard ownership). The acceptance ledger is `HYDRA_0_2_D002_ACCEPTANCE.md`.

**D002 IMPLEMENTATION NOT STARTED.** This record authorizes no Rust, tests, corpus, fuzz, dependency, workflow, tag or release changes. Tuple implementation and verification require a separately authorized campaign. Frozen D001 behavior is unchanged; D003 and other collection families remain deferred.
