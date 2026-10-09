# Hydra 0.1 State-Machine Contract

This document records the state transitions already implemented by the Hydra 0.1 compiler and reference interpreter. It is a contract for invariants and regression tests, not a requirement to introduce a generic finite-state-machine framework.

## Parser

The parser state is the token slice, `current` token position, recursive syntax `depth`, a synthetic EOF token, and accumulated diagnostics. Operationally it moves through reading, local recovery, and completion.

The top-level and block loops obey a progress invariant: each iteration either consumes input, performs a synchronization that advances to a recovery boundary, changes parser state, or terminates. Both loops retain a final `current == before` escape that consumes one token. `peek()` returns a synthetic EOF after the physical token slice ends, so truncated or manually supplied token streams cannot read beyond the slice or loop waiting for a missing EOF token.

Recovery is bounded by input size. Top-level recovery seeks the next viable function boundary; block recovery seeks a newline, `}`, or EOF. Recovery must never remain indefinitely on one token. Malformed input without a physical EOF is covered by a regression that feeds hundreds of non-progressing-looking tokens and requires bounded completion.

Two independent depth guards protect recursive downstream consumers. `MAX_PARSE_DEPTH = 128` limits recursive syntax nesting and produces E1105. `MAX_EXPR_DEPTH = 256` validates each constructed expression tree iteratively and produces E1106. The second guard is necessary because a long left-associative Pratt chain can create a deeply left-heavy AST while parser recursion itself stays shallow. Source-derived AST passed to resolver/checker therefore has bounded expression depth. Nested blocks are independently bounded by the syntax-depth guard.

`with_depth` increments before a nested parse and decrements on every ordinary success or failure return from the nested parser callback. Reaching the limit diagnoses and returns without incrementing. A depth failure therefore cannot poison the parser state for subsequent work.

## Resolver and checker

Resolution has two layers of state: program-wide function identity and per-function lexical scopes. The lifecycle is:

```text
collect function identities
-> clear lexical scope stack
-> create function scope
-> declare parameters
-> resolve body and nested scopes
-> discard function scope state
-> next function
```

Nested blocks push and pop one lexical scope. Lookup walks scopes from innermost to outermost, then user functions, then builtins. Mutability belongs to each resolved local declaration and therefore follows lexical scope rather than source spelling alone. Reusing a local name in another function or nested scope cannot inherit the previous declaration's mutability or identity.

The checker retains program-wide function signatures but resets `local_types` and `current_return` before checking each function. Its lifecycle is:

```text
collect signatures
-> clear local type state
-> install current function return type
-> bind parameter types
-> check/lower body
-> validate body type against function return type
-> next function
```

Diagnostics from one function remain in the result, but semantic state from that function must not affect later functions. Regressions cover duplicate/failing functions followed by valid functions, repeated local names across functions, nested scopes and mutability, a `Never`-producing function followed by ordinary checking, and a type error followed by a valid function.

## Values, `Unit`, `Never`, `Return`, and diagnostics

These concepts occupy different layers and must not be conflated:

- An ordinary value is a runtime `Value` and a statically typed expression result.
- `Unit` is a real semantic type and runtime value `()`. Successful assignment and statements without another value use it.
- `Never` is a static bottom type for paths that cannot complete normally. It has no runtime `Value` variant.
- `Return` is a runtime control-flow effect represented by `Flow::Return(Value)`, not a Hydra value or type.
- A runtime diagnostic is an error path (`Result::Err`) and is neither `Value` nor `Return`.

The runtime's ordinary expression result is `Flow::Value(Value)`. `Flow::Return` propagates outward through strict expression evaluation, blocks, conditional branches, loops, and pending calls until the current function call consumes it and converts the carried value into that call's result. A block stops immediately on `Return`; `if` returns the selected branch flow; `while` propagates `Return` from its condition or body; argument evaluation propagates `Return` before a call is made.

Hydra 0.2 D001 has extended this control-flow model with `Break` and `Continue`, implemented under a separately accepted and frozen contract. The description of 0.1 above is historical. D001's checker tracks distinct path effects rather than conflating loop exits with `Never`, statement termination or function returns.

### Hydra 0.2 D001 accepted transition — implemented and frozen

The normative D001 pipeline adds a distinct control state at each existing layer:

```text
source break / continue statement
-> checker validates enclosing while-body target and tracks distinct path effect
-> dedicated typed HIR statement carrying source span
-> runtime Flow::Break / Flow::Continue
-> nearest eligible while consumes its own body effect
```

