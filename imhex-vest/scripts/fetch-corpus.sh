#!/bin/bash
# Clones the GPL-2.0 ImHex-Patterns corpus (patterns, includes and binary
# fixtures) into vendor/ImHex-Patterns for the corpus tests and sweeps.
# The corpus is not vendored into this CC0 repository.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
mkdir -p "$ROOT/vendor"
if [ ! -d "$ROOT/vendor/ImHex-Patterns" ]; then
  git clone --depth 1 https://github.com/WerWolv/ImHex-Patterns.git "$ROOT/vendor/ImHex-Patterns"
else
  git -C "$ROOT/vendor/ImHex-Patterns" pull --ff-only
fi
echo "corpus at $ROOT/vendor/ImHex-Patterns"
echo "run the sweep with: IMHEX_PATTERNS=$ROOT/vendor/ImHex-Patterns cargo test -p hexpat --test corpus -- --nocapture"
