# Hydra 0.2 Decision Queue

Hydra 0.1 is preserved; Hydra 0.2 is **IN PROGRESS**. This is governance, not implementation authorization. Feature families require an accepted decision, normative specification, source behavior/regression obligations, and independent validation before implementation can be declared complete.

## Frozen first family: D001 loop control

`break` / `continue` is **ACCEPTED, IMPLEMENTED, AUDITED, FROZEN** at `764b901ec8a0f0febfb3624c392d27f14b631ff4`. See `docs/decisions/001-loop-control.md` and `HYDRA_0_2_D001_FREEZE.md`. Its lexical targeting, D001 checker path outcomes, HIR and runtime effects, and E3010/E3011 checks are no longer open candidate questions. D001 is preserved without reopening its contract.

## Selected second family: D002 Tuple — ACCEPTED / NORMATIVE CONTRACT LOCKED

Exactly **one** second-family design is human-approved on 2026-10-08: immutable ordered **structural Tuple** with parenthesized comma literal/type forms, read-only constant positional projection, matching-type equality and bounded shared storage. The **ACCEPTED** and locked contract is `docs/decisions/002-tuples.md`; the historical selection analysis is `HYDRA_0_2_D002_DESIGN_GATE.md`; the approval ledger is `HYDRA_0_2_D002_ACCEPTANCE.md`. The normative grammar, type, execution, lexical, diagnostics and state-machine documents now describe the accepted **future** tuple behavior. **D002 IMPLEMENTATION NOT STARTED.** Production Rust, tests, corpus, fuzzing, releases, tags and D003 remain unauthorized by this acceptance. The next action is a separately authorized D002 implementation campaign.

## Unselected and deferred families

### Type-constructor foundation

Type-argument grammar, constructor arity/identity, lookup and builtin/user-defined boundaries need a concrete accepted consumer. This candidate was assessed independently and deferred: choosing `List<T>` later may motivate its own type-application decision without requiring all user generics in advance.

### `List<T>` — H15 deferred

Open: homogeneous elements, type application, literal and indexing syntax, mutability/ownership, size/growth, bounds diagnostics, equality and builtin/API surface. No List contract is accepted.

### `Set<T>` — H15 deferred

Open: type application, syntax distinct from blocks, duplicate policy and equality, deterministic membership/iteration, mutability and storage. A hashing protocol is optional if a deliberately linear policy is accepted later.

### Fixed `Array<T, N>` — H15 deferred (optional)

First decide whether fixed arrays exist. If selected later, define length-bearing type identity, compile-time integer length, construction, mutability, indexing and bounds. General const generics are not intrinsically required.

### Modules/imports

Open: compilation unit, source/file mapping, namespace and visibility, import cycles, symbol identities across units and module initialization policy. Packages and incremental compilation are separate future work.

### Algebraic data types

Open: nominal `struct`/`enum` identities, declaration and constructor rules, fields/variants, access, equality and runtime representation. Pattern matching, exhaustiveness, generics and modules may be convenient but are not automatically hard dependencies.

### Generics

Open: parameter binders, substitution, inference/instantiation, identity and runtime/backend strategy. Generic functions can be considered independently from generic ADTs; a future builtin `List<T>` does not imply user generics are accepted.

Native code generation, WebAssembly, package management, LSP, REPL and web tooling remain outside this semantic decision queue until their prerequisites are documented. **D003 is not selected or authorized.**