The checker tracks normal fallthrough, return, break, continue, and potential divergence separately from `Never`'s normal-value bottom typing. Only a `while` **body** introduces its loop target: while checking/evaluating a new `while` condition, the surrounding loop target remains active. Condition effects are propagated and must not be consumed by the new loop. Functions reset loop context and are barriers to escaped effects. Invalid typed HIR that escapes `Break` or `Continue` across a function boundary reports E9004 with the keyword span. Runtime transitions charge the existing per-iteration budget tick on `Continue` before reevaluating the condition (E4006 on exhaustion). These are **implemented post-D001 transitions**, verified in `HYDRA_0_2_D001_FREEZE.md`.

### Hydra 0.2 D002 tuple transitions — implemented

The separately accepted normative D002 pipeline extends existing phase transitions without weakening their invariants. The following transitions are implemented and locally validated in D002:

```text
source tuple/type/projection syntax
-> lexer: Dot + decimal Int after Dot; ordinary Float remains unchanged
-> parser/AST: parenthesized unit/group/tuple disambiguation;
               recursive tuple type with 64-layer limit;
               tuple literal and constant projection nodes with spans
-> resolver: recurse into tuple elements and projection bases under existing scopes
-> checker: structural ordered type identity, inferred elements, typed projection,
            D001 left-to-right path effects, matching-type equality
            infer structural type -> tuple depth <= 64?
                valid -> typed HIR; invalid -> E3014 (checker, constructing span)
-> typed HIR: explicit tuple construction/projection, with spans and types
-> runtime: evaluate elements left to right into shared immutable storage;
            project checked constant field; bounded, fuel-charged tuple equality
-> normal Value / D001 Return-Break-Continue / diagnostic
```

**Lexer state:** after a projection `Dot`, a numeric index token consumes digits alone; it cannot absorb a following dot and digits as a Float. Ordinary float lexing and newline tokenization remain intact. This contextual distinction must reset after processing the index, including malformed syntax/recovery, and must never leak to unrelated numeric tokens.

**Parser state:** `()` stays Unit, `(expr)` grouping, and `(expr,)` a singleton tuple. A nonempty tuple requires a comma. Tuple types have matching grouping/Unit/singleton rules. At most 64 tuple fields are accepted (E1101); type paths have at most 64 tuple layers (E1105). Existing `MAX_PARSE_DEPTH = 128`, `MAX_EXPR_DEPTH = 256`, synthetic EOF, parser progress, depth restoration and bounded synchronization remain unchanged. The iterative expression-depth walker must traverse **every** tuple element and projection base (E1106), including malformed/recovery paths. Oversized decimal projection indices report E1102 without truncation; assigning to a projection reports E1104.

**Resolver/checker state:** tuple elements are visited in source order, and projections traverse their base without introducing lexical scopes or mutable locations. Per-function symbol/type/effect contexts reset as before. The checker infers and matches ordered tuple types exactly; E3012 diagnoses projection on a normally valued non-tuple, E3013 diagnoses out-of-range access, both with dot-through-index spans. A base with no normal value propagates the original D001 effect, independently of `Never` typing. Unreachable source is still checked, but unreachable expression effects must not become reachable path outcomes. Under the **2026-10-08 human-approved Option A amendment**, every inferred structural tuple path has at most 64 tuple layers (primitive depth 0; tuple depth 1 + maximum child depth), including locals, argument/return flows, joins, and projections. At 65 layers, the checker issues **E3014** with actual and allowed depths and the constructing expression's primary span; a successful typed HIR cannot contain an over-limit source-derived tuple type. Explicit type annotations above 64 remain parser-owned **E1105**. Runtime **E9004** is only a defensive rejection of malformed internal HIR, not a normal source-derived nesting diagnostic. The checker depth walk memoizes shared tuple subgraphs to avoid exponential traversal.

**HIR/runtime state:** construction and projection have dedicated typed HIR representations. The interpreter never exposes a partially initialized tuple. Each completed element snapshots its value; aggregate storage is shared and immutable across reads/calls/returns. A nested projection or tuple equality propagates Return/Break/Continue and errors according to frozen D001 semantics. Equality uses an iterative tuple-pair worklist and pair memoization that never skips first-visit descendant scalar equality, including shared NaN. Pair/field work consumes the existing 1,000,000-step fuel (E4006); malformed typed HIR or aggregate shape reports E9004 rather than panicking or indexing unchecked. Nonrecursive opaque host display, and rejection of source whole-tuple `print`/`println`, prevent accidental recursive formatting.

