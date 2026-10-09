# Hydra 0.1 Type System

Status: **IMPLEMENTED for the Hydra 0.1 core subset**.

Primitive types are `Int`, `Float`, `Bool`, `String`, `Unit`, and `Never`. `Never` represents diverging control flow. It joins to the other branch type when only one branch diverges; two diverging branches remain `Never`. A block whose execution is unconditionally terminated by `return` or a `Never`-typed statement is itself `Never`.

Hydra performs local inference for `let` initializers. Function parameter types are explicit. A missing function return annotation means `Unit`. There are no implicit `Int`/`Float` conversions.

Arithmetic requires matching numeric operands except `String + String`, which concatenates. Relational operators require matching numeric operands. Equality requires matching operand types. Boolean operators require `Bool`.

`Never` is compatible with any expected type because no normal value is produced on that path. Strict unary operations, strict binary operations, assignment right-hand sides, `if` conditions, and function/builtin arguments propagate `Never` when evaluation cannot continue. `&&` and `||` retain short-circuit semantics: a `Never` right operand does not make the whole expression unconditionally `Never` because that operand may not execute.

A successful assignment expression has type `Unit`. An assignment whose right-hand side is `Never` has type `Never`. Chained assignment therefore does not propagate the assigned value in 0.1.

Functions are not first-class values in 0.1. Calls are statically resolved to a user function or central builtin signature, and arity plus argument types are checked before HIR execution.

The builtin registry contains exactly `print` and `println`. Both take one printable argument and return `Unit`. Printable runtime types are `Int`, `Float`, `Bool`, `String`, and `Unit`; `Never` is accepted through bottom-type compatibility. Name resolution is local, then user function, then builtin, so a local or user function may shadow a builtin.

Unreachable source is still type-checked. Hydra 0.1 does not suppress type errors merely because an earlier statement in the same block is known to diverge.

## Hydra 0.2 accepted loop-control and effect contract — D001 implemented and frozen

D001 accepts value-less `break` and `continue` statements for `while` only. The following rules are **normative for Hydra 0.2 and implemented in the current post-D001 checker**. The 0.1-only descriptions above are historical; see `HYDRA_0_2_D001_FREEZE.md` for frozen implementation and regression evidence.

### Placement and distinct outcomes

The checker owns source placement legality: `break` (E3010) and `continue` (E3011) are valid only inside the lexical **body** of a `while` in the current function and target the nearest eligible enclosing loop. The condition of a `while` is checked under its *surrounding* loop context; only its body introduces that `while` as a target. Thus `while { break } { ... }` is invalid unless an outer `while` body supplies the target; if there is one, the condition's `break` targets that outer loop. A nested loop's body provides a nearer target. Block expressions and `if` do not add targets. Every function starts with no inherited loop target, so a function call cannot break/continue the caller's loop. The parser accepts well-formed statements, and the resolver does not duplicate checker legality diagnostics. Unreachable source is still statically checked for placement and type errors.

Normal values, `return`, `break`, `continue`, divergence, and runtime failure are distinct outcomes. `Type::Never` means **no normal value is produced on that path**; it does not identify which control effect occurred. It remains the bottom type for normal-value joins: `Type::join(Never, T) = T`. In particular, a path carrying only `Break` can have no normal value without becoming a `Return` or divergence. Runtime failure is a separate `Result::Err`, never a value or a substitute for control-flow effect analysis.

Effect analysis must independently preserve at least these conceptual path properties (without prescribing a Rust representation):

```text
falls_through  // a normal continuation is possible
returns        // a function return is possible
breaks         // an eligible enclosing loop exit is possible
continues      // an eligible enclosing loop restart is possible
may_diverge    // non-normal completion without one of those control transfers is possible
```

### Sequential composition and conditionals

For sequentially evaluated constructs `A` then `B`, effects of `B` are reachable **only where `A` falls through normally**. Combine non-normal effects of `A` with effects of `B` gated by this reachability, and allow normal continuation only if both stages can fall through. **`B` must still be statically checked** even if unreachable, following Hydra's existing policy. Neither a single `terminated` flag nor `Never` alone can represent this distinction.

An `if` evaluates its condition before selecting a branch; the branches contribute alternative reachable effects. A missing `else` contributes the normal `Unit` path. For `if condition { break } else { continue }`, there is no normal value and the effects include distinct `Break` and `Continue` alternatives. For `if condition { break } else { 5 }`, the normal-value type is `Int` while a possible `Break` survives; `join(Never, Int) = Int` applies only to the normal-value type and must not erase the effect.

### Expression evaluation and short circuit

Strict unary operands, binary operands, assignment right-hand sides, user-function arguments, builtin arguments, `if` conditions, return operands, and block expressions propagate non-normal effects in the existing **left-to-right evaluation order**. If an evaluated operand transfers `Break`, `Continue`, or `Return`, the pending operation does not execute and the transfer propagates. For instance, `print({ continue })` inside a loop does not call `print`; `return { break }` breaks the eligible loop without emitting a function return. Only effects on reachable evaluation paths are propagated, although unreachable source is still checked.

