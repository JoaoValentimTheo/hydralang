# Hydra 0.2 — D001 Acceptance Ledger

- Acceptance date: 2026-10-08
- Starting `main` / `origin/main`: `61af2b53a913ad76e96fd2ff84e8284bb7b5c0d6` (clean, synchronized)
- Starting design-gate Actions run: `37809079720`, **SUCCESS**, 5/5 jobs
- Decision: **APPROVED after independent architectural review** of lexer, parser, AST, resolver, type checker, `Never`, HIR, runtime `Flow`, state-machine invariants, budget, versioning, and diagnostics
- Authoritative ADR: [`docs/decisions/001-loop-control.md`](docs/decisions/001-loop-control.md) — **Accepted**
- Contract: **LOCKED**, documentation/specification-only acceptance

## Contract and compatibility

D001 accepts only reserved, value-less `break` and `continue` statements targeting the nearest enclosing `while` **body** in the same function. Inner-loop conditions retain the outer loop's target. The checker enforces placement; the parser owns syntax; the resolver does not duplicate legality. Distinct normal fallthrough, return, break, continue and divergence effects coexist with `Never` as the normal-value bottom type. Sequential, conditional, strict and short-circuit evaluation preserve reachability and left-to-right order. The interpreter's eventual explicit `Flow::Break`/`Flow::Continue` uses dedicated HIR statements; only a loop's body effects are consumed by that loop. `while` remains statement-style and normally yields `Unit`; function escapes from malformed HIR are E9004; `continue` retains deterministic iteration fuel charging (E4006). See D001 for the complete semantic contract and exclusions.

**Compatibility classification: intentional breaking change.** Hydra 0.1 allows `break` and `continue` as identifier spellings (for example `let break = 1` and `let continue = 2`); reserving these words in Hydra 0.2 invalidates such formerly valid programs. No other valid Hydra 0.1 behavior is intentionally changed. The separate implementation campaign must include regression tests proving the reservation's compatibility boundary and preservation of the 0.1 baseline.

The accepted, reserved checker diagnostic IDs are **E3010** (`break` outside an eligible enclosing `while` body) and **E3011** (`continue` outside an eligible enclosing `while` body). Malformed syntax remains E1101; invalid typed-HIR control escape remains E9004. Codes E3010/E3011 are **not yet emitted** by the 0.1 implementation.

## Acceptance scope and next gate

Updated normative and tracking files:

- `docs/decisions/001-loop-control.md`
- `docs/ERROR_CODES.md`
- `spec/GRAMMAR.md`
- `spec/TYPE_SYSTEM.md`
- `spec/EXECUTION_MODEL.md`
- `docs/STATE_MACHINE.md`
- `AGENT_STATE.md`
- `HYDRA_0_2_D001_ACCEPTANCE.md` (this ledger)

**No production Rust, Cargo, workflows, or test sources change in this campaign.** Hydra 0.1 is preserved; Hydra 0.2 design gate is complete; `break` / `continue` remain **NOT IMPLEMENTED**; Hydra 0.2 implementation is **NOT STARTED**; H15 Collections stays **DEFERRED**. The next gate is **separate human authorization of a D001-only implementation campaign**, including the regression and validation obligations from `docs/VERSIONING.md`. Acceptance of this normative contract alone does not authorize implementation, tag, or release.
