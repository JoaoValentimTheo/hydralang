# Hydra 0.2 Decision Queue — Current State

Hydra 0.1 remains frozen. Hydra 0.2 remains **IN PROGRESS**. Every new family requires human acceptance of its semantic decision, normative specifications, tests and independent validation before implementation or freeze. A proposed design is not authority to implement it.

## Frozen families

- **D001 — loop control:** `break` / `continue` **ACCEPTED, IMPLEMENTED, AUDITED, FROZEN**, checkpoint `764b901ec8a0f0febfb3624c392d27f14b631ff4`. Evidence: `docs/decisions/001-loop-control.md` and `HYDRA_0_2_D001_FREEZE.md`.
- **D002 — structural tuples:** **ACCEPTED, IMPLEMENTED, AUDITED, FROZEN**, checkpoint `f91d9829c6964fed8e63b97b0a8a05f7a255e58c`, five successful exact-SHA CI jobs in run `37866987551`. The human-approved Option A depth amendment is implemented. Evidence: `docs/decisions/002-tuples.md`, `HYDRA_0_2_D002_IMPLEMENTATION.md` and `HYDRA_0_2_D002_AUDIT_AND_FREEZE.md`. The past statement “D002 IMPLEMENTATION NOT STARTED” was correct when the acceptance gate was written but is obsolete as a current status claim. Historical acceptance documents were preserved.

## D003 — candidate selected for human review only

**Immutable homogeneous List** is the architectural recommendation in `HYDRA_0_2_D003_DESIGN_GATE.md`. Draft contract: `docs/decisions/003-list.md`, status **PROPOSED — HUMAN APPROVAL REQUIRED**. **D003 NOT ACCEPTED; D003 IMPLEMENTATION NOT STARTED.** Empty-list contextual typing, bounds/index diagnostics and precise allocation/aggregate depth limits require a human decision. Only the architecture/design documentation campaign is authorized; no normative `spec/` change, diagnostic allocation, Rust implementation or release follows automatically.

## Alternatives independently evaluated and deferred

| Candidate | Deferred reason / remaining decision |
| --- | --- |
| Standalone type constructors | No executable source-visible consumer without another family; only builtin `List<T>` spelling is proposed within D003. |
| Set | Brace/block grammar collision, equality and duplicate-removal costs, membership semantics and deterministic observability. Hashing is optional, not mandatory. |
| Fixed Array | Length-bearing identity, literal shape and element ownership policy; general const generics are not automatically needed. |
| Modules/imports | Units, file maps, namespaces, cycles, visibility, initialization and cross-file symbol identities. Packages/build systems are independent. |
| Algebraic data types | Named type and constructor identity, variants, fields, evaluation/equality; patterns and user generics are optional future decisions. |
| Generic functions | Binders, inference/instantiation, substitution, type/HIR identity and codegen strategy without an accepted polymorphic use case. |

List mutation, iteration, comprehensions, Set, Array and unrestricted type constructors are outside the proposed D003 boundary. Other backends, packages, tooling, releases and tags remain deferred.
