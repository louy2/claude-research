# GraphPad Prism binary project files (`.pzf`): reading them without Prism

## Why this format

GraphPad Prism is a standard graphing and statistics tool in biomedical
research. From Prism 4 (2003) its projects were saved as `.pzf`, a binary
format. Prism 5 added the XML `.pzfx`, and Prism 10 a ZIP+JSON `.prism`, but
`.pzf` stayed available and researchers kept using it.

* GraphPad's own manual: "This is a binary format that can be opened by
  Prism 4 or later, but not by other applications"
  ([Prism user guide](https://www.graphpad.com/guides/prism/latest/user-guide/pzf_vs__pzfx_file_format.htm)).
* Open-source readers handle only the newer formats: the R package `pzfx`,
  `pzfx_parser`, and `prism2R` (for `.pzfx` and `.prism`).
  `prismfile_converter` mentions `.pzf` in its description, but its code
  rejects anything that is not `.pzfx`.
* Cambridge University's research-data team lists Prism files as "legacy
  software" whose files may become inaccessible
  ([Unlocking Research blog, 2024](https://unlockingresearch-blog.lib.cam.ac.uk/2024/05/03/formatting-the-future-why-researchers-should-consider-file-formats/)).
* It is still being deposited. A Zenodo search in October 2026 found **506
  `.pzf` files in 90 records** (2012–2026), more than many other legacy
  formats (`.jnb` 41, `.jmp` 32, `.wps` 47).

## Result

[`pzf.py`](pzf.py) is a single Python 3 file with no dependencies.

```
$ python3 pzf.py tables "samples/abdulkareem2024_Figure 5.pzf"
[0] 'Data 1'
     col 5   rows ''                                       text
     col 9   Y    'Untreated Lysozyme'                     10 cells x2
     col 2   Y    'Ens-LSZ-Tris-HCL (50mM'                 10 cells x2
     ...
$ python3 pzf.py csv "samples/abdulkareem2024_Figure 5.pzf" 0
,Untreated Lysozyme [1],Untreated Lysozyme [2],Ens-LSZ-Tris-HCL (50mM [1],...
Refrigerated ,49659,5497.137,,,,,,
Control,,,38678,3482.113,34821,3715.337,43809,5075.737
...
```

### Validation

1. **Exact comparison with published CSVs.** Zenodo record
   [13629114](https://doi.org/10.5281/zenodo.13629114) (Abdulkareem et al.
   2024, *Molecules*, CC-BY 4.0) contains `.pzf` files together with CSV
   exports of the same figures. Across four figures (one with a 3,351-row
   IR spectrum), **20,197 of 20,197 cells are equal**, including blank
   cells, row titles and Mean/SD subcolumns. Three of these files and their
   CSVs are in `samples/`; `python3 check_samples.py` reproduces 91/91.
2. **Value-level comparison with deposited spreadsheets.** Thirteen `.pzf`
   files from three other deposits have a same-named `.xlsx`/`.xls`
   ([18739340](https://doi.org/10.5281/zenodo.18739340),
   [19632378](https://doi.org/10.5281/zenodo.19632378),
   [3698092](https://doi.org/10.5281/zenodo.3698092)). I counted how many of
   each file's data values appear anywhere in its spreadsheet at float32
   precision. Four match 100% (Fig 1G, 2E, 7F, S2C), and two more match 99.7%
   and 97.9%. Of the other seven, I checked five by hand, and in each the
   difference comes from the data, not the decoding:
   * Some Prism tables contain the authors' **rounded** values (3.1398 in the
     spreadsheet, 3.14 in Prism; Fig 7B).
   * Some contain **normalised** values (the control set to 1 or
     0.99/1.01; Fig 1C, 7D, S2F).
   * One holds **Mean/SEM/N summaries** instead of raw values. For example,
     Male Control = 357.5, 17.177, 4, and recomputing from the raw weights in
     the spreadsheet (322, 339, 369, 400) gives mean 357.5 and SEM 17.18.
   * Fig 2C and 2G (62.5% and 81%) were not examined further.
3. **Robustness.** All **164/164** binary `.pzf` files I downloaded (at least
   one from each of 88 deposits, dated 2012–2026) parse completely, with
   every container balanced. They cover three header variants and five
   versions of the column record. Seven further "`.pzf`" files were XML
   `.pzfx` files that had been renamed.

## Format notes

All integers are little-endian.

**Container grammar.** `PCFFGRA4` (8 bytes) is followed by one record
stream. Each record is a u16 tag, a u32 length and the payload. A tag with
bit `0x8000` set opens a container. The records that follow are its
children, until a bare 2-byte end marker `tag | 0x4000` with no length. The
whole file is one root container, `0x8001`, with a 12-byte header. A few
containers (for example a trailing `0x8003`) are never closed and are closed
implicitly by their parent's end marker.

| Tag | Meaning |
|---|---|
| `0x8001` | Root. 12-byte header; bytes 4–5 vary (`02 00`, `02 01`, `00 11`) and probably encode the Prism version. |
| `0x0004` | Printer name, driver and paper size (`HP DeskJet 2130 series`, `winspool`, `Letter`). |
| `0x8006` / `0x0007` | Font table. |
| `0x8008` | A data table or "Project info" sheet. `0x09` gives its title. `0x0B` (560 bytes) has the replicate subcolumn count of the Y columns as u16 at offset 536. |
| `0x800F` | A column. Byte 32 of its 60-byte header gives the role: **0 = row titles, 4 = X, 8 = Y** (flag bits such as `0x20` and `0x30` can be added to Y). |
| `0x008F` | Column info. u32 version 2, 3, 5, 6 or 7, giving lengths 12, 24, 52, 64 or 80 bytes. From v5: u32[6] = kind (1 data, 2 text) and u32[7] = subcolumns. From v6: **u32[13] = Windows codepage** (1252, 1250, 936 or 950 seen). |
| `0x0009` | Title text, NUL-terminated. |
| `0x0010` | Cells: u16 encoding, then u32 count. Encoding 0 uses 12-byte cells: u32 flags, u32 display info, **float32**. Encoding 1 uses 16-byte cells: u32 flags, u32, **float64**. Flag bit 0 = value present. Values are stored row-major, with subcolumns interleaved. |
| `0x0011` + `0x00B6` | Text column: a string blob (u32 length, then bytes), and an index of (row, offset, length) entries. Rich-text runs follow a NUL inside each string. |
| `0x8012`, `0x8025`, … | Graphs, axes and layouts (not decoded). |

## Side findings relevant to archivists

1. **Precision depends on how the data was saved.** Most columns hold 32-bit
   floats, about 7 significant digits; 87 columns in the corpus hold 64-bit
   doubles. The precision of the deposited data therefore varies from file
   to file in ways a depositor would never see.
2. **Files embed their environment.** 43 of 86 deposited files contain the
   creator's printer name. **8 are UNC network paths** (`\\server\queue`),
   which expose internal print-server hostnames at the authors'
   institutions. Info sheets add experiment dates, in 47 files; the locale
   shows in formats such as `ene.-11-2022`.
3. **"Blank" cells are not empty.** A cleared cell keeps its old value with
   the present flag turned off. In `abdulkareem2024_Figure 5.pzf` the blank
   "Refrigerated" cells of three columns still hold the next row's values
   (38678 / 3482.113, …). This is the same leftover-buffer pattern as the
   DB/TextWorks and Statistica files in this repository.
4. **A `.pzf` extension is not proof of binary content.** 7 of 171
   downloaded "`.pzf`" files were XML `.pzfx` files renamed, so format
   identification has to look at the magic bytes.

## What is still unknown

* **Cell flag bits** other than bit 0. `0x800001` appears 1,016 times and is
  probably Prism's "excluded value" marking; `0x3`, `0x5` and `0x69` also
  occur. A float32 value of 3.4e38 inside a present cell is treated as
  missing.
* **Table format names.** The reader labels subcolumns `[1]`, `[2]`, … and
  does not say whether a table holds replicates, Mean/SD/N or Mean/SEM/N.
  That setting is somewhere in `0x0A`/`0x0B`.
* **Analysis results.** Result sheets (t tests, column statistics) are listed
  as text tables. Their own records are not decoded, and tag `0x10` means
  something else there.
* Graphs, layouts, the meaning of the root header bytes, and files from
  before 2012 (Prism 4).

## Usage

```
python3 pzf.py tables  FILE.pzf        # data tables, column roles, cell counts
python3 pzf.py csv     FILE.pzf [N]    # table N (default: all) as CSV
python3 pzf.py strings FILE.pzf        # printer, fonts, info-sheet values
python3 pzf.py tree    FILE.pzf        # raw record tree
python3 check_samples.py               # re-run the CSV comparison
```

Samples: `abdulkareem2024_*` are from Abdulkareem RA, Fotaki N, Koumanov F,
Dodson CA, Sartbaeva A (2024), doi:10.5281/zenodo.13629114, CC-BY 4.0.
`zenodo8408323_Figure_5i.pzf` is from Rojas G et al. (2023), Molecular
reshaping of phage-displayed Interleukin-2 …, doi:10.5061/dryad.kh18932c8
(Zenodo record 8408323), CC0.
