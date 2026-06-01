#!/usr/bin/env bash
#
# cross-compile-windows.sh
#
# Cross-compile this Swift package into a native Windows x86_64 .exe FROM Linux.
#
# There is no official prebuilt "Windows Swift SDK" artifact bundle, and
# swift-sdk-generator only targets Linux/FreeBSD. So this script assembles a
# working Windows Swift SDK by hand out of three ingredients:
#
#   1. The Linux-hosted Swift toolchain  (the LLVM-based `swiftc` can emit
#      Windows COFF objects given `-target x86_64-unknown-windows-msvc`).
#   2. The Swift *Windows* runtime/SDK    (`.swiftmodule` + import libs built for
#      Windows), extracted out of the official Windows installer .exe.
#   3. The MSVC CRT + Windows SDK headers/import-libs, fetched with `xwin`.
#
# It then writes a Swift SDK artifact bundle (swift-sdk.json + toolset.json),
# installs it, applies a handful of compatibility fixups (see FIXUPS below) and
# runs `swift build --swift-sdk x86_64-unknown-windows-msvc`.
#
# Tested on Ubuntu 24.04 / x86_64 with Swift 6.3.2. Run from the repo root.
#
# Required tools: curl, msitools (msiextract/msiinfo), cabextract, 7z,
#                 p7zip-full, python3, and the `xwin` binary on PATH.
set -euo pipefail

SWIFT_VERSION="${SWIFT_VERSION:-6.3.2}"
WORK="${WORK:-$HOME/.swift-windows-xc}"        # scratch dir for SDK ingredients
UBUNTU="ubuntu24.04"; UBUNTU_DIR="ubuntu2404"
TC="$WORK/toolchain"                             # Linux Swift toolchain
WINROOT="$WORK/winsdk"                           # extracted Swift Windows SDK
XWIN="$WORK/xwin"                                # MSVC CRT + Windows SDK (xwin)
BUNDLE="$WORK/windows.artifactbundle"
SDK_ID="swift-windows-${SWIFT_VERSION}"
ARCH_TRIPLE="x86_64-unknown-windows-msvc"

mkdir -p "$WORK"

say() { printf '\n\033[1;36m==> %s\033[0m\n' "$*"; }

# ---------------------------------------------------------------------------
say "1/6  Linux Swift $SWIFT_VERSION toolchain"
if [[ ! -x "$TC/usr/bin/swiftc" ]]; then
  url="https://download.swift.org/swift-${SWIFT_VERSION}-release/${UBUNTU_DIR}/swift-${SWIFT_VERSION}-RELEASE/swift-${SWIFT_VERSION}-RELEASE-${UBUNTU}.tar.gz"
  curl -fSL -o "$WORK/swift-linux.tar.gz" "$url"
  mkdir -p "$TC"; tar xzf "$WORK/swift-linux.tar.gz" -C "$TC" --strip-components=1
fi
SWIFTC="$TC/usr/bin/swiftc"
"$SWIFTC" --version

# ---------------------------------------------------------------------------
say "2/6  Swift Windows SDK (extracted from the Windows installer .exe)"
WIN_SDK="$WINROOT/LocalApp/Programs/Swift/Platforms/${SWIFT_VERSION}/Windows.platform/Developer/SDKs/Windows.sdk"
if [[ ! -d "$WIN_SDK" ]]; then
  exe="$WORK/swift-windows.exe"
  curl -fSL -o "$exe" \
    "https://download.swift.org/swift-${SWIFT_VERSION}-release/windows10/swift-${SWIFT_VERSION}-RELEASE/swift-${SWIFT_VERSION}-RELEASE-windows10.exe"
  # The installer is a WiX "burn" bundle: a PE whose attached container holds
  # the MSI packages + payload cabs. Carve the attached container (first MSCF
  # after the UX cab) and pull out the windows.msi (Platform SDK) plus the
  # x86_64 SDK cabs it references.
  off=$(grep -abo MSCF "$exe" | sed -n 2p | cut -d: -f1)
  dd if="$exe" of="$WORK/attached.cab" bs=1M skip="$off" iflag=skip_bytes status=none
  # Map cab members (a0,a1,... in container order) to real filenames via the
  # burn manifest, then extract the runtime + Platform SDK + arch SDK cabs.
  7z x -y -o"$WORK/ux" "$exe" >/dev/null
  mkdir -p "$WORK/msi"
  python3 "$(dirname "$0")/_burn_extract.py" "$WORK/ux/0" "$WORK/attached.cab" "$WORK/msi"
  ( cd "$WINROOT" 2>/dev/null || { mkdir -p "$WINROOT"; cd "$WINROOT"; }
    msiextract -C "$WINROOT" "$WORK/msi/windows.msi" >/dev/null )
