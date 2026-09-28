# Inmagic DB/TextWorks textbases: reading them without the software

## Why this format

DB/TextWorks (Inmagic, now Lucidea) was a Windows "textbase" program that
libraries, archives and museums used for catalogues in the 1990s and 2000s.
Its databases are sets of binary files, and no public specification exists.

In February 2026, Johan van der Knijff, a digital preservation specialist at
the KB (National Library of the Netherlands), wrote about a database of
photographed collection items from around 2000
([bitsgalore.org](https://bitsgalore.org/2026/02/16/emulating-inmagic-dbtextworks-databases-with-qemu/)).
His colleagues "were unable to either read the database with modern software,
or to get the original software running on modern hardware." To get the data
out, he installed Windows XP in QEMU, installed DB/TextWorks 7.0.1 and exported
the records to CSV. That export cut every field name to its first two letters
("Signatuur" became "SI"), so he had to record the real names from screenshots.
He then made two small CC0 test textbases, one with DB/TextWorks 3.0 and one with
7.0.1, and published them in the
[OPF format corpus](https://github.com/openpreserve/format-corpus/tree/master/DBTextWorks).
Copies are in [`samples/`](samples/), taken from commit `366f068` of that corpus.

This directory contains a reader for those files, written from hex dumps
alone. It needs no emulator and no vendor code.

## Result

[`dbtw.py`](dbtw.py) is a single Python 3 file with no dependencies.

```
$ python3 dbtw.py csv samples/3.0/vegetables
identifier,vegetable
1,cauliflower
2,broccoli
3,carrot
4,green bean

$ python3 dbtw.py csv samples/7.0/fruits
identifier,fruit
1,apple
2,pear
3,banana
4,grape
5,watermelon
```

* **The full field names are recovered.** They are stored in the `.DBS` file,
  so the reader avoids the two-letter truncation that the emulated export had.
* **The decoding is checked independently.** `dbtw.py verify` reads the
  term index and word index (`.BTX`) and the occurrence lists (`.OCC`), which
  the software maintains separately from the records, and checks that every
  index entry points to a record containing that term or word. The results
  are 20/20 checks for the 3.0 sample and 22/22 for the 7.0 sample.
* **One reader works for both versions.** Each binary file starts with a
  format revision and a date, and these are the same in 3.0 (1998) and 7.0.1
  (2004). All of the dates are from 1993–95, which suggests the on-disk format
  did not change for at least ten years of product releases.

## Format notes

All integers are little-endian. Every binary member file starts with a
16-byte ASCII header of the form `TAG rev mm/dd/yy`, for example
`DBR 001 10/13/94`. The date appears to be when that structure was last
revised, not when the file was created.

| File | Header | What it holds (observed) |
|---|---|---|
| `.DBS` structure | `DBS 009 05/18/95` | 1 KB of zeros. At `0x410`: u32 next automatic number, then u32 record count. At `0x41D`: a TLV stream (u8 tag, u16 length, data) that ends with tag `0x36`, followed by a repeat of the file header. |
| `.DBO` record directory | `DBO 012 12/05/94` | 6-byte entries indexed by record number (entry 0 is unused): u16 status (`0x0100` = live), u32 offset into `.DBR`. |
| `.DBR` records | `DBR 001 10/13/94` | u32 first slot (`0x6C`), u32 end of data. Records sit in 64-byte slots. Slot layout: `+0` u32 own offset, `+0x0E` 2-byte kind (`06 02`), `+0x10` u32 record number, `+0x14` u16 (always 1 so far), `+0x16` u16 payload length, then `+0x18` a payload of repeated `(u16 field id, u16 length, bytes)`. The text is Windows-1252 with no terminator. |
| `.BTX` indexes | `BTX 005 02/09/95` | 1 KB pages. At `0x400 + 4·field`: root page of that field's **term** index. At `0x800 + 4·field`: root page of its **word** index. Page *n* is at `0x1000 + (n-1)·0x400`. Each page holds u32 page id, u8 kind, u16 count, then `count` entries of 10 bytes: u16 string offset within the page, u32 hit count, i32 value. The strings are packed from the end of the page. In term indexes the value is a record number. In word indexes it is the negated offset of a `.OCC` slot. Numbers are indexed as `0x1D`, then a letter giving the digit count (`A` = 1 digit), then the digits (e.g. `\x1dA1`), which makes them sort correctly as strings. |
| `.OCC` occurrences | `OCC 005 02/09/95` | Uses the same 64-byte slots as `.DBR`, with kind `06 07`. There is one slot per indexed word: `+0x13` u32 count, then `+0x17` u32 record number, then the slot's own offset and the record number again. |
| `.TBA` definition | `TBA 020 02/09/95` | Chained blocks (own offset, size, pointer to previous block). Contains the creation date and time. In these samples it holds no forms or report layouts. `.TBU` is a backup copy with tag `TBU`. |
| `.IXL`, `.SDO` | | Empty in both samples: no validation lists and no deferred updates. |
| `.ACF` | `ACF 002 12/14/93` | A 4-byte body, `00 00 19 00`, used for multi-user locking. |
| `.LOG`, `.INI`, `.IDI` | text | The change log in CRLF plain text, and settings. |

The TLV tags seen in `.DBS`:

| Tag | Meaning |
|---|---|
| `0x32` | Start of the field list. The length word holds the number of fields; there is no data. |
| `0x33` | A field definition (18 bytes). `+4` u16 type: `0x0D` was used for the automatic-number field and `0x01` for the text field. `+8..9` hold flags; the second byte is `03` when the field has both a term and a word index. |
| `0x46` | An optional 24-byte extra block for the preceding field. It appears only on the automatic-number field. |
| `0x34` | End of a field definition. |
| `0x35` | The field names, NUL-separated, in field order. |
| `0x3C`, `0x3D` | Reserved blocks (601 and 81 bytes), all zeros here. |
| `0x3E`–`0x41` | Created date, created time, modified date, modified time, as text. |
| `0x44`, `0x45` | The stop-word list and the leading-article list: u16 count, then `count` u32 pointers, then NUL-separated strings. |
| `0x42`, `0x43` | Unknown (46 bytes of zeros; 7 bytes `01 01 00…`). |

## Side findings relevant to archivists

1. **Record slack keeps text from earlier records.** Each 64-byte record slot
   is filled from a reused memory buffer, so bytes after a record's payload
   hold whatever the previous record left behind. For example, record 2 in the
   3.0 sample is `broccoli`, followed by the leftover `wer` from `cauliflower`.
   In a real catalogue, this slack could hold text from edited or deleted
   records. `dbtw.py residue` lists it.
2. **Occurrence-list slack in the 7.0 sample leaks the working path.** The
   `.OCC` slots contain the string `E:\fruits\` and a timestamp. `E:` matches
   the virtual USB drive in the author's QEMU setup.
3. **Memory addresses are written to disk.** The field definitions and word
   lists contain Win32 heap pointers such as `0x00ac2a5a`. The software saves
   its in-memory structures without converting them. A reader has to skip
   these values, which change on every save and mean nothing on disk.
4. The 3.0 sample ships with Dutch leading articles (`'t`, `de`, `het`), which
   suggests a Dutch-localised installation.

## What is still unknown

These two samples are very small: one automatic-number field, one short text
field, and four or five records. The following have not been tested:

* Records longer than about 40 bytes, which would not fit in one 64-byte slot.
  They may be chained across slots or given larger slots.
* Fields with more than one entry. The CSV export separates entries with `|`.
  The reader assumes each entry is stored as a repeated `(field id, length)`
  pair in the payload, but no sample confirms this.
* Deleted records, records with deferred updates (`.SDO`), and `.DBO` status
  values other than `0x0100`.
* Date, number and image field types, and the other type codes in `0x33`.
* `.BTX` trees deeper than a single leaf page, and `.OCC` lists with more than
  one record or position.
* Forms, query screens and report layouts in `.TBA`, and validation lists in
  `.IXL`.

A real textbase from the late 1990s would settle most of these questions. The
KB's recovered photo database would be a good test case. `dbtw.py verify`
reports any index entry it cannot reconcile with the records, which makes it a
quick way to find where these notes are wrong.

## Usage

```
python3 dbtw.py info    <dir or any member file>   # files, versions, fields, word lists
python3 dbtw.py csv     <dir>                      # records as CSV with full field names
python3 dbtw.py verify  <dir>                      # check records against the indexes
python3 dbtw.py residue <dir>                      # stale bytes in record/occurrence slack
```

The sample files are CC0 (Johan van der Knijff); see `samples/readme.md`.
