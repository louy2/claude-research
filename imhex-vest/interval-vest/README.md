# interval-vest: root-relative intervals and ranked jumps on vest_lib

A prototype of the extension of Interval Parsing Grammars (IPG; Zhang,
Morrisett and Tan, PLDI 2023, [arXiv:2304.04859](https://arxiv.org/abs/2304.04859))
needed for directory-based formats such as ZIP and Parquet. The extension
is verified in Verus and instantiated for a strict ZIP profile and a strict
Parquet profile; the Parquet footer goes through a verified canonical Thrift
compact codec.

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
| sorted layout ([`spec_sorted_layout`](src/ranked.rs)) | for formats whose regions have no fixed order (Parquet's indexes and bloom filters): the layout is the regions sorted by offset |

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

For ZIP (`src/zip.rs`) and Parquet (`src/parquet.rs`), the trait
implementations are verified against their specs.
[`lemma_zip_jumps_descend`](src/zip.rs) and
[`lemma_parquet_jumps_descend`](src/parquet.rs) show that ranks are a static
property of each format: no parsed node ever jumps sideways or up, so the
driver's rank check never rejects. [`theorem_zip_non_malleable`](src/zip.rs)
and [`theorem_parquet_non_malleable`](src/parquet.rs) are the generic
theorem specialized to each format.

The trusted base is Verus, z3 and vstd, plus vest_lib's combinator lemmas.
`cargo verus focus` does not re-verify dependencies; `cargo verus verify`
does.
`spec_layout`, the order in which regions must tile, is a modelling choice of
the format. Nothing proves that it matches what other readers expect.

```console
$ ../scripts/setup-verus.sh          # Verus 0.2026.09.06.8dea4a2, z3 4.16.0, Rust 1.98.0
$ cargo verus focus                  # in this directory
verification results:: 338 verified, 0 errors
```

The 338 include the generated region formats. Everything verifies at
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

## Canonical Thrift compact

[`src/thrift.rs`](src/thrift.rs) parses Thrift compact values from any
position of a byte sequence, schema-less, into a tree of booleans, integers,
doubles, binaries, lists and structs. It accepts one encoding per value:

- minimal varints below 2^64; `i16`/`i32`/`i64` zigzag-decoded and in range;
- a short field header exactly when the id delta is 1–15; field ids strictly
  increase;
- a short list header exactly when the size is below 15; list booleans are
  the bytes 1 and 2;
- no sets or maps; nesting at most 16 deep.

The list element type is part of the value. Writers use 1 or 2 for lists of
booleans, and fastparquet writes 0 for an empty list, so these choices do not
create ambiguity.

Proven: [`lemma_value_ps`](src/thrift.rs) shows that the serializer
reproduces exactly what the parser consumed, so
[`theorem_thrift_non_malleable`](src/thrift.rs) holds: two inputs that parse
to the same struct consumed the same bytes. The executable parser
(`parse_value`), field lookup (`get_field`) and value equality (`tval_eq`)
are verified against the spec. The parser also proves that a parsed tree is
no deeper than its depth budget; the Parquet copy check relies on that when
it compares values parsed at different depths.

## The strict Parquet profile

| kind | rank | region | jumps |
|---|---|---|---|
| tail | 2 | the last 8 bytes: metadata length, `PAR1` | the FileMetaData before it, which it must fill |
| FileMetaData | 1 | one Thrift struct | `PAR1` at 0; per column chunk: the chunk, its inline copy, column index, offset index, bloom filter |
| head | 0 | `PAR1` | — |
| column chunk | 0 | page header + page data, repeated to the end of the chunk | — |
| inline copy | 0 | a Thrift struct at `file_offset` | — |
| column index, offset index | 0 | a Thrift struct filling its interval | — |
| bloom filter | 0 | a Thrift header, then `numBytes` of bitset | — |

- **FileMetaData checks:**
  - no encryption;
  - no chunk in another file (`file_path`);
  - offsets, sizes and counts are non-negative and of the right Thrift type;
  - an index offset comes with its length;
  - `num_rows` is the sum of the row groups' `num_rows`.
- **Column chunk checks**, against values the FileMetaData passes down in
  the jump (the kind carries them, like a parameterized nonterminal):
  - a dictionary page comes first exactly when `dictionary_page_offset` is
    set, and every other page is a data page (v1 or v2);
  - `data_page_offset` is the first data page;
  - the data pages' `num_values` sum to the chunk's `num_values`;
  - headers plus uncompressed page sizes sum to `total_uncompressed_size`.
- **Inline copies.** Older writers (Impala, parquet-cpp 1.x, parquet-rs 9)
  put a copy of the ColumnChunk, or of its ColumnMetaData, after the chunk
  and point `file_offset` at it. When `file_offset` is the end of the chunk,
  the copy must be there and must equal the footer. That is the same
  redundancy ZIP has between local and central headers.
- **Layout:** all regions sorted by offset, which must tile the file.

`scripts/make-parquet-fixtures.py` writes the fixtures in `tests/data/`.
- **Accepted:**
  - pyarrow 26 files: plain, snappy, v2 data pages, several row groups, no
    dictionary, page index, bloom filter, and an empty table;
  - DuckDB 2.0 (with bloom filters) and fastparquet 2026.9;
  - a legacy layout with inline copies.
- **Rejected:** files that each break one property. pyarrow reads all but
  two of them:

| fixture | pyarrow | rejected by |
|---|---|---|
| `bad-trailing-byte` | error | resolve: the tail is not `PAR1` |
| `bad-no-head` (`PAR0` at 0) | reads | resolve: head |
| `bad-num-rows`, `bad-num-values`, `bad-uncompressed-size`, `bad-data-page-offset` | reads | resolve: node checks |
| `bad-inline-copy-mismatch` (copy's `num_values` differs from the footer) | reads | resolve: link |
| `bad-nonminimal-varint`, `bad-long-field-header`, `bad-field-order` (same FileMetaData, another encoding) | reads | resolve: canonical Thrift |
| `bad-gap-between-chunks` (3 bytes, offsets shifted) | reads | tiling |
| `bad-overlapping-chunks` (two chunks at the same bytes) | error | tiling |
| `bad-polars-unreferenced-copy` (written by Polars 2.1) | reads | tiling: see below |

Polars writes a ColumnMetaData copy after every chunk, but sets `file_offset`
to `data_page_offset + total_compressed_size`. For a chunk that starts with a
dictionary page, that points past the copy, which is then not referenced from
anywhere.

On the [apache/parquet-testing](https://github.com/apache/parquet-testing)
corpus (`examples/parquetcheck.rs`), the profile accepts 223 of 241 files.
Rejected:

- 6 of the 9 top-level `bad_data` files. The other three, and the
  `bad_data/variants` files, are malformed inside page contents (levels,
  dictionary values, variant encodings), which a layout profile does not
  look at.
- 5 files from parquet-mr 1.8–1.12 that write a dictionary page without
  setting `dictionary_page_offset`, and point `data_page_offset` at it.
- 2 files whose inline copy differs from the footer (parquet-rs 49 copies
  the ColumnMetaData before it knows the bloom filter offsets).
- `byte_array_decimal` (HVR): 25 unreferenced bytes after its chunk.
- `dict-page-offset-zero`: `dictionary_page_offset` is 0.
- `nation.dict-malformed`: pages overrun the chunk.
- `repeated_no_annotation` (parquet-rs 0.3): `num_rows` is 0 but the row
  group has 6 rows.

A 235 MB pyarrow file with 2,000 column chunks and a page index checks in
0.17 s.

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
- **Parquet.**
  - Page contents (levels, encodings, compression) are not checked.
  - Nor is the agreement between an offset index and the pages it lists.
    That link runs between two siblings, while links here run from a
    parent to its child.
  - Encryption and `file_path` chunks are out of the profile.
- **Thrift serialize-then-parse.** Only the parse-then-serialize direction
  is proven. The other direction would give a verified writer for the
  footer.
