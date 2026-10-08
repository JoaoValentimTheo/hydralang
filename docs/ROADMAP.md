# Roadmap

Hydra 0.1 is the validated primitive semantic core: compiler phase separation, diagnostics, primitive types, functions, current control flow, typed HIR, reference interpreter, CLI, classified corpus, property tests, fuzzing, and the baseline/governance contracts required to preserve that behavior.

Hydra 0.2 is **IN PROGRESS**. Its first family, D001 value-less `break`/`continue`, is **ACCEPTED, IMPLEMENTED, AUDITED and FROZEN** at `764b901ec8a0f0febfb3624c392d27f14b631ff4`; see `HYDRA_0_2_D001_FREEZE.md`. This is a completed family, not a completed 0.2 milestone.

The second-family D002 design gate selected exactly one candidate, structural **Tuple**, with selection evidence in `HYDRA_0_2_D002_DESIGN_GATE.md`. Human approval on 2026-10-08 promoted `docs/decisions/002-tuples.md` to **ACCEPTED / NORMATIVE CONTRACT LOCKED**; the acceptance ledger is `HYDRA_0_2_D002_ACCEPTANCE.md`. The specification now describes the accepted tuple syntax, structural typing, read-only positional projection, immutable sharing and fuel-bounded equality. **D002 implementation has not started**: no tuple behavior is present in the executable compiler. Type constructors, List, Set, optional fixed Array, modules, algebraic data types and generics remain deferred for independent future decisions. Any externally observable change requires the acceptance, specification and compatibility discipline in `docs/VERSIONING.md`.

The next milestone is a **separately authorized D002 production implementation** campaign to complete grammar, types, diagnostics, runtime, tests, validation and compatibility review before opening another family. Other collection families remain H15-deferred and separate in `docs/HYDRA_0_2_DECISION_QUEUE.md`; D003 has not begun or been authorized.

Native code generation, WebAssembly, package management, and tooling beyond the CLI remain deferred until their language and architecture prerequisites are explicit.
