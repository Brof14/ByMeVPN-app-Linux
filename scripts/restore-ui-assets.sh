#!/usr/bin/env bash
set -euo pipefail

# Restore the exact >1 MiB Windows artwork, pinned by Git blob SHA-1.
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DEST="$ROOT/ui/assets/globe.png"
EXPECTED="207a571158ecb2d22002b40b3c5a216a24701e63"
SOURCE="https://raw.githubusercontent.com/Brof14/ByMeVPN-app-windows/fa3ac9d88add0badca358c2da194f5755d1e6a9c/ui/assets/globe.png"

if [[ -f "$DEST" && "$(git hash-object "$DEST")" == "$EXPECTED" ]]; then
  echo "Verified UI globe image is already present."
  exit 0
fi
tmp="$(mktemp "$DEST.tmp.XXXXXX")"
trap 'rm -f "$tmp"' EXIT
curl --fail --location --silent --show-error --retry 3 --proto '=https' --tlsv1.2 \
  "$SOURCE" --output "$tmp"
actual="$(git hash-object "$tmp")"
if [[ "$actual" != "$EXPECTED" ]]; then
  echo "ERROR: UI asset mismatch: $actual" >&2
  exit 1
fi
mv -f "$tmp" "$DEST"
echo "Restored exact approved ByMeVPN globe asset."
