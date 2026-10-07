# Hydra 0.1 Language Specification

Status: **IN PROGRESS**.

Hydra is statically and strongly typed, null-free, deterministic, and explicit about mutability. Ordinary statements are separated by newlines; semicolons are not part of the 0.1 statement grammar.

The initial implemented semantic types are `Int`, `Float`, `Bool`, `String`, `Unit`, and `Never`. `Int` is signed 64-bit. Integer arithmetic is checked and reports a Hydra runtime diagnostic on overflow. Integer division and remainder by zero are errors. `i64::MIN / -1` and `i64::MIN % -1` are overflow errors.

Bindings use `let name = expr` or `let mut name = expr`. Immutable assignment is rejected during resolution. Functions use `fn name(param: Type) -> Type { ... }`; omission of a return annotation means `Unit`. Blocks evaluate to their final expression when present, otherwise `Unit`.

Hydra 0.1 currently has no `null`, implicit numeric conversion, generics, traits, macros, package manager, or concurrency model.

