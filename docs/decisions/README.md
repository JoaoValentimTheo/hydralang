# Language Decision Records

Hydra uses repository-local decision records for intentional language-surface changes. The purpose is to settle semantics before implementation and preserve why a boundary changed.

A decision record is required for changes to syntax, type-system behavior, evaluation semantics, name resolution, collection semantics, control flow, modules, generics, or public builtin behavior. Trivial implementation refactors, documentation corrections, test-only changes, and bug fixes that restore an already normative contract do not require a new record.

Copy `000-template.md`, assign the next available numeric prefix, and keep the decision in the same change set as the specification and regression updates it governs. A record may begin as `Proposed`; implementation must not claim the language change as accepted until the record is `Accepted`.

If later evidence reverses a decision, add a new record that supersedes the old one rather than rewriting historical rationale.
