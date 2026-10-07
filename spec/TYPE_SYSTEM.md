# Hydra 0.1 Type System

Status: **IN PROGRESS**.

Primitive types are `Int`, `Float`, `Bool`, `String`, `Unit`, and `Never`. `Never` is the type of a `return` expression and coerces to the other branch type when joining control-flow branches.

Hydra performs local inference for `let` initializers. Function parameter types are explicit. A missing function return annotation means `Unit`. There are no implicit `Int`/`Float` conversions.

Arithmetic requires matching numeric operands except `String + String`, which concatenates. Relational operators require matching numeric operands. Equality requires matching operand types. Boolean operators require `Bool`.

