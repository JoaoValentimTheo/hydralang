# Hydra Versioning and Compatibility Policy

Hydra is currently `0.1.0-dev`. That identifier means the repository has an engineering baseline but no published compatibility commitment. There is no Hydra 0.1 tag or GitHub release at this stage.

## Pre-release compatibility

Before a tagged public release, syntax and semantics may still change when evidence justifies the change. They must not change silently. An accepted program becoming rejected, a rejected program becoming accepted for intentional semantic reasons, a program producing a different value/effect, or a public builtin changing behavior is an externally observable semantic change.

Every intentional externally observable language change requires, in the same campaign:

1. an update to the normative specification;
2. a decision record explaining rationale and alternatives;
3. positive and/or negative regression coverage that captures the new boundary;
4. an explicit compatibility classification (compatible extension, intentional breaking change, diagnostic-only change, or bug fix restoring the existing contract);
5. the complete repository validation gate.

Bug fixes that only restore already documented semantics do not require a new language decision, but the commit or campaign ledger must identify the violated contract and carry a regression test.

## Diagnostics

Diagnostic codes are phase-oriented and are not reused for unrelated meanings. During `0.x-dev`, wording and source-label detail may improve without compatibility ceremony. Removing a code, changing the semantic condition represented by a code, or moving a user-observable failure to a materially different diagnostic requires specification/ledger review and regression updates. A tagged release may adopt stronger diagnostic-stability guarantees later; none are implied today.

## Syntax, CLI, and internal IR

Syntax and language semantics follow the language-change rule above. CLI flags, exit behavior, and machine-consumed output should be treated as user-visible interfaces; intentional incompatible changes must be recorded in the campaign ledger and tested.

AST and typed HIR are internal Rust interfaces in 0.1 and carry no external compatibility guarantee. Internal refactors still require the normal test/CI gate and must preserve source-language semantics unless accompanied by a language decision.

## Tags and releases

Engineering milestones and baseline commits do not imply publication. A future versioned release requires an explicit release campaign, release decision, clean synchronized repository, green required CI, and an intentionally created tag/release. Agents must never infer authorization to tag or publish from the existence of `0.1.0-dev`, an RC-quality engineering gate, or this baseline document.

Pre-1.0 tags may make narrower compatibility promises than post-1.0 semantic versioning. Any such promise must be written into the release record at the time of publication rather than inferred retroactively.
