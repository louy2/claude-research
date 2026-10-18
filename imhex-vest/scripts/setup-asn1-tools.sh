#!/bin/bash
# Builds the tools used by scripts/regen-asn1.sh and scripts/verify-asn1.sh
# from source into $TOOLS (default: ~/asn1-tools):
#
#   - ASN1SCC at the commit the patch was written against, with
#     asn1-formats/asn1scc-scala-fixes.patch applied (.NET 10 SDK needed);
#   - Stainless 0.9.8.9 (sbt and a JDK 17 needed; the generated Scala uses
#     library functions that appeared in 0.9.8.8);
#   - z3, cvc5 and cvc4 are expected on PATH (Ubuntu: apt install z3 cvc5 cvc4).
#
# It prints the environment variables to export afterwards.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TOOLS="${TOOLS:-$HOME/asn1-tools}"
ASN1SCC_COMMIT=5633b5ba678f8a21eb065c1a79247fbc5d20710c
STAINLESS_TAG=v0.9.8.9
mkdir -p "$TOOLS"

if [ ! -d "$TOOLS/asn1scc" ]; then
  git clone https://github.com/maxime-esa/asn1scc "$TOOLS/asn1scc"
fi
(
  cd "$TOOLS/asn1scc"
  git checkout -q "$ASN1SCC_COMMIT"
  git apply --check "$ROOT/asn1-formats/asn1scc-scala-fixes.patch" 2>/dev/null && git apply "$ROOT/asn1-formats/asn1scc-scala-fixes.patch"
  dotnet build Antlr/
  dotnet build parseStg2/
  dotnet build asn1scc/asn1scc.fsproj
)

if [ ! -d "$TOOLS/stainless" ]; then
  git clone https://github.com/epfl-lara/stainless "$TOOLS/stainless"
fi
(
  cd "$TOOLS/stainless"
  git checkout -q "$STAINLESS_TAG"
  git submodule update --init inox
  sbt stainless-dotty-standalone/assembly
)
jar=$(ls "$TOOLS"/stainless/frontends/stainless-dotty-standalone/target/scala-*/stainless-dotty-standalone-0.9.8.9.jar)
cat > "$TOOLS/stainless.sh" <<EOF
#!/bin/sh
exec java -Xss1G -Xmx10g -jar $jar "\$@"
EOF
chmod +x "$TOOLS/stainless.sh"

cat <<EOF
export ASN1SCC=$TOOLS/asn1scc/asn1scc/bin/Debug/net10.0/asn1scc
export STAINLESS=$TOOLS/stainless.sh
# Scala 3 compiler for scripts/regen-asn1.sh (scalac 3.3.x; the jars of
# scala3-library_3 and scala-library 2.13 go in SCALA_LIB):
export SCALAC=scalac SCALA_LIB=/path/to/scala3/lib
EOF
