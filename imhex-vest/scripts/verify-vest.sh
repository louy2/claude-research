#!/bin/bash
# Checks the Verus proofs of vest-formats and interval-vest (Verus from
# scripts/setup-verus.sh on PATH) and prints each crate's result line.
set -uo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
status=0
for crate in vest-formats interval-vest; do
  # cargo does not rerun Verus on an unchanged crate; drop its cached result.
  find "$ROOT/target/verus-partial" -maxdepth 4 -name "*$(echo $crate | tr - _)*" -exec rm -rf {} + 2>/dev/null
  out=$(cd "$ROOT/$crate" && cargo verus focus 2>&1)
  rc=$?
  line=$(echo "$out" | grep "verification results" | tail -1)
  echo "$crate: ${line:-no result} (exit $rc)"
  [ $rc -eq 0 ] || { echo "$out" | grep -E -A6 "^error" | head -40; status=1; }
done
exit $status