fi
ls "$WIN_SDK/usr/lib/swift/windows/x86_64/swiftCore.lib"

# ---------------------------------------------------------------------------
say "3/6  MSVC CRT + Windows SDK via xwin"
if [[ ! -d "$XWIN/crt" ]]; then
  xwin --accept-license --arch x86_64 splat --output "$XWIN"
fi

# ---------------------------------------------------------------------------
say "4/6  FIXUPS — place modulemaps + bridge legacy-UCRT / modern-VC gaps"
SHARE="$WIN_SDK/usr/share"
cp "$SHARE/vcruntime.modulemap"     "$XWIN/crt/include/module.modulemap"
cp "$SHARE/ucrt.modulemap"          "$XWIN/sdk/include/ucrt/module.modulemap"
cp "$SHARE/winsdk_um.modulemap"     "$XWIN/sdk/include/um/module.modulemap"
cp "$SHARE/winsdk_shared.modulemap" "$XWIN/sdk/include/shared/module.modulemap"

# xwin ships the legacy 10240 UCRT headers; Swift's modulemap (newer SDK)
# references 3 headers absent from that set:
CB="$(ls -d "$TC"/usr/lib/clang/*/include | head -1)"
cp "$CB/stdalign.h"    "$XWIN/sdk/include/ucrt/stdalign.h"      # clang builtin
cp "$CB/stdnoreturn.h" "$XWIN/sdk/include/ucrt/stdnoreturn.h"   # clang builtin
# corecrt_math.h: math decls live directly in <math.h> in this UCRT, and nothing
# but the modulemap references this file, so an empty placeholder satisfies the
# module without creating a corecrt<->ucrt include cycle.
printf '#pragma once\n' > "$XWIN/sdk/include/ucrt/corecrt_math.h"

# Exact-case symlinks for um headers/libs the autolink directives request but
# xwin only provided under other casings:
ln -sf lzexpand.h "$XWIN/sdk/include/um/LZExpand.h"
ln -sf ocidl.h    "$XWIN/sdk/include/um/OCIdl.h"
ln -sf imm32.lib  "$XWIN/sdk/lib/um/x86_64/Imm32.lib"

# Modern VC headers (14.x) reference UCRT warning macros not in the legacy
# corecrt.h. Define them (clang-aware) so those headers compile.
python3 - "$XWIN/sdk/include/ucrt/corecrt.h" <<'PY'
import sys
p=sys.argv[1]
s=open(p,encoding="utf-8",errors="surrogateescape").read()
if "_UCRT_DISABLE_CLANG_WARNINGS" not in s:
    block='''#include <vcruntime.h>

/* Swift cross-compile shim: bridge legacy UCRT <-> modern MSVC CRT headers. */
#ifndef _UCRT_DISABLED_WARNINGS
#define _UCRT_DISABLED_WARNINGS 4324 4514 4574 4710 4711 4793 4820 4995 4996 28719 28720 28726 28727
#endif
#ifndef _UCRT_DISABLE_CLANG_WARNINGS
#if defined(__clang__)
#define _UCRT_DISABLE_CLANG_WARNINGS _Pragma("clang diagnostic push") _Pragma("clang diagnostic ignored \\"-Wunknown-pragmas\\"")
#else
#define _UCRT_DISABLE_CLANG_WARNINGS
#endif
#endif
#ifndef _UCRT_RESTORE_CLANG_WARNINGS
#if defined(__clang__)
#define _UCRT_RESTORE_CLANG_WARNINGS _Pragma("clang diagnostic pop")
#else
#define _UCRT_RESTORE_CLANG_WARNINGS
#endif
#endif
'''
    open(p,"w",encoding="utf-8",errors="surrogateescape").write(s.replace("#include <vcruntime.h>",block,1))
    print("patched corecrt.h")
