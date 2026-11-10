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
| xilinx_bootgen | `32:1: cannot read 4 byte(s) at 0x2FFFFFFFD` |

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
