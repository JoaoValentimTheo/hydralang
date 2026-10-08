#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
cd "$repo_root"

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "ERROR: native macOS (Darwin) host required." >&2
  exit 1
fi

echo "Native macOS validation in: $repo_root"
uname -s
uname -m
sw_vers

for toolchain in stable 1.85.0; do
  if ! rustup run "$toolchain" rustc --version; then
    echo "ERROR: required Rust toolchain '$toolchain' is missing; install it separately before running this gate." >&2
    exit 1
  fi
done

cargo +stable fmt --all --check
cargo +stable clippy --workspace --all-targets -- -D warnings
cargo +stable test --workspace --all-features
cargo +1.85.0 check --workspace
cargo +stable run -p hydra-cli -- check examples/hello.hyd
cargo +stable run -p hydra-cli -- run examples/hello.hyd
