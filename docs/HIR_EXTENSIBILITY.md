# Typed HIR Extensibility Review

This is a read-only Hydra 0.1 architectural review. It records confirmed extension pressure without introducing future syntax, MIR, SSA, or speculative abstractions.

The current typed HIR separates source parsing/resolution/type checking from execution. `HirExpr` carries resolved `SymbolId`/`FunctionId`, stable `BuiltinId`, static `Type`, and source spans. The runtime consumes HIR and does not repeat source name lookup or type checking. That remains a sound boundary for additional backends.

## Break and continue

The HIR currently has statement variants for `While` and `Return`, while runtime control effects are `Flow::Value` and `Flow::Return`. Adding loop exits later does not require replacing HIR: dedicated statement/effect variants can extend it. The confirmed pressure is in control-effect analysis. The checker currently summarizes divergence primarily through `Never` and `stmt_diverges`; `break`, `continue`, and nested loops will require explicit loop-context validation and a richer effect summary so function return, loop exit, and ordinary divergence remain distinct.

## Collections

HIR expression nodes can carry additional typed literal/construction/indexing operations without changing the phase boundary. The missing prerequisite is semantic design, not an HIR container abstraction. List, Tuple, Set, and any fixed Array must first receive independent decisions for type identity, equality, mutability, iteration, indexing, and runtime representation. Introducing one generic "collection" HIR node before those decisions would encode accidental semantics.

## Structs and enums

Stable nominal type/constructor identity will eventually be needed across resolver, type system, HIR, and runtime/backends. Current `Type` and HIR only model the primitive 0.1 universe, so nominal declarations are genuine extension pressure. No ID scheme or layout abstraction is justified until declaration/name-space semantics are decided.

## Modules

Current `FunctionId` and `SymbolId` are program-local, and resolution assumes one program namespace plus lexical scopes. Modules/imports will pressure symbol identity, namespace qualification, visibility, and compilation-unit ownership. Those are resolver/type-system decisions that should precede HIR changes. Nothing in current HIR requires modules to be interpreter-specific.

## Future backends

HIR contains semantic operations and types rather than interpreter closures or host-language callbacks. `BuiltinId` is also backend-neutral: a backend can lower the same identity without source-string dispatch. The main backend pressure is that current HIR preserves structured control flow (`If`, `While`, blocks) rather than a lower control-flow graph. That is suitable for the reference interpreter and simple future lowering passes. A MIR/SSA layer may become useful for optimization or native code generation later, but Hydra 0.1 has no evidence requiring one now.

The current HIR therefore has no blocking architectural trap for the listed future families. Confirmed future work belongs in decisions around control effects, nominal identity/namespaces, collection semantics, and optional lower IR for code generation.
