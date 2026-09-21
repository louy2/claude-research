#!/bin/bash
# Regenerates vest-formats/vest/*.vest and vest-formats/src/gen/*.rs from
# formats/*.hexpat using the hexpat emitter and the `vest` compiler
# (`cargo install vest`).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
VEST="${VEST:-vest}"
cargo build -q --manifest-path "$ROOT/Cargo.toml" -p hexpat
for pat in "$ROOT"/formats/*.hexpat; do
  name="$(basename "$pat" .hexpat)"
  "$ROOT/target/debug/hexpat" emit-vest "$pat" -o "$ROOT/vest-formats/vest/$name.vest"
  "$VEST" "$ROOT/vest-formats/vest/$name.vest" -o "$ROOT/vest-formats/src/gen/$name.rs" > /dev/null
  echo "generated $name"
done