Boolean `&&` and `||` retain short-circuit evaluation. A right-hand loop effect occurs only on runtime paths where the RHS is actually evaluated; the potential to skip it preserves a possible normal-value path. An effectful or `Never`-typed RHS therefore does not by itself render the whole boolean expression unconditionally `Never`. This rule imposes no new constant-folding requirement.

### `while` result and effect ownership

`while` remains **statement-style**, yielding `Unit` on normal completion; `break` has no value. A condition can be false before the body executes, so even a body that always breaks, continues, or returns does not prove that its surrounding `while` is `Never`. No constant-true or termination proof is required. The loop consumes its **body's** reachable `breaks` and `continues`; it propagates reachable `returns` and possible divergence. Effects emitted while evaluating its **condition** belong to the surrounding loop context and are never consumed by the newly entered `while`. If the condition has no possible normal outcome, the loop must preserve that non-normal outcome instead of inventing a normal `Unit` result.

These rules lock the implemented and frozen D001 source-language semantics. No labels, value-carrying control, expression loops, generalized effect system or MIR/SSA change is accepted.

## Hydra 0.2 D002 implemented structural tuple types

**2026-10-08 approved Option A amendment:** All tuple type trees, whether
written or inferred, have maximum structural depth 64 on every path. A
primitive has depth 0; `Tuple(fields)` has depth `1 + max(depth(fields))`;
grouping adds zero. Parser E1105 owns written depth violations; checker E3014
owns inferred violations, pointing to the constructing expression and giving
actual and permitted depths. Checker traversal must account for shared type
subtrees without exponential work. Locals, calls, returns, joins and projections
cannot bypass this invariant. Preserve whole-expression `Never` and frozen
D001 effect rules. The following normative semantics are implemented by D002.

Human acceptance on 2026-10-08 locks the following **normative D002 semantics**; the compiler implements tuple types, expressions and positional projections. The frozen D001 type and path-effect behavior above remains implemented and unchanged.

Tuple types are ordered, fixed-arity, heterogeneous structural products: `(Int, String)` equals only a tuple type with the same ordered element types and arity. Thus `(Int, String)` differs from `(String, Int)`, and `(Int,)` differs from `Int`. Nested forms such as `((Int, String), Bool)` are structural. `()` is `Unit`, `(T)` is grouping of type `T`, and `(T,)` is a singleton tuple type; `(Unit,)` differs from `Unit`. A trailing comma is permitted on nonempty tuple types. No nominal tuple name, tuple width conversion, variance, subtyping, numeric coercion, generic `Name<T>` application, or other collection family is introduced.

The `TypeExpr` representation structurally distinguishes primitive names, grouped types and tuples, with accurate spans. Tuple annotations may appear wherever a type is currently accepted, including parameter, return and local annotations. `let pair = (1, "ready")` independently infers `(Int, String)` in element source order. Annotations and inferred types use the same structural identity for parameters, assignments, returns and branch checking. `Type::join(Never, T) = T` applies at the **whole-expression** level; two ordinarily returning tuple types join only if structurally identical, otherwise the existing incompatible-branch/type-mismatch diagnostic applies. There is no elementwise tuple promotion or coercion.

Tuple construction is strict left to right. Each element is statically checked even if earlier elements prevent execution; runtime evaluates only reachable elements. If an element has no normal completion, the **whole tuple expression** has normal-value type `Never` and preserves the particular D001 path effects (`Return`, `Break`, `Continue`, divergence) independently of its type. A declared structural `Tuple(..., Never, ...)` remains a well-formed but uninhabited element position; it is **not** the same type as whole-expression `Never`. Later elements contribute effects only on normal fallthrough paths, with exactly the frozen D001 sequence/effect rules.

Read-only `t.0` projection requires a normally valued tuple operand, and the checker determines the resulting field type from the static zero-based index. Chaining `t.0.1` projects successively; projection from a whole-expression `Never` base is itself `Never` and propagates existing effects. Projection from a normally valued non-tuple reports **E3012**; an out-of-bounds static index reports **E3013**, with the dot-through-index span. Assignment to `t.0` remains invalid (E1104), because projection is never a mutable place.

Two tuple expressions may use `==` and `!=` only for **identical static tuple types**, following the existing equality operand policy. Equality compares corresponding fields in order, stopping on the first inequality; `!=` is its negation. Nested Float fields retain IEEE semantics (`NaN != NaN`), including when aggregate storage is shared. Tuple `<`, `<=`, `>` and `>=` are invalid (E3003). An entire tuple argument to `print` or `println` is unsupported (existing E3007), because the builtins remain primitive-only; a projected printable primitive remains allowed. No tuple hash, total ordering, source formatting, destructuring or mutation contract is accepted.