PY

# ---------------------------------------------------------------------------
say "5/6  Assemble + install the Swift SDK artifact bundle"
DEST="$BUNDLE/$SDK_ID/x86_64-windows"
mkdir -p "$DEST"
cat > "$BUNDLE/info.json" <<EOF
{ "schemaVersion": "1.0", "artifacts": { "$SDK_ID": {
  "type": "swiftSDK", "version": "$SWIFT_VERSION",
  "variants": [ { "path": "$SDK_ID/x86_64-windows",
                  "supportedTriples": ["x86_64-unknown-linux-gnu"] } ] } } }
EOF
# NOTE: librarySearchPaths in swift-sdk.json are not forwarded to the linker in
# this Swift version, so the CRT/UCRT/UM lib dirs are passed via the toolset's
# swiftCompiler -L options instead (the driver translates -L -> /libpath:).
cat > "$DEST/toolset.json" <<EOF
{ "schemaVersion": "1.0", "rootPath": "$TC/usr/bin",
  "swiftCompiler": { "extraCLIOptions": [
    "-use-ld=lld",
    "-L", "$WIN_SDK/usr/lib/swift/windows/x86_64",
    "-L", "$XWIN/crt/lib/x86_64",
    "-L", "$XWIN/sdk/lib/ucrt/x86_64",
    "-L", "$XWIN/sdk/lib/um/x86_64" ] },
  "cCompiler":   { "extraCLIOptions": ["-Wno-unused-command-line-argument"] },
  "cxxCompiler": { "extraCLIOptions": ["-Wno-unused-command-line-argument"] } }
EOF
cat > "$DEST/swift-sdk.json" <<EOF
{ "schemaVersion": "4.0", "targetTriples": { "$ARCH_TRIPLE": {
  "sdkRootPath": "$WIN_SDK",
  "swiftResourcesPath": "$WIN_SDK/usr/lib/swift",
  "swiftStaticResourcesPath": "$WIN_SDK/usr/lib/swift_static",
  "includeSearchPaths": [
    "$XWIN/crt/include", "$XWIN/sdk/include/ucrt",
    "$XWIN/sdk/include/um", "$XWIN/sdk/include/shared", "$XWIN/sdk/include/winrt" ],
  "librarySearchPaths": [
    "$WIN_SDK/usr/lib/swift/windows/x86_64",
    "$XWIN/crt/lib/x86_64", "$XWIN/sdk/lib/ucrt/x86_64", "$XWIN/sdk/lib/um/x86_64" ],
  "toolsetPaths": ["toolset.json"] } } }
EOF
"$TC/usr/bin/swift" sdk install "$BUNDLE" 2>/dev/null || true
"$TC/usr/bin/swift" sdk list

# ---------------------------------------------------------------------------
say "6/6  Cross-compile"
export PATH="$TC/usr/bin:$PATH"
swift build -c release --swift-sdk "$ARCH_TRIPLE"
out=".build/$ARCH_TRIPLE/release/wcdemo.exe"
file "$out"
say "Done -> $out"
echo "Bundle runtime DLLs from: $WINROOT (the 'rtl' runtime) to run on Windows."
