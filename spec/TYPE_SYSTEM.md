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
