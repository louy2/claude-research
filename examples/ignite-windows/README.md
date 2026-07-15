# Cross-compiling Ignite to Windows (a real-world stress test)

A follow-up to the [`wcdemo`](../../README.md) experiment: can the same
Linux→Windows Swift SDK cross-compile a *real* third-party package —
[**Ignite**](https://github.com/twostraws/Ignite), Paul Hudson's static-site
generator — and does the result actually run?

**Short answer: yes to the build, mostly yes to the run.** The whole package
(the `Ignite` library **and** the `IgniteCLI` executable, plus all
dependencies) cross-compiles to a native Windows x86-64 `.exe`, and the CLI
runs under Wine — `--help`, `--version`, and the real work of `ignite new`
(cloning + scaffolding a site) all succeed. It then trips over a Wine +
Windows-Foundation subprocess limitation, described below.

Tested with Swift 6.3.2 against `Ignite` @ `ebc296e`, using the exact SDK setup
from [`scripts/cross-compile-windows.sh`](../../scripts/cross-compile-windows.sh).

---

## What it took

Ignite is genuinely non-trivial: it pulls in `swift-markdown` (which wraps the
**cmark-gfm C library**), `SwiftSoup`, `swift-collections`, and
`swift-argument-parser`. Three extra things were needed beyond the base SDK:

### 1. `swift-cmark`: `dllimport cannot be applied to a definition`

cmark's export header defaults `CMARK_GFM_EXPORT` to `__declspec(dllimport)`
unless `CMARK_GFM_STATIC_DEFINE` is set. `swift-cmark`'s `Package.swift` *does*
set it — but guarded by `#if os(Windows)`, which is evaluated **when the
manifest is compiled on the build host (Linux)**, not for the target. So the
Windows branch never fires during cross-compilation. Fix: define it on the
command line for the whole build:

```bash
swift build --swift-sdk x86_64-unknown-windows-msvc -Xcc -DCMARK_GFM_STATIC_DEFINE
```

> General lesson: `#if os(Windows)` in a **`Package.swift`** reflects the
> manifest host, not the cross-compilation target. Package-level platform
> conditionals can silently do the wrong thing when cross-compiling.

### 2. SDK fix: `'pow' was expected to be in 'corecrt'`

The prebuilt Windows `Foundation.swiftmodule` records that C math functions
(`pow`, …) belong to the **`corecrt`** Clang module. In the [wcdemo
writeup](../../README.md) `corecrt_math.h` was stubbed *empty* (nothing
referenced it there). Ignite's heavier Foundation use exposed that shortcut:
with an empty header, `pow` landed in a different module and deserializing
`Foundation` failed with *"reference to top-level declaration 'pow' broken by a
context change"*.

Proper fix — put the **real** math declarations in `corecrt_math.h` so `pow`
lives in `corecrt`, and make `math.h` forward to it (this legacy UCRT ships the
declarations directly in `math.h`; newer SDKs split them out, which is what
Foundation was built against):

```sh
cp  ucrt/math.h  ucrt/corecrt_math.h      # real declarations -> corecrt module
printf '#pragma once\n#include <corecrt_math.h>\n' > ucrt/math.h
```

No include cycle results, because `corecrt_math.h` only pulls `corecrt.h`
(same module). This supersedes the empty-stub approach and works for both
projects.

### 3. Ignite source: POSIX `getifaddrs`

`IgniteCLI/RunCommand.getLocalIPAddress()` uses `getifaddrs`/`getnameinfo`/
`sockaddr_in`, which don't exist on Windows. The function is only ever *called*
from the `#if canImport(CoreImage)` (Apple-only) QR path, but it's *compiled*
unconditionally. One-line guard to return `nil` on Windows —
see [`ignite-runcommand-windows.patch`](ignite-runcommand-windows.patch).

---

## Build

```bash
git clone https://github.com/twostraws/Ignite
cd Ignite
git apply ../ignite-runcommand-windows.patch
swift build -c release --swift-sdk x86_64-unknown-windows-msvc -Xcc -DCMARK_GFM_STATIC_DEFINE
# -> .build/x86_64-unknown-windows-msvc/release/IgniteCLI.exe  (PE32+ x86-64)
```

```
$ file IgniteCLI.exe
IgniteCLI.exe: PE32+ executable (console) x86-64, for MS Windows, 29 sections
$ llvm-objdump -p IgniteCLI.exe | grep 'DLL Name'
  swiftCore.dll  Foundation.dll  FoundationEssentials.dll  swiftCRT.dll
  swiftWinSDK.dll  swift_Concurrency.dll  swift_StringProcessing.dll
  KERNEL32.dll  VCRUNTIME140.dll  api-ms-win-crt-*.dll
```

The `Ignite` **library** target compiles for Windows too (it's built as part of
the package graph), which is the real proof — the HTML DSL, the markdown/cmark
pipeline, SwiftSoup and Collections all cross-compile.

---

## Smoke test (WineHQ 10.0 `wine64`, runtime DLLs alongside)

| Command | Result |
|---|---|
| `IgniteCLI.exe --help` | ✅ full ArgumentParser help (`new` / `build` / `run`) |
| `IgniteCLI.exe --version` | ✅ `0.6.9` |
| `IgniteCLI.exe new TestSite` | ⚠️ **clones the starter template and writes the complete site scaffold** (`Package.swift`, `Sources/Site.swift`, `Content/`, `Assets/`, …), then traps |

The `new` crash is instructive rather than a build defect:

- `IgniteCLI/Process-Execute.swift` **hardcodes** `process.executableURL =
  /bin/bash` and runs commands (`git clone …`, `rm -rf …/.git`) via `bash -c`.
  Under Wine, `/bin/bash` resolves to the host shell (`Z:\bin\bash`) — which is
  *why the git clone actually succeeds*. On a real Windows box there is no
  `/bin/bash`, so `ignite new` wouldn't run as written regardless of our build.
- The trap is a `ud2` inside Windows `Foundation` on a Swift-Concurrency worker
  thread, in the `Task { try process.run() }` + `Pipe.readDataToEndOfFile()`
  path — i.e. `Foundation.Process`/`FileHandle` async subprocess handling under
  Wine, *after* all files are written.

So the executable itself is sound and substantially functional; the failure is
a **Wine + Windows-Foundation runtime limitation** plus Ignite's CLI assuming a
Unix shell — not a cross-compilation problem. `ignite build`/`run` can't be
fully exercised here anyway, since they invoke a Windows Swift toolchain that
isn't installed in the Wine prefix.

## Takeaways

- The hand-built Linux→Windows Swift SDK scales from a toy CLI to a real
  package with a C dependency and four SwiftPM dependencies.
- Two cross-compilation gotchas worth remembering: `#if os(Windows)` in a
  *manifest* is host-evaluated, and the prebuilt Windows `Foundation` pins C
  math to the `corecrt` module (so SDK header shims must preserve that).
- "Compiles and links" ≠ "behaves natively": a tool that shells out to
  `/bin/bash` and `rm` is portable only as far as its runtime assumptions.
