#!/usr/bin/env bash
# The definition of "may land": the same for a person and for afkconvoy.
set -euo pipefail
cd "$(dirname "$0")"

# rustup's toolchain, whatever the shell that started it has on its PATH: a
# Homebrew `cargo` ahead of rustup's lacks rustup's components.
PATH="$HOME/.cargo/bin:$PATH"
RUSTUP_TOOLCHAIN=stable
export PATH RUSTUP_TOOLCHAIN

cargo fmt --all --check
cargo clippy --workspace --all-targets --quiet -- -D warnings
cargo test --workspace --quiet
