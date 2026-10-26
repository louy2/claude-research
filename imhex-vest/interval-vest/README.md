# interval-vest: root-relative intervals and ranked jumps on vest_lib

A prototype of the extension of Interval Parsing Grammars (IPG; Zhang,
Morrisett and Tan, PLDI 2023, [arXiv:2304.04859](https://arxiv.org/abs/2304.04859))
needed for directory-based formats such as ZIP. The extension is verified
in Verus and instantiated for a strict ZIP profile.

## Why

IPG gives every nonterminal an interval relative to its parent. Attributes
are synthesized only, and an array element's interval cannot depend on the
previous element (`lib/transformer.ml` in the PLDI artifact forbids
`A[A(i).end, …]`). These restrictions give compositionality and a simple
termination argument: intervals shrink along every recursion cycle. They
also rule out the ZIP central directory. Its entries have variable length,
and each holds an absolute offset into the rest of the file. The artifact's
ZIP grammar consequently reads the file forward as a list of `PK` records and
never follows `ofs_central_dir` or `ofs_local_header`.

This crate keeps IPG's interval view and adds the missing pieces:

| piece | what it does |
|---|---|
| `At(l, r, c)` ([`spec_at`](src/ranked.rs)) | runs a Vest format `c` on `root[l..r)` of the **whole** input; the footprint is `[l, l + n)` |
| backward scan ([`spec_scan_back`](src/ranked.rs)) | the last `p` where `c` parses `root[p..)` exactly: records anchored at the end (ZIP end record, PDF trailer) |
| ranked jumps ([`spec_resolve`](src/ranked.rs)) | a node's parsed value yields jumps `(kind, l, r)` into the whole input; a jump is followed only to a kind of **strictly lower rank**, so `rank` is the termination measure for any input |
| local structure | within a region, ordinary Vest combinators; a variable-length table is a plain `Vec<entry>` |
| links | a format-specific check between a node and each node it jumped to (e.g. local header vs central entry) |
| tiling ([`spec_tiled`](src/ranked.rs)) | in the format's layout order, regions are contiguous from 0 to the end of the input, and each region appears once |

## What is proven

Verus checks the following for every format that implements `RankedFormat`
(`src/ranked.rs`):

- **Termination.** `spec_resolve` / `spec_resolve_children` are mutually
  recursive with the measure `(rank, ·, ·)`, and the rank check is part of
  the definition.
- **Geometry** ([`theorem_regions_disjoint`](src/ranked.rs)). The regions of
  an accepted input are in bounds and pairwise disjoint, and every byte of
  the input lies in one of them.
- **Non-malleability** ([`theorem_non_malleable`](src/ranked.rs)). Two
  accepted inputs with the same regions and values are equal byte strings.
  The proof combines tiling with Vest's per-region `PSRoundTrip`: a region's
  bytes are the serialization of its value.
- **Executable correctness.** `parse_file` returns `Some(rs)` exactly when
  `spec_parse_file` does, with the same regions and values; the same holds
  for `resolve`, `at`, `scan_back` and `check_tiled`. Memory safety and
  absence of panics come from Verus.

For ZIP (`src/zip.rs`), the trait implementation is verified against its
spec. [`lemma_zip_jumps_descend`](src/zip.rs) shows that ranks are a static
property of the format: no parsed ZIP node ever jumps sideways or up, so the
driver's rank check never rejects. [`theorem_zip_non_malleable`](src/zip.rs)
is the generic theorem specialized to ZIP.

The trusted base is Verus, z3 and vstd, plus vest_lib's combinator lemmas.
`cargo verus focus` does not re-verify dependencies; `cargo verus verify`
does.
`spec_layout`, the order in which regions must tile, is a modelling choice of
the format. Nothing proves that it matches what other ZIP readers expect.

```console
$ ../scripts/setup-verus.sh          # Verus 0.2026.09.06.8dea4a2, z3 4.16.0, Rust 1.98.0
$ cargo verus focus                  # in this directory
verification results:: 200 verified, 0 errors
```

The 200 include the generated region formats. Everything verifies at
Verus's default resource limit.

## The strict ZIP profile

| kind | rank | region | jumps |
|---|---|---|---|
| end of central directory | 2 | found by the backward scan; must end the file | central directory at `[cd_offset, cd_offset + cd_size)`, which it must fill |
| central directory | 1 | `Vec<cd_entry>` | one local file per entry, at `local_header_offset` |
| local file | 0 | header, name, extra, compressed data | — |

- Records ([`vest/zip_regions.vest`](vest/zip_regions.vest)): single disk,
  flags limited to bits 1–2 and 11 (so no encryption and no data
  descriptors), stored or deflated, no ZIP64. The fields a local header
  repeats from its central entry are one shared `file_info` record.
- Node check: `entries_disk == entries_total`.
- Links: the central directory has `entries_total` entries, and each local
  header has the same `file_info` and name as its central entry.
- Layout: local files in central-directory order, then the central
  directory, then the end record, with no gaps.

`scripts/make-zip-fixtures.py` writes the fixtures in `tests/data/`.
`tests/zip_profile.rs` checks that every `ok-*` fixture is accepted and every
`bad-*` fixture is rejected, at the stage expected:

| fixture | `unzip -t` | rejected by |
|---|---|---|
| `bad-trailing-byte` (one byte after the end record) | ok | backward scan: no end record ends the file |
| `bad-prefixed` (4 bytes before, offsets not adjusted) | error | resolve: no central directory at `cd_offset` |
| `bad-name-mismatch` (central name ≠ local name) | error | resolve: link |
| `bad-count-mismatch` (`entries_total` off by one) | error | resolve: node check |
| `bad-data-descriptor-flag`, `bad-infozip-stdin-data-descriptor` | error / ok | resolve: `flags` refinement |
| `bad-gap-between-files`, `bad-gap-before-cd` (padding, offsets valid) | ok | tiling |
| `bad-local-order` (local files in reverse order) | ok | tiling |
| `bad-overlap` (two central entries, one local file) | error | tiling: both links hold, only tiling sees the overlap |
| `bad-fake-eocd-in-comment` (comment ends with an empty archive's end record) | error | tiling: the scan finds the fake record, which resolves, but it cannot tile the file |

A last test flips every byte of an accepted archive in turn and checks that
each result is either rejected or parses to something different. That is
the non-malleability theorem, checked on one input.

On other archives on the test machine (`examples/zipcheck.rs`), the profile
accepts 334 of 398:

- 13 of 13 Python wheels and 273 of 309 jars;
- Go's `archive/zip` test archives written by 7-Zip, WinRAR, WinZip, macOS,
  Windows 7/XP and Info-ZIP.

The rejections are archives with data descriptors (most rejected jars, Go's
`dd.zip`), ZIP64, and Go's malformed test cases (`test-prefix`,
`test-trailing-junk`, `test-badbase`, `test-baddirsz`, `comment-truncated`,
and `readme.zip`, which has one byte after its comment).

## Relation to IPG

| IPG | here |
|---|---|
| `A[el, er]`, relative to the parent's interval | inside a region: Vest's sequential combinators; across regions: `At` on the whole input |
| synthesized attributes in intervals | jumps computed from the parsed value |
| termination: Z3 checks that intervals shrink on each grammar cycle | termination by rank, checked by Verus once for all formats |
| overlapping intervals allowed (PDF's two passes) | regions must tile; overlap is rejected |
| no serializer | per-region Vest serializers; the file is determined by its regions |
| backward parsing (`bInt[0, EOI-1]`) | `spec_scan_back` for end-anchored records |

## Not done yet

- **Generator direction.** No function yet builds an archive from a list of
  files, and nothing proves that `parse_zip(build(files))` recovers the
  files. This is IPG's "reverse attribute computation". It needs a
  side condition on the comment: it must not end with bytes that form a
  valid end record, the case `bad-fake-eocd-in-comment` exercises.
- **ZIP64 and data descriptors.** Both fit the scheme. ZIP64 needs a
  locator and record at rank 3 above the end record. Data descriptors need a
  local-file region that extends past the data.
- **Duplicate names** and other semantic checks from the ZipDiff study.
  These belong in `spec_node_ok` on the central directory.
- **Parquet.** The footer is end-anchored (`PAR1` and a 4-byte length) and
  `FileMetaData` points at column chunks, so the driver applies directly.
  The missing piece is a Thrift-compact combinator with a canonical profile,
  in the way PulseParse handles deterministic CBOR.
