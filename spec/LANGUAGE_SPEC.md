# Hydra 0.1 Language Specification

Status: **IMPLEMENTED for the Hydra 0.1 core subset**.

Hydra is statically and strongly typed, null-free, deterministic, and explicit about mutability. Ordinary statements are separated by newlines; semicolons are not part of the 0.1 statement grammar. Newlines may be consumed as layout only in grammar positions that explicitly allow it, including parenthesized expressions, calls, and the continuation after a binary operator.

The implemented semantic types are `Int`, `Float`, `Bool`, `String`, `Unit`, and `Never`. `Never` is the bottom type used for expressions that cannot complete normally, such as a block path terminated by `return`.

`Int` is signed 64-bit. Integer arithmetic is checked and reports E4004 on overflow. Integer division and remainder by zero report E4005. `i64::MIN / -1` and `i64::MIN % -1` are overflow errors. Integer tokens are parsed as non-negative `i64` values and unary `-` is a separate operator, so the source spelling `-9223372036854775808` is rejected with E1102 because `9223372036854775808` is already outside the literal range. The minimum value remains constructible, for example as `-9223372036854775807 - 1`.

`Float` uses Rust `f64` operations and IEEE-754 behavior. Floating-point division by zero does not use the integer E4005 path: finite nonzero values divided by zero produce signed infinity, `0.0 / 0.0` produces NaN, and NaN is unequal to itself. Hydra 0.1 does not add implicit numeric conversions or a separate floating-point exception mode.

Bindings use `let name = expr` or `let mut name = expr`. Immutable assignment is rejected during resolution. A successful ordinary assignment expression has type `Unit`; if its right-hand side is `Never`, the assignment expression is also `Never`. Consequently, chained assignment is not a value-propagating construct in 0.1: an inner successful assignment produces `Unit`, which will not satisfy an outer assignment expecting another value type.

Functions use `fn name(param: Type) -> Type { ... }`; omission of a return annotation means `Unit`. Blocks evaluate to their final expression when present, otherwise `Unit`. Function arguments evaluate left-to-right.

Lexical scopes permit nested shadowing but reject duplicate parameters or bindings in the same scope. Name lookup is deterministic: the nearest local wins, then a user-defined function, then a builtin. This means locals can shadow functions or builtins, and user functions can shadow builtins.

`return` exits the current function, and control-flow expressions whose reachable branches all return have type `Never`. Source appearing after a statically diverging statement is still parsed, resolved, and type-checked in 0.1; it can therefore produce diagnostics even though runtime execution cannot reach it. Hydra 0.1 does not define an unreachable-code warning.

The only builtins in 0.1 are `print(value)` and `println(value)`. Each takes exactly one printable argument and returns `Unit`; `println` appends a newline while `print` does not. `Int`, `Float`, `Bool`, `String`, and `Unit` are printable runtime values. `Never` is accepted by the type checker through bottom-type compatibility, but a `Never` argument cannot complete evaluation and therefore the builtin call is not reached on that path.

Hydra 0.1 currently has no `null`, implicit numeric conversion, generics, traits, macros, package manager, or concurrency model.
