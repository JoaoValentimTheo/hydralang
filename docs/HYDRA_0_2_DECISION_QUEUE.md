# Hydra 0.2 Decision Queue — Current State

Hydra 0.1 remains frozen. Hydra 0.2 remains **IN PROGRESS**. Every new family requires human acceptance of its semantic decision, normative specifications, tests and independent validation before implementation or freeze. A proposed design is not authority to implement it.

## Frozen families

- **D001 — loop control:** `break` / `continue` **ACCEPTED, IMPLEMENTED, AUDITED, FROZEN**, checkpoint `764b901ec8a0f0febfb3624c392d27f14b631ff4`. Evidence: `docs/decisions/001-loop-control.md` and `HYDRA_0_2_D001_FREEZE.md`.
- **D002 — structural tuples:** **ACCEPTED, IMPLEMENTED, AUDITED, FROZEN**, checkpoint `f91d9829c6964fed8e63b97b0a8a05f7a255e58c`, five successful exact-SHA CI jobs in run `37866987551`. The human-approved Option A depth amendment is implemented. Evidence: `docs/decisions/002-tuples.md`, `HYDRA_0_2_D002_IMPLEMENTATION.md` and `HYDRA_0_2_D002_AUDIT_AND_FREEZE.md`. The past statement “D002 IMPLEMENTATION NOT STARTED” was correct when the acceptance gate was written but is obsolete as a current status claim. Historical acceptance documents were preserved.

## D003 — accepted, implemented, independently audited; freeze conditional

**Immutable homogeneous List** is the chosen family, with historical design evidence in `HYDRA_0_2_D003_DESIGN_GATE.md`. The human owner **ACCEPTED — NORMATIVE CONTRACT LOCKED** on 2026-10-09, then authorized and completed implementation, followed by a separately authorized adversarial audit. Authoritative decision: `docs/decisions/003-list.md`; historical acceptance ledger: `HYDRA_0_2_D003_ACCEPTANCE.md`; executable normative contract: `spec/`, `docs/ERROR_CODES.md` and `docs/STATE_MACHINE.md`. Empty `[]` requires one of five narrow exact expected-type contexts; dynamic readonly indexing uses Int and E4007 for bounds; literals cap at 256; combined List/Tuple depth caps at 64, enforcing written E1105, inferred mixed E3016, and frozen tuple-only E3014. E3015–E3019 and E4007 have active emitters. Implementation CI at `12585a294281ff3a9b2d1be0d8e02c453a3eb39f` passed 5/5. **Final D003 semantic freeze depends on exact-SHA 5/5 CI of the independently audited commit.** The audit and conditional freeze evidence is in `HYDRA_0_2_D003_AUDIT_AND_FREEZE.md`. D004, additional collections, Hydra 0.2 releases and tags are not authorized.

## Alternatives independently evaluated and deferred

| Candidate | Deferred reason / remaining decision |
| --- | --- |
| Standalone type constructors | No independently authorized source-visible consumer; builtin type-only `List<T>` is accepted solely within D003. |
| Set | Brace/block grammar collision, equality and duplicate-removal costs, membership semantics and deterministic observability. Hashing is optional, not mandatory. |
| Fixed Array | Length-bearing identity, literal shape and element ownership policy; general const generics are not automatically needed. |
| Modules/imports | Units, file maps, namespaces, cycles, visibility, initialization and cross-file symbol identities. Packages/build systems are independent. |
| Algebraic data types | Named type and constructor identity, variants, fields, evaluation/equality; patterns and user generics are optional future decisions. |
| Generic functions | Binders, inference/instantiation, substitution, type/HIR identity and codegen strategy without an accepted polymorphic use case. |

List mutation, iteration, comprehensions, Set, Array and unrestricted type constructors are outside the accepted D003 boundary. Other backends, packages, tooling, releases and tags remain deferred.
