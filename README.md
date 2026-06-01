# wcdemo — cross-compiling a non-trivial Swift CLI to a Windows `.exe` from Linux

This repo is the record of an experiment: **take a non-trivial Swift command-line
tool and produce a native Windows x86-64 `.exe` by cross-compiling on a Linux
(Ubuntu 24.04) host — no Windows machine, no Docker, no CI.**

**Result: it works.** `swift build --swift-sdk x86_64-unknown-windows-msvc`
produces a genuine PE32+ console executable:

```
$ file .build/x86_64-unknown-windows-msvc/release/wcdemo.exe
wcdemo.exe: PE32+ executable (console) x86-64, for MS Windows, 29 sections

$ llvm-objdump -p wcdemo.exe | grep 'DLL Name'
  swiftCore.dll  Foundation.dll  FoundationEssentials.dll  swiftCRT.dll
  swiftWinSDK.dll  swift_Concurrency.dll  KERNEL32.dll  VCRUNTIME140.dll
  api-ms-win-crt-runtime-l1-1-0.dll  api-ms-win-crt-stdio-l1-1-0.dll  ...
```

Everything was done with Swift **6.3.2**.

---

## The CLI

`wcdemo` is deliberately more than a hello-world. It is a multi-module SwiftPM
package that exercises an external dependency, Foundation, and conditional
compilation — the things that actually break under cross-compilation:

- **`DemoCore`** library target — pure logic: a 64-bit FNV-1a hasher, a
  Unicode-aware word-frequency counter, overflow-checked Fibonacci, and a
  compile-time `Platform` probe.
- **`wcdemo`** executable target — depends on `DemoCore` and
  [`swift-argument-parser`](https://github.com/apple/swift-argument-parser),
  with four subcommands:
  - `wordfreq <files…>` — word-frequency report (Foundation `String`/IO)
  - `checksum <files…>` — FNV-1a digest of each file (Foundation `Data`)
  - `fib <n>` — nth Fibonacci number
  - `info` — platform/runtime info; reports **`Windows`** when built for Windows
    even though it was compiled on Linux.
- **`DemoCoreTests`** — XCTest unit tests (run natively).

```bash
# native (Linux/macOS):
swift build -c release && swift test
.build/release/wcdemo info
```

---

## How the cross-compilation works

There is **no official prebuilt "Windows Swift SDK"** you can `swift sdk install`,
and `swift-sdk-generator` only targets Linux/FreeBSD. So the Windows SDK is
assembled by hand from three ingredients:

| # | Ingredient | Where it comes from |
|---|------------|---------------------|
| 1 | **`swiftc`** that emits Windows COFF | the normal Linux Swift toolchain (LLVM backend + `-target x86_64-unknown-windows-msvc`) |
| 2 | **Swift Windows runtime/SDK** (`.swiftmodule` + import libs) | extracted out of the official `swift-…-windows10.exe` installer |
| 3 | **MSVC CRT + Windows SDK** (headers + import libs) | downloaded from Microsoft with [`xwin`](https://github.com/Jake-Shadle/xwin) |

These are wired together with a **Swift SDK artifact bundle**
(`swift-sdk.json` + `toolset.json`), and the link step is driven by `clang` +
`lld-link` (both bundled in the toolchain).

The whole recipe — including extracting the WiX *burn* installer — is automated:

```bash
# prerequisites: curl msitools cabextract p7zip-full python3, and `xwin` on PATH
./scripts/cross-compile-windows.sh
```

---

## The interesting part: what broke, and the fixes

A clean cross-build does **not** happen out of the box. The obstacles, in order:

1. **No Windows zip — only a 1.76 GB WiX *burn* `.exe` installer.**
   It's a PE wrapping a UX cabinet + an attached container of MSIs and payload
   cabs. Carving the attached container (`MSCF` magic) and mapping its members
   via the burn manifest lets `cabextract` + `msiextract` reconstruct the
   `Windows.sdk` tree on Linux. (`scripts/_burn_extract.py`)

2. **`msiextract` stops at the first missing media.** `windows.msi` references
   `windows.cab` (media 1), `sdk.windows.arm64.cab` (2) and
   `sdk.windows.x64.cab` (3) in sequence — so media 1–3 must all be present to
   reach the x86_64 files.

3. **Clang module build failure: `could not build module 'SwiftOverlayShims'`**
   (`_sopen_s` / `_SH_DENYNO` / `errno_t` undeclared). Root cause: `xwin` ships
   the **legacy 10240-era UCRT headers**, but Swift's `ucrt.modulemap` (written
   for a newer split-header SDK) references `corecrt_math.h`, `stdalign.h` and
   `stdnoreturn.h`, which don't exist in that header set. Fix:
   - `stdalign.h` / `stdnoreturn.h` → copy from clang's builtin headers;
   - `corecrt_math.h` → empty placeholder (the math decls already live in
     `<math.h>`; an `#include <math.h>` here instead creates a
     `corecrt ↔ ucrt` **module cycle**).

4. **Modern MSVC CRT (14.44) headers vs legacy UCRT.** `xwin` pairs a current
   VC runtime with the old UCRT, so headers like `threads.h` reference UCRT
   warning macros (`_UCRT_DISABLE_CLANG_WARNINGS`, `_UCRT_RESTORE_CLANG_WARNINGS`,
   `_UCRT_DISABLED_WARNINGS`) that the old `corecrt.h` never defined. Fix: define
   them (clang-aware, as `#pragma clang diagnostic push/pop`) in `corecrt.h`.

5. **Case-sensitivity.** Autolink directives ask for `Imm32.lib`, `LZExpand.h`,
   `OCIdl.h`; `xwin` only created other casings. Fix: a few exact-case symlinks.
   (`xwin` does handle most case variants automatically.)

6. **`librarySearchPaths` not forwarded to the linker.** In this Swift version
   the `swift-sdk.json` `librarySearchPaths` did **not** reach `lld-link`
   (the `includeSearchPaths` were emitted as `-L` instead). Fix: pass the
   CRT/UCRT/UM lib dirs through the toolset's `swiftCompiler` `-L` options — the
   driver translates `-L` → `/libpath:` for `lld-link`.

After all six, the link succeeds and emits `wcdemo.exe`.

---

## Running it on Windows

The build links against the Swift/Foundation/VC runtime DLLs. To run on a real
Windows machine, drop the executable next to those DLLs (taken from the `rtl`
runtime inside the same Windows installer — `swiftCore.dll`, `Foundation*.dll`,
`swiftCRT.dll`, `swift_Concurrency.dll`, `vcruntime140*.dll`, `msvcp140*.dll`,
`dispatch.dll`, `BlocksRuntime.dll`, …), or install the Swift runtime
redistributable. Then:

```
> wcdemo.exe info
> wcdemo.exe fib 90
> wcdemo.exe wordfreq sample.txt
```

> **Note:** the binary could not be executed *in this Linux sandbox*: Wine 9.0
> aborts at startup (`free(): invalid pointer`) — a Wine/glibc-in-container bug
> that fires before the program is even loaded, unrelated to the binary itself.
> Verification here is therefore via `file` + import-table inspection; running
> needs a real Windows host (or a working Wine).

---

## Layout

```
Package.swift
Sources/DemoCore/        # Hashing, WordCounter, Fibonacci, Platform
Sources/wcdemo/          # ArgumentParser front-end (main.swift)
Tests/DemoCoreTests/
scripts/
  cross-compile-windows.sh   # end-to-end: toolchain → SDK → build
  _burn_extract.py           # WiX burn installer extractor
```