Parser checks cap tuple arity at **64** (E1101), tuple-type nesting at **64 tuple layers per path** (E1105), and preserve existing parser syntax/expression depth guards (E1105/E1106). Type traversal of adversarial internal structures must avoid unbounded host recursion. Inferred tuple depth above 64 is checker E3014; the guards are implemented.

## Hydra 0.2 D003 — accepted immutable homogeneous List types (NOT IMPLEMENTED)

This is a **locked contract for a future implementation**, accepted 2026-10-09. The existing checker has no List type or List inference. The frozen D001 effects and D002 tuple-only rules above retain their existing meanings.

`List<T>` is a builtin **type constructor**, recognized only in type positions, with precisely one concrete type argument. Its name remains an ordinary identifier in value positions; no callable `List` constructor, user generics, generic functions, variance, subtyping, implicit conversion, numeric promotion or generalized `Name<T>` application is introduced. `List<Int>`, `List<Float>`, `List<(Int, String)>`, and `List<List<Int>>` have invariant structural type identity. Two normally completed List values are compatible only with exactly matching element types. Unknown type names are E3001; a syntactically well-formed `List<>` or `List<Int, Bool>` reaches checker E3017, not function-call arity E3009. Malformed type delimiters or argument syntax remain parser E1101.

A nonempty `[a, b, ...]` has homogeneous elements with one exact normal static element type; heterogeneity reports E3002. Construction checks elements in lexical order including unreachable elements, but non-normal effects of unreachable elements cannot flow into reachable paths. The first evaluated non-normal element makes that execution path's List construction non-normal, with the **whole expression** typed `Never` where no normal completion exists. D001 `Return`, `Break`, `Continue`, possible divergence and runtime diagnostics retain their separate meanings. There is no runtime `Never` element, implicit elementwise `Never` coercion or fabricated `List<Never>` from a nonempty non-normal construction. An explicitly annotated **empty** `List<Never>` is permitted when `Never` is otherwise valid in type annotations and contains no value of type `Never`.

An empty `[]` cannot infer `T` itself: absent an *exact* permitted expected `List<T>`, checker **E3015** applies. Expected typing is permitted **only** at:

1. An explicitly annotated local initializer, `let xs: List<Int> = []`.
2. An actual argument at the corresponding statically resolved and typed `List<T>` function parameter, `consume([])`.
3. A tail expression or explicit `return []` of a function **explicitly** declaring `-> List<T>`.
4. The RHS of an assignment to an existing mutable local whose static type is `List<T>`, `xs = []`, under existing assignment rules.
5. An immediate element of a directly constructed List/Tuple literal receiving an exact expected aggregate type, e.g. `let xs: List<List<Int>> = [[], []]` or `let pair: (List<Int>, Bool) = ([], true)`. Propagation recurs through **directly constructed aggregate literal children only**, with each child receiving exactly its corresponding structural member type.

The checker must introduce an expectation at the entry to an approved checking context and remove it on **all** exits (including diagnostics/recovery). It must not bleed into other statements, unrelated operands, different variables or functions. An `if` branch containing untyped `[]` is not rescued by an unrelated sibling branch's `List<T>` or an expected type passed through the `if`; no arbitrary bidirectional inference or inference from later assignments or unconstrained generic variables is authorized. Ordinary joins of **already independently typed** identical List types remain allowed; a whole-expression `Never` joins as before. Calls and returns cannot bypass exact type validation.

The structural aggregate type-depth function is, for primitive types including `Never`, **0**; for `List<T>`, `1 + depth(T)`; for `(T1, ..., Tn)`, `1 + max(depth(Ti))`; for grouping parentheses, unchanged. `()` is `Unit` at depth 0. **Maximum combined List/Tuple depth is 64**, including mixed nesting. Written type depth 65 is parser **E1105**; inferred type depth 65 containing a List is checker **E3016** at its first constructing expression (message includes observed and allowed depth); inferred **tuple-only** depth 65 retains frozen checker **E3014**. No successfully checked source-derived typed HIR may exceed 64 through inference, declarations, parameters, results, fields, calls, assignments, branch joins, projections or indexing; every construction and type introduction must enforce the invariant before HIR becomes executable. Shared type-DAG traversal must avoid exponential walking. Pure tuple types accepted under D002 remain valid.

An index expression `base[index]` requires a normally valued base of static `List<T>` (otherwise E3018) and a normally valued `Int` index (otherwise E3019). Its normal result type is `T`; a whole-expression `Never` base or index propagates its actual D001 effects rather than inventing E3018/E3019. Indexing is never an assignable location (parser E1104), and cannot mutate the List. List types support only matching-type `==` and `!=`, preserving ordered structural equality and IEEE NaN, including physical sharing; invalid ordering operators are E3003, incompatible List types are E3002, and `print`/`println` of a whole List are E3007. No hashing, membership, iteration, slicing or serialization follows from acceptance. Runtime evaluation and fuel obligations are in `spec/EXECUTION_MODEL.md`.
