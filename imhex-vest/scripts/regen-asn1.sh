#!/bin/bash
# Regenerates asn1-formats/asn1/<name>/ (ASN.1, ACN, type map, Scala harness)
# from formats/*.hexpat, then, when the tools are available, runs ASN1SCC's
# Scala backend into asn1-formats/build/<name>/ and compiles the result with
# the harness.
#
#   ASN1SCC   asn1scc executable (patched, see asn1-formats/README.md)
#   SCALAC    Scala 3 compiler (default: scalac3)
#   SCALA_LIB directory holding scala3-library_3-*.jar and scala-library-2.13*.jar
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ASN1SCC="${ASN1SCC:-asn1scc}"
SCALAC="${SCALAC:-scalac3}"
SCALA_LIB="${SCALA_LIB:-/opt/scala3/lib}"
cargo build -q --manifest-path "$ROOT/Cargo.toml" -p hexpat
names=("$@")
if [ ${#names[@]} -eq 0 ]; then
  for pat in "$ROOT"/formats/*.hexpat; do names+=("$(basename "$pat" .hexpat)"); done
fi
for name in "${names[@]}"; do
  spec="$ROOT/asn1-formats/asn1/$name"
  mkdir -p "$spec"
  "$ROOT/target/debug/hexpat" emit-asn1 "$ROOT/formats/$name.hexpat" -o "$spec" 2> "$spec/notes.txt" || { cat "$spec/notes.txt"; exit 1; }
  if ! command -v "$ASN1SCC" > /dev/null; then
    echo "generated $name (asn1scc not found; skipping code generation)"
    continue
  fi
  build="$ROOT/asn1-formats/build/$name"
  rm -rf "$build"
  mkdir -p "$build/classes"
  "$ASN1SCC" -Scala -ACN -invertibility -o "$build/scala" "$spec/$name.asn" "$spec/$name.acn"
  cp "$spec/HexpatHarness.scala" "$build/harness.scala"
  lib="$build/scala/lib/stainless-library_3-0.9.8.7.jar"
  "$SCALAC" -classpath "$lib" -d "$build/classes" $(find "$build/scala/src" -name '*.scala') "$build/harness.scala"
  echo "$build/classes:$lib:$(ls "$SCALA_LIB"/scala3-library_3-*.jar):$(ls "$SCALA_LIB"/scala-library-2.13*.jar)" > "$build/classpath"
  echo "generated and compiled $name"
done
