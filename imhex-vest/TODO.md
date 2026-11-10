# TODO

Open items found while building and checking this repository, with what
is known about each. Checks against ImHex used the 1.38.1 release and the
nightly 1.39.0.WIP (built 2026-10-08 from 34483cb), run through
`imhex --pl format -f json` (see `scripts/compare-imhex.py`).

## hexpat interpreter: matching ImHex

Corpus state (first fixture of each pattern, `scripts/compare-imhex.py`
against the nightly, 177 patterns):

| | patterns |
|---|---|
| JSON byte-identical | 94 |
| both succeed, JSON differs | 35 |
| nightly aborts, hexpat succeeds | 30 (see [Upstream](#upstream-reports)) |
| nightly succeeds, hexpat fails | 13 |
| both fail | 5 |

`hexpat/tests/corpus.rs` (hexpat alone) evaluates 159 of 177.

### Byte-faithful strings

- [ ] Keep strings as raw bytes. Today `lower::bytes_to_string` decodes a
  byte slice as UTF-8 when it is valid and maps each byte to U+0000–U+00FF
  otherwise. Afterwards, `§` decoded from `C2 A7` and an invalid lone `A7`
  are the same character, so the JSON export escapes the first as `§`
  where ImHex writes `§`.
  - This is the last difference on gvas (`ue4_utf-16.sav`).
  - It also affects `gguf`, `mo` and `nbt`.
  - It also means `std::string::length`/`at`/`substr` count characters,
    where ImHex counts bytes. They are now at least consistent with each
    other (commit 8f3dc46).
  - Possible approach: map invalid bytes to a private-use range, and add a
    `str → bytes` helper for the JSON export, hashing and the
    `std::string` builtins.

### Patterns the nightly parses and hexpat does not

| pattern | hexpat error |
|---|---|
| adts | `278:1: cannot use pattern "Header" as an integer` |
| blend | `36:1: assertion failed 'Invalid magic value!…'` (reads zeros at 0xE3A08) |
| bsp_goldsrc | `138:1: cannot read 3657459902029883000 byte(s) at 0x1BDC` |
| dotnet_binaryformatter | `234:1: cannot read 8 byte(s) at 0x16D: out of bounds` |
| ico | `in import png: 197:1: cannot read 982691674 byte(s) at 0x11C` |
| notepadwindowstate | `772:1: cannot read 1 byte(s) at 0xA88` (end of file) |
| popcap_luc | `167:1: cannot read 4 byte(s) at 0x69` (end of file) |
| sdb | `88:1: std::string::substr: position out of range` |
| uefi_boot_entry | `49:1: cannot read 1 byte(s) at 0x72` (end of file) |
| vdf | `125:1: expected a non-negative value, got -1` |
| veado | `9:1: cannot read 1 byte(s) at 0x12F6` (end of file) |
| vhd | `in import fs/mbr: 52:1: unknown type MBR` |
| xilinx_bootgen | `36:1: termination check: … instantiated inside itself at 0x2FFFFFFFD, past the end of the data` (before the measure checks: `cannot read 4 byte(s) at 0x2FFFFFFFD`) |

Several are reads at or past the end of the file that ImHex tolerates.
How it does (PatternLanguage 300d738): main-section reads are not
bounds-checked. The `format` CLI path does a short `fread` into a buffer
that every reader zero-initializes, and `run` zero-fills explicitly. So
a single value that ends past the end reads as zeros. The only end checks
are on a placement's start offset (`<= size`) and on arrays ("Array
expanded past end of the data"). Matching this means zero-filling scalar
and struct-member reads but keeping the array checks.

Both fail, but the nightly only because of its own crash, so these are
ours to fix once it parses: `bencode` (read at end of file), `id3` (read
at end of file), `parquet` (`504:1: pattern value has no member value`),
`protobuf` (`36:1: Unknown WireType`). `dod-save` fails in both
("Array expanded past end of the data" in ImHex; `string index 9 out of
range` here).

### JSON differences (both succeed)

First differing line, ImHex nightly vs hexpat:

- [ ] **Enum values in bitfields and some enums print as numbers:**
  - `ne` (`DGroupType::MultipleData` vs `2`)
  - `nes` (`MirroringType::Vertical` vs `1`)
  - `usb` (`EndpointDirection::IN` vs `1`)
  - `keytab` (wrong entry: `unassigned`)
  - `terminfo` (`extended` vs `number_format::extended`)
- [ ] **Strings stop at the first NUL in the nightly:** `cab`, `job`,
  `uefi_fv_varstore` (we keep `\u0000`). 1.38.1 behaviour to be checked
  before following it.
- [ ] **A sealed struct without a format function prints its contents in
  the nightly** (`struct Model { vertices = [ … ] }`): `stl`, `pck`, `pcx`.
- [ ] **Array entries get display names** (`"qoi::op_index @ 0x0E"`,
  `"RecordHeader @ 0x64"`, `"SubCurve<…> @ 0x346"`) where we print `""`:
  `qoi`, `shp`, `wfm`.
- [ ] **Different values or members.** These need a look one by one:
  - `bplist` (`marker` `"51"` vs `"null"`)
  - `fez` (`key`)
  - `hprof` (`0xA0000001` vs `0x2684354561`)
  - `java_class` (`ACC_PUBLIC` 1 vs 0)
  - `nro` (pointer target 255 vs 65)
  - `sqlite3` (`pageSize` 32 vs 8192)
- [ ] **Extra or missing members:**
  - `dos` (`__goto__lastByteInProgramImage`)
  - `midi` (`g_next_status`)
  - `mp4` (`unk`)
  - `pe` (`resourceIDType`)
  - `pex` (`g_GameId`)
  - `selinux` (`type_g`)

  Several look like globals or locals that ImHex does not export.
- [ ] **Errors inside formatter output:**
  - `csnap` (`std::string::substr: position out of range`)
  - `rar` (`` `vi` is not a pattern ``)
  - `tiff` (`recursion limit reached calling format_read_big`)
- [ ] **Trailing commas** after the last entry: `btrfs_send_stream`,
  `pcapng`, `q3demo`.

### Termination

Done: every evaluation stops at a limit, which the ImHex limit pragmas can
raise up to a cap; the measure checks stop a run where it goes wrong (`Limits`, `hexpat/tests/termination.rs`); the static
check gives a verdict per loop, array and cycle (`hexpat termination`,
`hexpat/src/termination.rs`, `scripts/termination-sweep.py`,
`scripts/termination-fuzz.py`). On the 243 corpus patterns: 190 fully
proven, 24 more decided by run-time checks, 29 with something not proven
(37 obligations).

Open:

- [ ] The limit pragmas only raise a limit, up to a cap. ImHex also lets
  a pattern lower one and switch one off with `0`. If exact ImHex limit
  errors are ever wanted (for the JSON comparison), the counts have to
  match ImHex's first: it counts scopes for the depth and applies the
  array limit before reading.
- [ ] The command line has no flags for the limits or the caps; only a
  pragma or the library API changes them.
- [ ] Memory is bounded only loosely: built strings stop at 64 MiB each
  and heap sections at 16 MiB each, but nothing bounds how many are alive.
- [ ] The depth limit of 256 needs about 8 MiB of stack in a debug build
  (2 MiB in release). The CLI runs on a 1 GiB thread; library callers
  get whatever stack they run on.
- [ ] The measure checks refuse an array whose entries read nothing and
  that ends because its condition counts them (ImHex accepts it). No
  corpus fixture does this. If one turns up, let a counter in the
  condition stand in for cursor progress.
- [ ] Zero-filling reads past the end (to match ImHex, see above) keeps
  arrays and nested instances bounded, because the measure checks compare
  them with the end of the data. A `while` statement in a struct body that
  is proven by cursor progress would lose its bound; give it the same end
  check when that change is made.
- [ ] Loops proven by a counter or by cursor progress rest on the static
  argument alone (the run-time check only catches a wrapping counter).
  Checking them per site needs a way to name a statement across the
  analysis and the interpreter; lines repeat between files.
- [ ] Outside the measures, left to the depth limit: template arguments
  that take a new value at every level (`T<n + 1>`), and placements into
  sections created along the way.
- [ ] More arguments, by what the corpus needs (37 unproven obligations,
  and 49 that are only decided at run time):
  - a cursor assignment that provably moves forward: `$ += n` with an
    unsigned `n`, `$ = start + size` where the entry already read `size`
    (20 arrays, 4 cycles);
  - an `if`/`match` chain over an enum that covers every value, so that an
    entry cannot be empty (part of the 17 "entry can be empty");
  - loops that shift or divide a value towards zero (`x >>= 1`,
    `x /= 1024`), and loops whose exit is a data-dependent scan
    (34 loops);
  - a bound on an expression that grows with the counter (`$ + i < end`,
    as in `ne`);
  - a decreasing argument for recursive functions (1 in the corpus);
  - a template argument that is computed but stays in a finite set (the
    2 cycles in gvas and one using `std::ptr::NullablePtr`).
- [ ] `import * from x as Name` (a whole pattern as a type) is not
  followed; the analysis reports the type as unknown.
- [ ] `hexpat lint`, next steps:
  - `duplicate-member` does not see that two `match`es on related values
    exclude each other (two of the five gvas findings are of this kind).
  - Lines of text pulled in with `#include` are reported as the
    pattern's own, with the included file's line numbers.
  - A check that a declared size matches what was parsed, and which
    branches of a pattern no test file exercises (branch coverage).
  - `missing-case` could feed the termination check: a dispatch with a
    branch for every enum value cannot leave an array entry empty.
- [ ] Evaluation gets slower the deeper types nest. Found by
  `scripts/termination-fuzz.py`: a 64 KiB bencode input nested about 250
  deep took 18 s for 1 million steps, because every name lookup rebuilt
  its candidate list once per enclosing scope. That is fixed (2.2 s); the
  time still grows faster than the input, so other per-scope costs remain.
- [ ] The depth limit counts type nesting and function calls together, so
  an input nested close to 256 deep fails in whatever function is called
  there (`recursion limit reached calling read_unsigned`). The measure
  would allow the nesting; the limit is a cap on the stack. Consider a
  separate, larger limit for type nesting.

### Other

- [ ] ImHex 1.38.1 aborts on the pattern in `hexpat/tests/json.rs`
  (`objects_arrays_strings_enums_and_pointers`); without the pointer
  member it exits with an error. Minimize it, so that our pointer and
  padding JSON can be checked against a reference.
- [ ] Decide which ImHex version the JSON export targets. 1.38.1
  percent-encodes strings; the nightly writes JSON escapes and changed
  string and sealed-struct output.
- [ ] Make `scripts/compare-imhex.py` part of a check (it needs an
  unpacked ImHex at `/app`).

## Upstream reports

Not filed; drafts only.

- [ ] **ImHex nightly: invalid UTF-8 in a `char` array aborts `--pl format -f json`.**
  - Repro: `char x[1] @ 0;` over the byte `0x89`.
  - Cause: `PatternString::getValue` now decodes through the string codec
    and throws E0004 "invalid byte sequence for encoding".
    `JsonPatternVisitor::formatString` calls it via `toString()` and does
    not catch the error, so the process aborts (SIGABRT).
  - Impact: 27 corpus patterns fail on their first fixture, all of which
    1.38.1 parses. 7z, bgcode, chm, commodore_basic, credhist, dmp64,
    dpapiblob, dpapimasterkey, fbx, fdt, hinf_luas, hinf_module, hinf_tag,
    jpeg, ktx, lua52, lua53, nacp, notepad-cache, pyc, pyinstaller, ras,
    ttf, bencode, id3, parquet, protobuf.
- [ ] **ImHex nightly: `gltf.hexpat` aborts in `Evaluator::popScope`** (1.38.1 parses it).
- [ ] **ImHex: `arc`, `bcss`, `lcesave`, `roblox_pack` abort in an
  `Event<std::filesystem::path>` handler.** On 1.38.1 two of them parse
  and two exit with an error.
- [ ] **ImHex (both versions): `cpio_new_ascii` and `cpio_portable_ascii`
  abort in `Token::Literal::toString`.**
- [ ] **ImHex-Patterns `ne.hexpat`: a `u8` loop counter wraps.**
  `finddosmessage` and `findpointingcode` run
  `for (u8 i = 0, $+i < std::mem::read_unsigned(0x3C, 4), i = i + 1)`.
  When the header field is more than 255 past the cursor and the byte
  searched for is absent, `i` wraps to 0 and the loop never ends (ImHex
  stops at its loop limit). Found by `scripts/termination-fuzz.py` and
  reproduced; a wider counter type fixes it.
  - `pe.hexpat` has a related loop,
    `for (u8 cursor = $, cursor < $+16, cursor += 1)`. By reading, it
    cannot end when `$ + 16` is above 255; not reproduced.
- [ ] **ImHex-Patterns: findings of `hexpat lint`** (`scripts/lint-sweep.py
  --list CHECK`). Read against the source, not reproduced on a file;
  check each before filing.
  - `bcss.hexpat`: `const u8 max_path_size = 1000;` does not fit.
  - A `u8` (or `u16`, `s16`) loop counter against `sizeof(...)`, a string
    length or a wider field: `bink_container`, `dotnet_binaryformatter`,
    `fbx`, `gen`, `kindle_update`, `notepad-cache`, `notepad-state`,
    `notepadwindowstate`, `rar`, `rkvd`, `terminfo`.
  - A member declared twice: `msgpack` (`Type type`, then `s8 type`);
    after a partly failed `try`: `adtfdat`, `msf`, `tiff`; a default local
    and then a member of the same name: `pf`.
  - `pe.hexpat`: `FunctionTableEntry` has branches for 4 of the 28
    `ArchitectureType` values and reads nothing for the others.
- [ ] **PatternLanguage: the redeclaration check skips `match` arms.**
  - Duplicates at struct level and inside `if` are rejected at
    compile time (`validator.cpp` descends into `if`, not `match`).
    The same duplicates inside `match` arms are accepted.
  - The evaluator's struct check (`ast_node_struct.cpp`) does not see
    arm members either: match/if push straight into the scope.
  - Lookup then takes the last declaration.
- [x] **ImHex-Patterns `gvas.hexpat`: duplicated members in legacy saves.**
  Arrays of structs in pre-UE5 saves gave one `FPropertyTag` two
  `propertyType` members, and two each of `arrayIndex` and `terminator`
  (216 tags in the test saves). The pattern's author has submitted a fix
  upstream that reads the element tag as its own `FPropertyTag`, as their
  `gvas` crate does.
- [ ] **Polars 2.1 Parquet writer: the inline ColumnMetaData copy is
  unreferenced.** `file_offset` is set to
  `data_page_offset + total_compressed_size`, which misses the copy when
  the chunk starts with a dictionary page.
- [ ] **IPG PLDI'23 artifact `spec/zip/ipg/zip.ipg`:** `CentralDirEty`
  reads `last_mod_dos_datetime` at `btoi[6,10]`, overlapping
  `compression_method`. It should be `[8,12]`.

## interval-vest

### Generic layer (`src/ranked.rs`)

- [ ] Links between sibling regions (needed for a Parquet offset index
  against the pages it lists); links now run from parent to child only.
- [ ] The sorted layout is an insertion sort. It is fine for thousands of
  regions, but quadratic when regions arrive far out of order.

### ZIP profile

- [ ] **Generator direction.** Build an archive from files and prove that
  `parse_zip(build(files))` gives them back. Needs a side condition: the
  comment must not end with bytes that form a valid end record (the case
  `bad-fake-eocd-in-comment` exercises).
- [ ] ZIP64: a locator and record at rank 3 above the end record.
- [ ] Data descriptors: a local-file region that extends past the data.
- [ ] Duplicate names and other ZipDiff checks (`spec_node_ok` on the
  central directory).

### Parquet profile

- [ ] Page contents (levels, encodings, compression) are not checked.
- [ ] Offset index against actual page positions (needs sibling links).
- [ ] Encryption and `file_path` chunks are out of the profile.
- [ ] Decide on lenient variants for writer quirks the strict profile
  rejects:
  - parquet-mr 1.8–1.12 writes the dictionary page without setting
    `dictionary_page_offset`;
  - parquet-rs 49 writes inline copies without the bloom filter offsets.

### Thrift

- [ ] Serialize-then-parse direction (a verified writer for the footer);
  only parse-then-serialize is proven.
- [ ] Sets and maps, if a format needs them.

## Housekeeping

- [ ] In the cloud container used for the ImHex checks, `/app` is a
  symlink to an unpacked flatpak (`ln -sfn …`). Remove it there when done
  (`rm /app`; the automated removal was blocked).
