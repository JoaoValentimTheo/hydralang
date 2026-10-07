# Contributing

Hydra changes should preserve the pipeline invariant `SPEC -> GRAMMAR -> AST -> RESOLUTION -> TYPES -> HIR -> EXECUTION -> TESTS`.

Before committing compiler changes, run formatting, Clippy with warnings denied, and the workspace tests. Confirmed defects should receive regression coverage. User-controlled source must not reach `panic!`, `todo!`, `unimplemented!`, `unwrap`, or `expect` in production compiler paths.

Language changes must update the relevant files under `spec/` and `docs/` in the same change.

