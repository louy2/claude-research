#!/bin/bash
# Regenerates vest-formats/vest/*.vest and vest-formats/src/gen/*.rs from
# formats/*.hexpat using the hexpat emitter and the `vest` compiler, and
# interval-vest/src/gen/zip_regions.rs from its hand-written .vest file.
# The generated code targets vest_lib 0.2.1, so use the matching compiler:
# `cargo install vest --version 0.2.1` (0.2.2 emits code for vest_lib 0.2.2).
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
"$VEST" "$ROOT/interval-vest/vest/zip_regions.vest" -o "$ROOT/interval-vest/src/gen/zip_regions.rs" > /dev/null
echo "generated interval-vest zip_regions"
