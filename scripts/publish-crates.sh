#!/usr/bin/env bash
# Publish workspace crates to crates.io in dependency order.
set -euo pipefail
cd "$(dirname "$0")/.."

DRY_RUN=false
if [[ "${1:-}" == "--dry-run" ]]; then
  DRY_RUN=true
  echo "==> pre-publish validation (first release: publish sequentially, not all at once)"
fi

CRATES=(
  docrafter-core
  docrafter-font
  docrafter-layout
  docrafter-office
  docrafter-pdf-write
  docrafter-docx
  docrafter-odt
  docrafter-html
  docrafter-template
  docrafter-ocr
  docrafter-pdf-render
  docrafter-pdf-read
  docrafter
  docrafter-cli
)

if $DRY_RUN; then
  chmod +x scripts/check.sh
  ./scripts/check.sh
  echo "==> package leaf crate (no path deps)"
  cargo package -p docrafter-core --allow-dirty --no-verify
  echo "==> publish order (each step needs prior crates on crates.io):"
  for crate in "${CRATES[@]}"; do
    echo "  cargo publish -p $crate"
  done
  exit 0
fi

for crate in "${CRATES[@]}"; do
  echo "==> publish $crate"
  cargo publish -p "$crate"
done

echo "OK"
