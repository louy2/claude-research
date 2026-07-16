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

### 3. Ignite source: non-portable CLI assumptions

The CLI (`IgniteCLI`) makes a series of Unix/macOS assumptions. All of them are
fixed in [`ignite-portability.patch`](ignite-portability.patch); see the
[audit table](#portability-audit--patching-the-cli) below.

---

## Build

```bash
git clone https://github.com/twostraws/Ignite
cd Ignite
git apply ../ignite-portability.patch
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

## Portability audit — patching the CLI

Beyond *compiling*, the CLI assumed a Unix/macOS runtime in several places.
Every item below is fixed in [`ignite-portability.patch`](ignite-portability.patch):

| # | Location | Assumption | Fix |
|---|----------|------------|-----|
| 1 | `Process-Execute.swift` | Hardcodes `/bin/bash -c` for **every** command — nothing runs on native Windows | Use `ComSpec`/`cmd.exe /c` on Windows, `/bin/bash -c` elsewhere |
| 2 | `NewCommand` | `rm -rf name/.git` via the shell | `FileManager.removeItem` — no shell, works everywhere |
| 3 | `NewCommand` | Success only if stderr lacks the word `"fatal"` — a missing git ("`'git' is not recognized`") reads as **success** | Verify the clone produced `name/Package.swift` |
| 4 | `NewCommand` | Message says `open Package.swift` / "build with Xcode → My Mac" | Platform-neutral guidance |
| 5 | `RunCommand` | `open <url>` to launch the browser | `open` (macOS) / `start` (Windows) / `xdg-open` (Linux) |
| 6 | `RunCommand` | `toolDir` via `lastIndex(of: "/")` — misses Windows `\` → server script "missing" | URL path APIs (`deletingLastPathComponent`) |
| 7 | `RunCommand` | `lsof -t -i tcp:port` for the port check — no `lsof` on Windows | Portable socket-bind probe (POSIX + WinSock via `WinSDK`) |
| 8 | `RunCommand` | `python3` | `python` on Windows, `python3` elsewhere |
| 9 | `RunCommand` | loopback name `"lo0"` | also skip Linux's `"lo"` |
| — | `RunCommand` (prior) | POSIX `getifaddrs`/`sockaddr_in` compiled unconditionally | `#if os(Windows) return nil` (helper only reached from the Apple-only QR path) |

The socket-bind port check is the meatiest one — it replaces the `lsof`
shell-out with a real `bind()` probe, using `WinSDK` (`WSAStartup`/`socket`/
`bind`/`closesocket`) on Windows and Glibc/Darwin sockets elsewhere. That's why
the patched `IgniteCLI.exe` now imports `WS2_32.dll`.

## Smoke test (WineHQ 10.0 `wine64`, runtime DLLs alongside)

| Command | Before patches | After patches |
|---|---|---|
| `--help` / `--version` | ✅ works | ✅ works (`0.6.9`) |
| `run` (with a `Build/` dir) | n/a | ✅ runs the **WinSock** port probe, resolves the tool dir to `Z:\…`, exits cleanly at the "server script missing" check — no crash |
| `new <name>` | ⚠️ `ud2` trap in Windows `Foundation` on a Swift-Concurrency worker (in the `Task { process.run() }` + `Pipe.readDataToEndOfFile()` path), *after* writing files | ✅ no crash; honestly reports `❌ Failed … Is git installed and on your PATH?` when git is absent from the prefix |

Native Linux is unaffected — `ignite new` still clones, strips `.git` (now via
`FileManager`), and scaffolds a full site.

> Why the old `new` "worked" under Wine and the new one reports failure: the old
> code shelled to `/bin/bash`, which Wine bridges to the **host** shell (so the
> host's `git` ran). The patched code correctly uses `cmd.exe`, so `git` must be
> a real Windows program — absent from the bare Wine prefix, hence the honest
> error. On a real Windows box with Git + Swift installed, the patched paths are
> the correct ones. (`build`/`run` still need a Windows Swift toolchain and
> `python`, which aren't in the prefix.)

## Takeaways

- The hand-built Linux→Windows Swift SDK scales from a toy CLI to a real
  package with a C dependency and four SwiftPM dependencies.
- Two cross-compilation gotchas worth remembering: `#if os(Windows)` in a
  *manifest* is host-evaluated, and the prebuilt Windows `Foundation` pins C
  math to the `corecrt` module (so SDK header shims must preserve that).
- "Compiles and links" ≠ "behaves natively". Making it behave meant replacing
  every `/bin/bash`, `rm`, `open`, `lsof`, `python3`, and `/`-separator
  assumption with a platform-aware equivalent — the bulk of a real Windows port.
