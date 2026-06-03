#!/usr/bin/env bash
# Refresh PDF/DOCX snapshot fixtures after intentional layout changes.
set -euo pipefail
cd "$(dirname "$0")/.."

export DOCRAFTER_UPDATE_SNAPSHOTS=1
cargo test -p docrafter
cargo test -p docrafter-layout
cargo test -p docrafter-pdf-write
unset DOCRAFTER_UPDATE_SNAPSHOTS

echo "Snapshots updated."
