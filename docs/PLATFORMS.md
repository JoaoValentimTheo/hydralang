# Platform Policy

Hydra `0.1.0-dev` is intended to be host-portable on Linux, macOS, and Windows.

Required automated remote GitHub Actions CI for the 0.1 engineering baseline is:

- Linux: formatting, Clippy, full workspace all-features tests, MSRV check, and bounded fuzz smoke.
- Windows: full workspace all-features tests on stable Rust.

Required native local platform gate:

- macOS: run `./scripts/ci-macos.sh` on an actual maintained Mac. This verifies formatting, Clippy, all-features workspace tests, MSRV, and CLI hello check/run. The local gate is mandatory before designated baseline, release, or semantic-closeout pushes; its result does not appear as a GitHub-hosted status check.

Fuzzing remains Linux-only because the purpose of the bounded smoke job is compiler/runtime robustness rather than host portability. Platform-specific packaging, installers, filesystem integration, terminal behavior, and performance guarantees are not part of the 0.1 baseline.

A future public release must have successful Linux and Windows remote gates and the native macOS gate, or explicitly narrow the supported-host claim through a recorded release decision. Adding OS-specific behavior requires targeted tests on the affected host.

GitHub-hosted macOS validation was removed after repeated runner acquisition failures blocked otherwise-green remote CI. The move changes validation infrastructure policy only; it does not change Hydra semantics or macOS support. See `docs/CI_POLICY.md`.
