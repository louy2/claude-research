#!/bin/bash
# Installs the Verus release that matches the vstd version pinned in
# Cargo.toml, the z3 it expects, and the Rust toolchain it was built with,
# into $TOOLS (default: ~/verus), and prints the PATH to export. Linux x86-64.
#
# Afterwards, `cargo verus focus` in vest-formats/ or interval-vest/ checks
# that crate's proofs (scripts/verify-vest.sh runs both).
set -euo pipefail
TOOLS="${TOOLS:-$HOME/verus}"
VERUS_RELEASE=0.2026.09.06.8dea4a2   # vstd 0.0.0-2026-09-06-0133
Z3_VERSION=4.16.0                    # source/tools/get-z3.sh at that release
Z3_ASSET=z3-$Z3_VERSION-x64-glibc-2.39
mkdir -p "$TOOLS"
cd "$TOOLS"
if [ ! -x verus-x86-linux/verus ]; then
  curl -sSL -o verus.zip \
    "https://github.com/verus-lang/verus/releases/download/release%2F$VERUS_RELEASE/verus-$VERUS_RELEASE-x86-linux.zip"
  unzip -q verus.zip
  rm verus.zip
fi
if [ ! -x verus-x86-linux/z3 ]; then
  curl -sSL -o z3.zip "https://github.com/Z3Prover/z3/releases/download/z3-$Z3_VERSION/$Z3_ASSET.zip"
  unzip -q z3.zip
  cp "$Z3_ASSET/bin/z3" verus-x86-linux/z3
  rm -r z3.zip "$Z3_ASSET"
fi
toolchain=$(sed -n 's/.*"toolchain": "\([^ "]*\).*/\1/p' verus-x86-linux/version.json)
rustup toolchain install "$toolchain" --profile minimal
echo "export PATH=$TOOLS/verus-x86-linux:\$PATH"
