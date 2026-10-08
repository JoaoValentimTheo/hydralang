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

Future `Break` and `Continue` can extend the same control-flow effect model without changing Hydra 0.1 today: the runtime flow representation can gain loop-control variants, loops can consume the variant targeted at themselves, and other expression/block layers can propagate it. The checker will then need a richer control-effect analysis than the current `Never`/statement-divergence summary so that loop exits and function returns are not conflated. This is known extension pressure, not a reason to implement those constructs in 0.1.

### Hydra 0.2 accepted future transition (D001; implementation pending)

The normative D001 pipeline adds a distinct control state at each existing layer:

```text
source break / continue statement
-> checker validates enclosing while-body target and tracks distinct path effect
-> dedicated typed HIR statement carrying source span
-> runtime Flow::Break / Flow::Continue
-> nearest eligible while consumes its own body effect
```

The checker tracks normal fallthrough, return, break, continue, and potential divergence separately from `Never`'s normal-value bottom typing. Only a `while` **body** introduces its loop target: while checking/evaluating a new `while` condition, the surrounding loop target remains active. Condition effects are propagated and must not be consumed by the new loop. Functions reset loop context and are barriers to escaped effects. Invalid typed HIR that escapes `Break` or `Continue` across a function boundary reports E9004 with the keyword span. Future runtime transitions must charge the existing per-iteration budget tick on `Continue` before reevaluating the condition (E4006 on exhaustion). These are **accepted future transitions**, not Hydra 0.1 implementation behavior.

## Function-call lifecycle

For a user function, runtime execution follows this order:

```text
function lookup
-> arity invariant check
-> call-depth guard
-> fresh frame creation
-> parameter binding
-> call_depth increment
-> body evaluation
-> Flow::Value / Flow::Return / diagnostic
-> call_depth decrement
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