The D002 runtime additionally checks each function argument against its declared HIR parameter type before binding and checks a normal function result against the declared return type after restoring call depth. Tuple shape and nested fields are checked iteratively with existing fuel; malformed HIR returns source-spanned E9004 instead of silently carrying a wrongly typed aggregate across the function boundary.

Acceptance of D002 defines the locked transition obligations. D002 source, integration and runtime regression tests now exercise them; implementation validation is recorded in `HYDRA_0_2_D002_IMPLEMENTATION.md`, and the subsequent independent adversarial audit in `HYDRA_0_2_D002_AUDIT_AND_FREEZE.md`. Frozen D001 behavior, parser state guards, per-function resets, runtime call-depth restoration and diagnostic unwinding retain their required semantics. The D002 technical freeze was confirmed by five successful GitHub Actions jobs at exact SHA `f91d9829c6964fed8e63b97b0a8a05f7a255e58c` (run `37866987551`).

## Function-call lifecycle

For a user function, runtime execution follows this order:

```text
function lookup
-> arity invariant check
-> call-depth guard
-> fresh frame creation
-> runtime argument/type invariant validation (D002)
-> parameter binding
-> call_depth increment
-> body evaluation
-> Flow::Value / Flow::Return / Flow::Break or Flow::Continue (invalid across function boundary: E9004) / diagnostic
-> call_depth decrement
-> normal result/type invariant validation (D002)
-> frame destruction
-> caller continuation or diagnostic propagation
```

The implementation checks the depth guard before incrementing and restores `call_depth` immediately after body evaluation, before propagating a possible diagnostic. A dedicated regression calls a valid function and a malformed-HIR function through the same interpreter and requires `call_depth == 0` after both paths.

Builtin calls use `BuiltinId`, fetch the authoritative signature from `hydra-stdlib`, validate HIR arity against that signature, and dispatch exhaustively by the stable ID. Adding a builtin ID without runtime behavior makes the runtime match non-exhaustive at compile time.

## Execution budget

Each interpreter instance starts with 1,000,000 evaluation steps. `tick(span)` checks for zero, emits E4006 on exhaustion, otherwise decrements by one. Statements and expressions tick at their evaluation boundaries, and loop iterations also tick at the body boundary. This makes non-terminating or pathologically long reference-interpreter execution deterministic and bounded.

The budget is a resource policy of the Hydra 0.1 reference interpreter. It is not an intrinsic semantic requirement for every possible future backend. A future backend may use a different resource mechanism unless a later language decision explicitly elevates a limit into cross-backend language semantics.

## Internal IR boundary

Resolver, checker, and interpreter contain recursive walkers. For source-derived programs, parser structural guards bound the AST that reaches those walkers. Typed HIR is an internal compiler product and inherits that bound. Hand-constructed malformed HIR used by tests is outside the source-language trust boundary; the runtime diagnoses checked invariants such as missing locals, missing functions, arity disagreement, and invalid typed operations with E9004, but Hydra 0.1 does not promise adversarial stack safety for arbitrarily deep externally fabricated HIR.

The relevant regressions live in `hydra-parser`, `hydra-resolve`, `hydra-check`, and `hydra-runtime` unit tests and are exercised by the workspace test gate.

## D003 future List transitions — ACCEPTED, NOT IMPLEMENTED

The following transitions are **normatively locked for future production authorization**; none is part of today's parser/resolver/checker/typed-HIR/runtime implementation. This document remains the **single** state-machine contract. All frozen Hydra 0.1, D001 and tuple-only D002 transitions above continue to apply.

### Parser states and restoration

```text
bracket in primary position -> List literal start
-> parse 0..256 comma-separated expressions, optional final comma
-> consume matching ] -> completed List AST
bracket following an eligible same-statement postfix base
-> parse one index expression -> consume ] -> indexed postfix AST
type-position identifier List followed by < -> parse type arguments
-> close each > (including adjacent >> in nested type positions)
-> checker owns valid syntactic arity != 1
```

