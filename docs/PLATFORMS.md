# Platform Policy

Hydra `0.1.0-dev` is intended to be host-portable on the three primary Rust desktop CI hosts: Linux, macOS, and Windows.

Required CI for the 0.1 engineering baseline is:

- Linux: formatting, Clippy, full workspace all-features tests, MSRV check, and bounded fuzz smoke.
- macOS: full workspace all-features tests on stable Rust.
- Windows: full workspace all-features tests on stable Rust.

Fuzzing remains Linux-only because the purpose of the bounded smoke job is compiler/runtime robustness rather than host portability. Platform-specific packaging, installers, filesystem integration, terminal behavior, and performance guarantees are not part of the 0.1 baseline.

A future public release must keep all three host test gates green or explicitly narrow the supported-host claim through a recorded release decision. Adding OS-specific behavior requires targeted tests on the affected host.
