# imhex-vest: ImHex pattern language → tree-sitter → vest_lib

A Rust toolchain for the [ImHex pattern language](https://docs.werwolv.net/pattern-language)
(`.hexpat`):

1. **`tree-sitter-hexpat`** – a tree-sitter grammar for the language, with
   Rust bindings. It parses 286 of the 287 files in the
   [ImHex-Patterns](https://github.com/WerWolv/ImHex-Patterns) corpus without
   errors (the last one only parses after macro expansion).
2. **`hexpat`** – a preprocessor, a CST→AST lowering, and an **interpreter**
   whose byte-level decoding is done by [`vest_lib`](https://github.com/secure-foundations/vest)
   combinators (`U8`, `U16Le`, `U32Be`, `Pair`, `RepeatN`, `Varied`, …). It
   evaluates real-world patterns against binary files and produces an
   ImHex-style pattern tree. On the corpus it evaluates 157 of the 178
   patterns that ship with a test fixture (88%).
3. **`hexpat emit-vest`** – a translator from the *declarative* subset of
   hexpat to the **Vest DSL**. The Vest compiler then generates Rust parsers
   and serializers built on `vest_lib` that carry Verus proofs of memory
   safety, panic freedom and parse/serialize round-tripping.
4. **`vest-formats`** – verified parsers for ten popular formats (BMP, GIF,
   PNG, WAV, ELF, tar, ZIP, ICO, QOI, gzip) generated through that pipeline
   from the CC0 patterns in [`formats/`](formats/), with tests showing that
   the verified parser and the interpreter agree on every checked field and
   that the verified serializer reproduces the input bytes.

```
formats/*.hexpat ──tree-sitter──▶ AST ──interpreter (vest_lib primitives)──▶ pattern tree
                                   │
                                   └──emit-vest──▶ *.vest ──vest compiler──▶ verified Rust (vest_lib)
```

## Layout

| Path | What |
|---|---|
| `tree-sitter-hexpat/` | grammar (`grammar.js`), generated `src/parser.c`, Rust bindings, `test/corpus` |
| `hexpat/src/preprocess.rs` | `#define` expansion, `#include`, `#ifdef`, `#pragma` |
| `hexpat/src/lower.rs` | tree-sitter CST → `ast.rs` (match ranges, compound assignment, literals) |
| `hexpat/src/reader.rs` | every integer/byte read goes through a `vest_lib` combinator |
| `hexpat/src/interp/` | evaluator: types, scopes, structs, unions, arrays, bitfields, pointers, sections, functions, builtins, format strings, dumps |
| `hexpat/src/emit.rs` | hexpat → Vest DSL |
| `hexpat/src/bin/hexpat.rs` | CLI: `parse`, `run`, `emit-vest` |
| `formats/` | CC0 patterns for popular formats (declarative subset) |
| `vest-formats/` | generated `.vest` + generated verified Rust + end-to-end tests |
| `scripts/` | `fetch-corpus.sh`, `regen-formats.sh` |

## Usage

```console
$ cargo build --release
$ target/release/hexpat run formats/png.hexpat image.png
png: Png @ 0x0 [5890] = Png { ... }
  signature: Signature @ 0x0 [8] = Signature { ... }
    highBit: u8 @ 0x0 [1] = 137 (0x89)
    png: char[3] @ 0x1 [3] = "PNG"
  ...
$ target/release/hexpat run --json pattern.hexpat file.bin              # JSON, identical to ImHex's export
$ target/release/hexpat format -p pattern.hexpat -i file.bin -f json -m # same, plcli-style, with metadata
$ target/release/hexpat run -I vendor/ImHex-Patterns/includes bmp.hexpat x.bmp   # with the real std library
$ target/release/hexpat emit-vest formats/elf.hexpat                    # print the Vest DSL
$ target/release/hexpat parse pattern.hexpat                            # syntax check + summary
```

`hexpat run` accepts `-I dir` for `import` resolution, `--in name=value`
for `in` variables, `--hidden`, `--max-entries N`, and `HEXPAT_TRACE=1`
prints every statement and pattern creation.

### JSON export

`hexpat run --json` and `hexpat format -f json` write the pattern tree in
the same JSON that the reference implementation's `plcli format -f json`
(ImHex's `FormatterJson`) produces: a nested object keyed by variable name,
arrays as lists, `char[]` arrays and enums and characters as strings,
integers/floats/booleans as literals unless a `[[format]]` function is
attached (then its result as a string), sealed structs as their formatted
value, pointers as an object holding `*(name)`, and padding or `[[hidden]]`
patterns omitted. `-m` adds the reference's metadata fields (`__type`,
`__address`, `__size`, `__color`, `__endian`, `__comment`) to every
object. `hexpat/tests/json.rs` checks the output against the reference
project's own expected export byte for byte. Automatic colors follow the
reference palette in creation order, which can differ from ImHex when
patterns are created in a different order. The earlier structured dump
(offsets and sizes on every node) is still available as `--tree-json`.

Without an include directory the interpreter provides native
implementations of `std::mem`, `std::core`, `std::string`, `std::math`,
`std::time`, `std::hash::crc32`, `std::print/format/assert/error/warning`
and `type::Magic`; with `-I <ImHex-Patterns>/includes` it loads and
executes the real `.pat` standard library, where only the `builtin::std::*`
functions are native.

## The interpreter

The evaluator follows the reference implementation's semantics: placements
(`Type name @ addr`), struct bodies executed statement by statement with the
cursor `$`, `parent`/`this`, `sizeof`/`addressof`, `if`/`match`/`try`,
`while`-sized, unsized and fixed arrays, `break`/`continue` propagation out
of nested structs into arrays, unions, bitfields (least- and
most-significant-first, nested, typed, `[[no_unique_address]]`), pointers
(`T *p : u32`, `[[pointer_base]]`), templates (`struct S<T, auto N>`,
`using A<T> = …`), namespaces and import aliases, functions with `ref`,
`auto`, defaults and parameter packs, struct-scoped locals reachable as
members, heap sections for local struct variables (`std::time::TimeConverter`
tricks work), `[[transform]]` (LEB128-style integers used as sizes),
`[[format]]`, `[[name]]`, `[[comment]]`, `[[hidden]]`, `[[inline]]`,
`[[fixed_size]]`, and an automatic `fn main()`.

Every byte decode is a `vest_lib` primitive: see
[`reader.rs`](hexpat/src/reader.rs). Widths that `vest_lib` has no single
combinator for (48/96/128 bits) are composed with `Pair`.

### Corpus results

Fetch the corpus (`scripts/fetch-corpus.sh`) and run
`IMHEX_PATTERNS=vendor/ImHex-Patterns cargo test -p hexpat --test corpus -- --nocapture`.

| Stage | Result |
|---|---|
| tree-sitter parse, no preprocessing | 286 / 287 files without `ERROR` nodes |
| preprocess + lower to AST | all top-level patterns and includes |
| interpret against the bundled fixture | 157 / 178 |

The remaining fixture failures are mostly patterns that read past the end
of a fixture that the reference runtime tolerates differently, patterns
whose fixture directory holds several files for different branches,
`std::string` edge cases, and a few unimplemented corners (`.parent` on
arbitrary patterns, some section semantics).

## hexpat → Vest

The Vest DSL describes formats declaratively, so only the format-shaped
subset of a pattern can be translated:

| hexpat | Vest |
|---|---|
| `struct S { u32 a; T b; }` | `s = { a: u32, b: t, }` |
| `u8 x[n]`, `char x[4]`, `T x[count]`, `T x[a.b]` | `[u8; @n]`, `[u8; 4]`, `[t; @count]`, `[t; @a.b]` |
| `enum E : u8 { A = 1, … }` | `e = enum { A = 1u8, …, ... }` (open) |
| `if (f == C) {…} else if (f == D) {…} else {…}` | `choose(@f) { C => …, D => …, _ => … }` |
| `match (f) { (C): …; (_): …; }` | `choose(@f) { … }` |
| `if (name == "IHDR") …` on `char name[4]` | byte-string `choose` |
| `type::Magic<"GIF"> m;` | `const m: [u8; 3] = [0x47, 0x49, 0x46]` |
| `padding[n]` | `_padN: [u8; n]` |
| `bitfield B { a : 3; b : 5; }` | `b = bits { b: u5, a: u3, }` (MSB-first order) |
| trailing `T x[while(!std::mem::eof())]` | `Vec<t>` |
| `struct A : B { … }` | B's fields inlined |

Unsupported constructs (placements inside structs, pointers, `$`
arithmetic, unsized strings, unions, locals that feed sizes, functions)
are reported as notes and the affected struct is left out rather than
emitted incorrectly. Signed integers and floats are emitted as their
unsigned width with a note, because the Vest DSL does not expose them yet.

`scripts/regen-formats.sh` (needs `cargo install vest`) regenerates
`vest-formats/vest/*.vest` and `vest-formats/src/gen/*.rs`. The generated
Rust builds with plain cargo; `cargo verus verify` checks the proofs when
Verus is installed.

## Tests

```console
$ cargo test                       # grammar, preprocessor, lowering, interpreter, emitter, end-to-end formats
$ cd tree-sitter-hexpat && tree-sitter test
```

`vest-formats/tests/formats.rs` builds a synthetic file for each format,
parses it with the verified parser, evaluates the same bytes with the
interpreter, compares the fields, and serializes the parsed value back to
the original bytes.

## Licensing

Everything here is CC0-1.0. The ImHex-Patterns corpus (GPL-2.0) is used only
as an external test input and is fetched on demand, never vendored. The
patterns in `formats/` were written for this repository.
