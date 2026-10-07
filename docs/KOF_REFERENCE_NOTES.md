# Kof4j Reference Notes

Hydra was designed independently. Kof4j is used only as a read-only architectural reference.

## ADOPT

- **One language frontend feeding backend-neutral semantic IR.** Kof separates the language/frontend from multiple execution targets. Hydra adopts the principle with a small typed HIR so the interpreter is only the first consumer.
- **Golden program testing.** Kof keeps executable golden corpora. Hydra adopts pass/fail program corpora as the language surface grows.

## ADAPT

- **Compiler phase separation.** Kof has lexer, parser, semantic analysis, type system, IR, and target backends. Hydra uses finer Rust crate boundaries around source provenance, diagnostics, AST, resolution, checking, HIR, and runtime while keeping the graph acyclic.
- **Direct execution over shared frontend state.** KofScript executes the shared compiler IR. Hydra's 0.1 interpreter executes typed HIR directly; later code generators should consume equivalent semantics.

## AVOID

- **Large target surface during foundation work.** Kof's mature repository supports many backends and platform subsystems. Hydra 0.1 deliberately avoids backend proliferation until semantics are stable.
- **Target-specific semantics leaking upward.** Hydra's HIR and checker must remain independent of interpreter storage or a future native ABI.

## DEFER

- Native, JavaScript, WebAssembly, JVM-style, and other multi-target work.
- Broad standard library, concurrency, web/database/UI subsystems, and package tooling.

Reference inspected: `KofLang/Kof4j`, including repository layout, README architecture overview, golden tests, language-reference organization, and backend separation.

