#!/usr/bin/env bash
# Run the full local quality gate (same as CI). Usage: ./scripts/check.sh
set -euo pipefail
cd "$(dirname "$0")/.."

export RUSTFLAGS="-D warnings"
export CARGO_TERM_COLOR=always

echo "==> fmt"
cargo fmt --all -- --check

echo "==> clippy"
cargo clippy --workspace --all-targets -- -D warnings

echo "==> test"
cargo test --workspace --all-targets

echo "==> doc"
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps

echo "==> deny"
if command -v cargo-deny >/dev/null 2>&1; then
  cargo deny check
else
  echo "skip: install cargo-deny for license/advisory checks (cargo install cargo-deny)"
fi

echo "OK"
