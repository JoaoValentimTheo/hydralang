# Roadmap

Hydra 0.1 is the validated primitive semantic core: compiler phase separation, diagnostics, primitive types, functions, current control flow, typed HIR, reference interpreter, CLI, classified corpus, property tests, fuzzing, and the baseline/governance contracts required to preserve that behavior.

Hydra 0.2 is **IN PROGRESS**. Its first family, D001 value-less `break`/`continue`, is **ACCEPTED, IMPLEMENTED, AUDITED and FROZEN** at `764b901ec8a0f0febfb3624c392d27f14b631ff4`; see `HYDRA_0_2_D001_FREEZE.md`. This is a completed family, not a completed 0.2 milestone.

The second-family D002 **design-only gate** selects exactly one candidate, structural **Tuple**, with its decision record in `docs/decisions/002-tuples.md` (**PROPOSED**) and evidence ledger in `HYDRA_0_2_D002_DESIGN_GATE.md`. **Human approval is required; D002 implementation has not started.** Neither this proposal nor roadmap text changes existing grammar, types, runtime or normative contracts. Type constructors, List, Set, optional fixed Array, modules, algebraic data types and generics remain deferred for independent future decisions. Any externally observable change requires the acceptance, specification and compatibility discipline in `docs/VERSIONING.md`.

After human acceptance, one separately authorized implementation campaign may complete the selected family's grammar, types, diagnostics, tests, validation and compatibility review before opening another family. Collections remain H15-deferred and separate in `docs/HYDRA_0_2_DECISION_QUEUE.md`; D003 has not begun.

Native code generation, WebAssembly, package management, and tooling beyond the CLI remain deferred until their language and architecture prerequisites are explicit.
