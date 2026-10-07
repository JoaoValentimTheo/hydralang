# Hydra 0.2 Decision Queue

Status: design queue only. Hydra 0.2 implementation has not started.

No item in this file authorizes implementation. A future campaign must select one coherent feature family, resolve its dependencies through accepted language decision records, update the specification, and implement/validate that family before beginning another large family.

## Control-flow extension

Candidate questions include `break` and `continue` syntax, legal placement, nested-loop targeting, expression/block typing, `Never` interaction, unreachable-code behavior, and runtime/HIR control-effect representation. Dependency: explicit checker model for loop-local control effects.

## Type-constructor foundation

Before parameterized collections or algebraic data types, decide whether Hydra introduces type constructors/generic type arguments in source syntax, how arity is diagnosed, how types are interned/compared, and which parts are user-definable versus builtin-only. Do not smuggle this decision into the first collection parser change.

## Collections: separate candidates

Collections are not one feature and H15 remains deferred. Each candidate needs its own decision covering syntax, type representation, equality, mutability, iteration, indexing, runtime representation, HIR representation, diagnostics, and generic/type-constructor prerequisites.

### `List<T>`

Open questions: homogeneous element typing, literal syntax, mutability model, length/growth operations, indexing bounds behavior, equality, iteration order, ownership/copy behavior, and builtin/API surface.

### Tuple

Open questions: heterogeneous fixed arity, unit/singleton syntax ambiguity, positional access, structural versus nominal typing, equality by element, and whether tuples participate in destructuring only after pattern semantics exist.

### `Set<T>`

Open questions: literal syntax distinct from blocks, hashability/equality requirements, deterministic iteration policy, duplicate handling, mutability, and runtime representation. Set syntax must not be chosen merely by reusing list/tuple delimiters.

### Fixed `Array<T, N>` (optional)

First decide whether fixed arrays are wanted at all. If yes: length in the type, constant-expression rules for `N`, literal/repeat syntax, indexing, equality, mutability, layout guarantees, and relationship to `List<T>`.

## Modules/imports

Decide compilation unit, file/module mapping, namespace model, visibility, import syntax, cycles, module initialization (if any), and stable cross-module symbol identity before changing resolver/HIR.

## Algebraic data types

Decide nominal type identity, struct/enum declaration syntax, construction, field/variant access, equality, exhaustiveness/pattern prerequisites, layout opacity, and module namespace interaction.

## Generics

Depends on type-constructor and nominal-type decisions. Decide source syntax, inference boundaries, monomorphization versus runtime representation, constraints, diagnostics, and whether generic functions/types are introduced separately.

Native code generation, WebAssembly, package management, LSP, REPL, and web tooling remain outside this decision queue until the language-semantic prerequisites relevant to them are explicit.
