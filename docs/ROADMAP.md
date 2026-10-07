# Roadmap

Hydra 0.1 is the validated primitive semantic core: compiler phase separation, diagnostics, primitive types, functions, current control flow, typed HIR, reference interpreter, CLI, classified corpus, property tests, fuzzing, and the baseline/governance contracts required to preserve that behavior.

The next milestone is a Hydra 0.2 design/decision gate. It may analyze candidate feature families and their dependencies, but design work does not authorize implementation. Any externally observable language change must first satisfy `docs/VERSIONING.md` and the decision process in `docs/decisions/`.

After an accepted design decision, implementation proceeds one coherent feature family at a time. Candidate families include control-flow extensions, a type-constructor foundation, collections, modules, algebraic data types, and generics. Their order must be chosen from dependency evidence rather than this list.

Several large language families must not be implemented in one campaign. Complete the selected family's semantics, specification, diagnostics, tests, validation, and compatibility review before opening another implementation family. Collections remain H15-deferred and are further separated into independent candidates in `docs/HYDRA_0_2_DECISION_QUEUE.md`.

Native code generation, WebAssembly, package management, and tooling beyond the CLI remain deferred until their language and architecture prerequisites are explicit.