Element 257 reports parser E1101 by **element count**, with no unbounded collection expansion. Invalid delimiters, incomplete indices or type punctuation report E1101, and indexed assignment targets E1104. A statement boundary newline must end postfix eligibility, so the next line's `[` begins a new expression. Call `()`, tuple projection `.DIGITS`, and List index `[]` form one left-associative high-precedence postfix chain without changing D002 floats. All List/type subparsers must preserve guaranteed token progress/recovery-to-EOF, restore recursive `depth` on every exit (success, failure, missing `]` or `>`) and preserve the existing expression-depth guard 256. Recursive syntax remains 128 (E1105); written combined List/Tuple depth 65 is E1105 and is checked independently of parser recursion. Tuple arity 64 and recovery behavior are unchanged. UTF-8 spans must describe the correct source boundaries.

### Resolver transitions

```text
enter ordinary expression / existing scopes
-> visit List elements in source order, or index base then index operand
-> resolve names with unchanged lexical -> function -> builtin precedence
-> exit expression with the same scope stack
```

No List scopes, names, `List` value constructor, new shadowing restriction or mutable per-element binding are created; `List` is contextual **type-position-only** syntax. The resolver must check all source children including statically unreachable ones, preserve existing error recovery, and never duplicate a checker-owned index/type diagnostic.

### Checker expectation and type/effect transitions

```text
entry at annotated let / statically typed function actual argument
  / explicitly declared return tail or return expression
  / assignment RHS to existing typed mutable local
-> introduce exact expected List<T> for that direct expression
entry at directly constructed List or Tuple literal with expected aggregate type
-> supply each immediate directly constructed literal child its exact member type
-> recurse only along immediate aggregate-literal edges
all exits (normal, diagnostic, recovery, end function)
-> remove contextual expectation; restore previous type, loop and function state
```

Standalone `[]` without an exact approved expectation is E3015. No expected type propagates from sibling elements, subsequent assignment, arbitrary operators, unrelated branches or enclosing `if` expressions. An independently typed List joins with an identical normal List type, and whole-expression `Never` joins under frozen D001 semantics; it is never used as a fabricated runtime List element. Check each List element, including unreachable children, but gate every child's effects by **normal fallthrough of earlier children**, preserving the distinct Return/Break/Continue/divergence possibilities and runtime errors. Every normal List construction has one exact homogeneous element type (E3002 otherwise). `List<T>` is a single builtin type constructor, with well-formed incorrect arity E3017, without generic binder state.

Before emitting executable typed HIR, validate **every** source-derived aggregate type introduction (locals, annotations, signatures, aggregate fields, inferred expressions, calls, assignments, branch joins, projections, indexes) at combined depth <=64. Written depth 65 belongs to parser E1105, inferred depth 65 containing List to checker E3016, and pure tuple-only inferred depth 65 remains D002 E3014. Share-aware iterative/memoized depth walks prevent repeated DAG expansion. Index typing first checks normal List base (E3018), then normal `Int` index (E3019), and computes the element type, or propagates D001 non-normal effects. Checker state from one function or failed expected-type context must never reach another.

### Runtime transitions and resource accounting

```text
List literal -> validate typed-HIR shape and limits -> charge existing fuel
-> evaluate each element once left-to-right, charging before work
-> propagate non-normal Flow/error without publishing partial aggregate
-> on all normal values, validate types and publish immutable shared List
List index -> evaluate base once -> propagate effects -> evaluate index once
-> propagate effects -> charge/check 0 <= index < len -> element value or E4007
List == / != -> verify matching static types -> iterative mixed List/Tuple
  pair worklist with memoization + fuel for each visited pair/edge
-> compare first-visit descendants even for identical pointers (NaN)
-> return equality result / its exact negation
```

Runtime value validation (including nested List/Tuple, parameter/return boundaries, and source spans) must reject malformed internal HIR/value shapes as E9004 without replacing ordinary valid-source E4007. No deep copying, recursive formatting, partial-publication escape or source-level cycles are permitted. The 1,000,000-step E4006 budget charges construction, indexing, equality, type/value validation and existing D001 loop/call work before expensive expansion; fuel is not a hard global memory cap. Tuple arity 64, List literal maximum 256, structural type depth 64 and call depth 128 remain enforced, with call/frame depth restored after normal, error and effect paths. D001 loop/function barriers and D002 tuple-only behavior are unchanged. Implementation proof obligations are listed in `HYDRA_0_2_D003_ACCEPTANCE.md`; they are **not** executed in this acceptance campaign.
