# asn1-formats: hexpat → ASN.1/ACN → ASN1SCC → Stainless

`hexpat emit-asn1` translates the declarative subset of an ImHex pattern
into an ASN.1 module plus an ACN encoding spec. [ASN1SCC](https://github.com/maxime-esa/asn1scc)
generates codecs from the pair; its Scala backend annotates them for the
[Stainless](https://github.com/epfl-lara/stainless) verifier, as described in
*Formally Verifiable Generated ASN.1/ACN Encoders and Decoders: A Case Study*
(Bucev, Chassot, Felix, Schramka, Kunčak, [arXiv:2412.07235](https://arxiv.org/abs/2412.07235)).
This directory holds the translation of the ten patterns in [`../formats/`](../formats/),
the ASN1SCC fixes the translation needed, and the results of running and
verifying the generated code.

```
formats/x.hexpat ──emit-asn1──▶ asn1/x/x.asn + x.acn ──asn1scc -Scala -ACN -invertibility──▶ build/x/scala (Scala + Stainless annotations)
                                                                                          ├──scalac + HexpatHarness──▶ decode/re-encode files, compared with the interpreter
                                                                                          └──stainless──▶ results/x.txt
```

| Path | What |
|---|---|
| `asn1/<name>/<name>.asn`, `.acn` | the generated specs (checked against the emitter by `hexpat/tests/asn1.rs`) |
| `asn1/<name>/<name>.map.json` | where each ASN.1 component lives in the hexpat tree (for comparisons) |
| `asn1/<name>/HexpatHarness.scala` | decodes a file with the generated decoders, prints JSON, re-encodes |
| `asn1/<name>/notes.txt` | the emitter's notes (caps, lossy lengths, signedness) |
| `asn1scc-scala-fixes.patch` | fixes for ASN1SCC's Scala backend (against commit `5633b5b`) |
| `stainless.conf` | ASN1SCC's own Stainless configuration (`asn1scala/stainless.conf`) |
| `results/<name>.txt` | Stainless summary per format |
| `build/` | generated Scala, classes, logs (not committed) |

## Translation

| hexpat | ASN.1 | ACN |
|---|---|---|
| `u16` in a little-endian file | `INTEGER (0..65535)` | `[encoding pos-int, size 16, endianness little]` |
| `struct S { … }` | `S ::= SEQUENCE { … }` | `S [] { … }` |
| `enum E : u8 { … }` | `E ::= INTEGER (0..255)`, names in a comment | `E [encoding pos-int, size 8]` |
| `char x[4]`, `u8 x[4]` | `OCTET STRING (SIZE(4))` | `[]` |
| `u8 x[len]`, `T x[len]` | `OCTET STRING (SIZE(0..max))`, `SEQUENCE (SIZE(0..max)) OF T` | `len INTEGER [...]` (ACN-inserted), `x [size len]` |
| `if (c) { A a; } else { B b; }` | `a A OPTIONAL, b B OPTIONAL` | `a [present-when (c)], b [present-when (!(c))]` |
| `match (f) { (1): …; (2 \| 3): …; (_): … }` | OPTIONAL components | `present-when (f == 1)`, `(!(f == 1) and (f == 2 or f == 3))`, … |
| `type::Magic<"qoif">` | `magic INTEGER (1903126886)` | `[encoding pos-int, size 32]` |
| `bitfield B { a : 3; b : 5; }` | `SEQUENCE { b INTEGER (0..31), a INTEGER (0..7) }` | `[encoding pos-int, size 5]`, … (stream order) |
| `padding[n]` | `padding OCTET STRING (SIZE(n))` | `[]` |

ACN's rules shape the rest of the translation:

- **Sizes come from ACN-inserted fields.** ASN1SCC rejects ASN.1 fields as
  length determinants, so a hexpat length field leaves the ASN.1 value and
  the encoder recomputes it from the array. If every array that uses it is
  conditional, the value cannot be recovered when the array is absent; the
  emitter lists such fields as *lossy*. PNG's `Chunk.length` is the one
  case here: for an IHDR chunk the re-encoded length is 0 instead of 13.
- **Conditions are expressions over plain ASN.1 fields of the same
  SEQUENCE.** `present-when` cannot refer to `child.field`, so a condition
  or length that does (ELF's `ident.class`, ICO's `header.count`) inlines
  the child's components into the parent as `ident-class`, … .
  A `char[n]` (n ≤ 8) compared with a string literal is read as an `n`-byte
  big-endian integer so that the comparison is an integer comparison
  (PNG's `type == "IHDR"` becomes `type-field == 1229472850`).
- **Every size needs an upper bound.** ASN1SCC sizes buffers for the worst
  case, and the generated decoders require that many bytes to be
  addressable. Bounds are the length field's range capped at `--max-len`
  (default 1 MiB).
- **Open enums.** hexpat enums accept unnamed values, so they become INTEGER
  types; an ENUMERATED would reject them.
- **No "until the end of the input".** ACN's `size deduced` is not
  available in the Scala backend, so a trailing `T x[while(!std::mem::eof())]`
  of the root struct is left out of the root SEQUENCE and the harness decodes
  `T` repeatedly until the input ends (PNG chunks, ZIP entries). That loop
  is the only unverified decoding code.

The emitter also works around gaps in ASN1SCC's Scala runtime that a patch
cannot fix without new proofs:

- `s16`/`s32`/`s64` fields are read as unsigned integers of the same width.
  The runtime has the `_pure` decoders, postconditions and prefix lemmas that
  `-stainlessInvertibility` needs only for 8-bit two's complement.
  `--twos-complement` emits the signed encodings for the other backends.
- Magics are single-value INTEGER fields rather than ACN-inserted
  `NULL [pattern ...]` fields, whose invertibility proofs are placeholders
  (`TODO_ACN_OTHER_prefixLemma`), or OCTET STRING value constraints, for
  which the Scala backend prints C syntax.

## ASN1SCC fixes

The specs above hit six bugs in the Scala backend at ASN1SCC commit
`5633b5b` (2026-10-17). `asn1scc-scala-fixes.patch` fixes them; the C
output for all ten specs and for ASN1SCC's own `21-PresentWhenExpression`
tests is byte-identical with and without the patch.

1. The encoder's consistency check for an OPTIONAL component with a
   `present-when` expression referred to `x.x.isDefined` and compared it
   with the unparenthesized expression (`a != b == c`).
2. The decoder evaluated that expression inside the outlined decoding
   function of the component, which receives only the presence flag:
   it assigned to the parameter and referred to `pVal`, which does not
   exist yet when decoding. The patch evaluates the expression in the
   caller, over the decoded siblings (`pVal_field`).
3. The expression compared `ULong` fields with `Int` literals, which Scala's
   strict equality rejects; fields are read with `.toRaw` and literals get
   an `L` suffix.
4. A length determinant whose dependent is OPTIONAL read `nCount` through
   the `OptionMut` wrapper instead of its content.
5. The ghost size variables of two sibling OPTIONAL SEQUENCE components
   (ELF's two headers) had the same names (`size_1_8`, …).
6. The prefix lemma of a SEQUENCE with more than 21 nested size variables
   bound them through a Scala tuple and its `_N` accessors, which stop at 22.

ASN1SCC's own test case `21-PresentWhenExpression/001` still does not
compile in Scala after the patch (5 errors before, 3 after): the remaining
errors are in optional ENUMERATED decoding, which these specs do not use.

## Results

`hexpat/tests/asn1.rs` decodes 22 files (synthetic samples plus the
ImHex-Patterns corpus fixture of each format) with the generated Scala
decoders and with the hexpat interpreter:

- every decoded field agrees (19 to 58 fields per file), including ELF with
  a 32-bit, a 64-bit and an unknown class, PNG with eight chunks of the
  corpus file, and ZIP entries with names and extra fields;
- the corpus ZIP file, whose central directory `zip.hexpat` does not
  describe, is rejected by both;
- for every decoded value, decoding its re-encoding gives the same value;
- the re-encoded bytes are identical to the input except for PNG IHDR
  chunks, whose lossy `length` is written as 0.

### Stainless

`scripts/verify-asn1.sh` runs Stainless 0.9.8.9 with ASN1SCC's own
`stainless.conf` (solvers cvc5, z3 and cvc4 as a portfolio, 120 s per VC)
on every function ASN1SCC generated for a format; calls into the
`asn1scala` runtime are checked against the runtime's contracts.

**Safety mode** (code generated without `-stainlessInvertibility`):
absence of runtime errors (array bounds, integer overflow, preconditions of
every runtime call, match exhaustiveness), termination, and the exact number
of bits each encoder writes and each decoder reads, as stated by the
functions' postconditions.

| Format | VCs | Valid | Unknown (timeout) | Invalid | Wall time |
|---|---:|---:|---:|---:|---:|
| bmp | 988 | 988 | 0 | 0 | 32 min |
| elf | 4111 | decoders 1706 / 1706; encoders in progress | – | 0 | – |
| gif | 796 | 796 | 0 | 0 | 7 min |
| gzip | 720 | 719 | 1 | 0 | 14 min |
| ico | 1330 | 1330 | 0 | 0 | 16 min |
| png | 1521 | 1521 | 0 | 0 | 21 min |
| qoi | 298 | 298 | 0 | 0 | 2 min |
| tar | 912 | 912 | 0 | 0 | 60 min |
| wav | 747 | 747 | 0 | 0 | 9 min |
| zip | 848 | 845 | 3 | 0 | 61 min |

"Unknown" means no solver answered within 120 s; none of the VCs was
found invalid. The VCs reported as unknown are listed in
`results/<name>.safety.txt`. "From cache" VCs in those files were proven
in earlier runs of the same Stainless version on the same code, so the
wall times are those of the final runs, not of a run from scratch. ELF's
4111 VCs are split into its decoders (1706) and everything else (2405).

**Invertibility mode** (code generated with `-stainlessInvertibility`, which
adds `decode(encode(v)) == v` for SEQUENCE types and the prefix lemmas it
relies on) was attempted on QOI: 537 of its 1005 VCs had been checked when
the run was stopped, the remaining ones taking up to the 120 s timeout each.
Completing it, and the larger formats, needs far more solver time than the
4-core, 13.7 GB container this was run in.


## Reproducing

```console
$ scripts/setup-asn1-tools.sh            # ASN1SCC (patched) and Stainless 0.9.8.9 from source
$ export ASN1SCC=… STAINLESS=… SCALAC=… SCALA_LIB=…
$ scripts/regen-asn1.sh                  # specs, generated Scala, compiled harness
$ cargo test -p hexpat --test asn1       # differential test (skipped without build/)
$ scripts/verify-asn1.sh                 # Stainless, summaries in results/
```

The specs alone need only the hexpat binary:
`hexpat emit-asn1 formats/png.hexpat -o out/` writes `png.asn`, `png.acn`,
`png.map.json` and `HexpatHarness.scala`.
