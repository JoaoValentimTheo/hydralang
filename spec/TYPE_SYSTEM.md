# Hydra 0.1 Type System

Status: **IMPLEMENTED for the Hydra 0.1 core subset**.

Primitive types are `Int`, `Float`, `Bool`, `String`, `Unit`, and `Never`. `Never` represents diverging control flow. It joins to the other branch type when only one branch diverges; two diverging branches remain `Never`. A block whose execution is unconditionally terminated by `return` or a `Never`-typed statement is itself `Never`.

Hydra performs local inference for `let` initializers. Function parameter types are explicit. A missing function return annotation means `Unit`. There are no implicit `Int`/`Float` conversions.

Arithmetic requires matching numeric operands except `String + String`, which concatenates. Relational operators require matching numeric operands. Equality requires matching operand types. Boolean operators require `Bool`.

Functions are not first-class values in 0.1. Calls are statically resolved to a user function or central builtin signature, and arity plus argument types are checked before HIR execution.
