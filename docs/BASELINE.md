# Hydra 0.1 Semantic Baseline

Status: engineering baseline freeze in progress. No public Hydra release or compatibility tag exists.

- Authoritative pre-freeze checkpoint: `2a4ea08e3b3856437ba2ccac2ebb776e374ecf92`
- Frozen semantic implementation commit: `22d8cf99f3f355e00c5bc5ae299d0b268e853394`
- Package version: `0.1.0-dev`
- Rust edition: 2024
- MSRV: Rust 1.85.0

This document identifies the accepted 0.1 engineering semantics for future comparison. The normative details remain in `spec/`; this file is deliberately a compact index.

## Language surface

Hydra 0.1 supports primitive literals, lexical `let`/`let mut` bindings, assignment, blocks, functions and calls, `if`/`else`, `while`, `return`, unary operators, binary arithmetic/comparison/boolean operators, and the `print`/`println` builtins. Ordinary statements are newline-delimited; semicolons are not statement separators.

Primitive semantic types are `Int`, `Float`, `Bool`, `String`, `Unit`, and `Never`. `Int` is signed 64-bit checked arithmetic. `Float` follows Rust `f64`/IEEE-754 behavior. There are no implicit numeric conversions.

Assignment to an immutable local is rejected during resolution. Successful assignment has type `Unit`; a right-hand side of type `Never` makes the assignment `Never`. `Never` is the bottom type and propagates through strict contexts according to `spec/TYPE_SYSTEM.md`.

Name lookup is nearest lexical local, then user function, then builtin. Nested local shadowing is permitted; duplicate declarations in one scope are rejected. Locals and user functions may shadow builtin names.

Function arguments evaluate left-to-right. Runtime entry is a `main` function with zero parameters. The reference interpreter propagates function return as an internal control-flow effect rather than a value.

The only 0.1 builtins are `print(value)` and `println(value)`. Stable `BuiltinId` identity is resolved before HIR; the shared stdlib registry owns source names, parameter contracts, and return types, while runtime behavior dispatches exhaustively on the ID.

## Resource and structural limits

- Recursive parser syntax nesting: 128 (E1105).
- Constructed expression-tree depth: 256 (E1106).
- Reference-interpreter call depth: 128 frames (E4003).
- Reference-interpreter execution budget: 1,000,000 evaluation steps (E4006).

The interpreter limits are reference-runtime resource policy. The parser limits define what source the 0.1 compiler accepts and protect recursive compiler phases.

## Deferred and unsupported surface

Collections remain H15-deferred. Hydra 0.1 does not include List, Tuple, Set, fixed Array, break/continue, structs, enums, pattern matching, modules/imports, generics, traits/interfaces, closures or first-class functions, async/concurrency, macros, FFI, alternate code-generation backends, package management, registry, LSP, REPL, or web playground.

Future work must follow `docs/VERSIONING.md` and the decision protocol in `docs/decisions/`. The design queue in `docs/HYDRA_0_2_DECISION_QUEUE.md` does not start Hydra 0.2 implementation.
