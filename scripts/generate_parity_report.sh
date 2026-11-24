#!/usr/bin/env bash
# Render the live parity comparison as a local, self-contained HTML file.
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
out="${1:-$root/reports/parity/index.html}"
mkdir -p "$(dirname "$out")"
tmp="$out.tmp"
trap 'rm -f "$tmp"' EXIT

cd "$root"
cargo run --release --features parity --bin parity -- --dump \
  | python3 scripts/parity_report.py > "$tmp"
test -s "$tmp"
grep -q 'id="write"' "$tmp"
mv "$tmp" "$out"
trap - EXIT
printf 'Parity report: %s\n' "$out"
