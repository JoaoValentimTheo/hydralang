# Architecture

Hydra uses explicit compiler phases with one-way crate dependencies:

```text
source -> lexer -> parser/AST -> resolution -> type checking -> typed HIR -> runtime -> CLI
```

`hydra-source`, `hydra-diagnostics`, and `hydra-types` provide shared source/diagnostic/type support. `hydra-stdlib` is the authoritative builtin-definition layer and is consumed by resolution, checking/HIR, and runtime execution. The HIR carries stable builtin identity (`BuiltinId`), so runtime dispatch does not depend on unconstrained source strings.

The parser AST records syntax and spans. Resolution assigns stable IDs to functions and locals. The checker validates primitive semantics and lowers to typed HIR. Runtime execution consumes HIR rather than parser AST.

Resolution lookup order is nearest local, then user-defined function, then builtin. The checker is the source of type truth for HIR, including `Never` propagation and `Unit` assignment semantics; the runtime treats violations of those typed-HIR invariants as E9004 rather than re-resolving or re-type-checking source semantics.

The HIR is intentionally backend-neutral so future backends can consume the same resolved, typed semantics. `docs/HIR_EXTENSIBILITY.md` records the current extension pressure without committing Hydra 0.1 to a MIR/SSA layer or any future backend.
