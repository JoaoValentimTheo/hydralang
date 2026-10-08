# CI Platform Policy

Hydra uses complementary platform gates: GitHub Actions runs Linux formatting, Clippy, workspace tests, MSRV 1.85, and bounded fuzz smoke (pinned nightly `nightly-2026-10-02` and `cargo-fuzz 0.13.2`), plus Windows workspace tests. Maintainer-native macOS validation runs `./scripts/ci-macos.sh` on a maintained Mac before designated baseline, release, and semantic-closeout pushes.

GitHub-hosted macOS jobs repeatedly queued and were cancelled without running any test steps. In particular, workflow `37792710864` reported that the job was not acquired by a hosted runner after multiple attempts. This is infrastructure evidence, not evidence of a compiler failure. The hosted macOS job was therefore removed from remote CI; a successful native local run is recorded separately and is not a GitHub status check.

A maintainer's personal Mac is not configured as a public repository self-hosted runner: that would expose the personal machine to repository-triggered workflows and increase operational/security obligations.

Local native macOS validation and GitHub Actions remote CI are complementary gates. Neither may be silently omitted when the campaign requires both.

Revisit this policy if GitHub-hosted macOS runner acquisition becomes reliable or an appropriately isolated managed runner becomes available. Any change requires an explicit platform-policy decision and successful replacement validation; it does not alter Hydra language semantics or the supported-host intent.
