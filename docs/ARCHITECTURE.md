# Architecture

Hydra uses explicit compiler phases with one-way crate dependencies:

```text
hydra-source
  -> hydra-diagnostics
  -> hydra-lexer
  -> hydra-ast
  -> hydra-parser
  -> hydra-resolve
  -> hydra-check + hydra-types + hydra-stdlib
  -> hydra-hir
  -> hydra-runtime
  -> hydra-cli
```

The parser AST records syntax and spans. Resolution assigns stable IDs to functions and locals. The checker validates primitive semantics and lowers to typed HIR. Runtime execution consumes HIR rather than parser AST.

Resolution lookup order is nearest local, then user-defined function, then builtin. The checker is the source of type truth for HIR, including `Never` propagation and `Unit` assignment semantics; the runtime treats violations of those typed-HIR invariants as E9004 rather than re-resolving or re-type-checking source semantics.

The HIR is intentionally backend-neutral so future native and WebAssembly backends can consume the same resolved, typed semantics.
