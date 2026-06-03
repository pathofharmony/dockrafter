#!/usr/bin/env bash
# Install the `docrafter` binary into ~/.cargo/bin.
set -euo pipefail
cd "$(dirname "$0")/.."
cargo install --path crates/docrafter-cli --locked "$@"
echo "Installed: $(command -v docrafter 2>/dev/null || echo '~/.cargo/bin/docrafter')"
