# Statistica / CSS data files (`.STA`): reading them without Statistica

## Why this format

STATISTICA (StatSoft, first released 1991 as a successor to StatSoft's DOS
program CSS, the Complete Statistical System) was a widely used statistics
package in the natural and social sciences during the 1990s and 2000s. Its
data files ("spreadsheets") are binary, have no public specification, and
open tools cannot read them:

* In 2017 a user asked the R `haven` package to support `.sta`
  ([tidyverse/haven#264](https://github.com/tidyverse/haven/issues/264)).
  The only R package that had read them was off CRAN "due to licensing
  issues", so they "had to fall back to obtaining a copy of Statistica,
  opening the files in that and exporting to something more useful". The
  issue is still open.
* Johan van der Knijff (KB, National Library of the Netherlands) added
  `.STA`, `.STG` and `.SCR` files to the
  [OPF format corpus](https://github.com/openpreserve/format-corpus/tree/master/statistica)
  with the note "exact version unknown (might be 4 or 5)". No reader was
  provided.
* Researchers keep depositing these files as research data. A Zenodo search
  for the `sta` file type in October 2026 found 30 `.sta` files from 2013–2026.
  Most are in a newer layout (see below), but `.sta` remains a common
  supplementary-data format.

## Result

[`sta.py`](sta.py) is a single Python 3 file with no dependencies. It reads
the older `CSS ` layout. It has been tested on three files from two
independent sources:

| File | Source | Content | Check |
|---|---|---|---|
| `PEYNEVL2.STA` | OPF corpus (CC0) | 230 field sites × 25 variables: Lambert coordinates, slope, aspect, soil, texture, land use, bulk density, moisture | The file stores a formula, `NEWVAR = sin((v6/360)*2*Pi)`. Recomputing it from ASP matches all 230 stored values, and every coded value has a text label. 914/914 checks pass. |
| `KSBASE.STA` | OPF corpus (CC0) | 84 hydraulic-conductivity measurements | All coded values have text labels. 167/167 checks pass. |
| `oo_31598.STA` | [Zenodo 902054](https://doi.org/10.5281/zenodo.902054) (CC-BY 4.0) | 525 rows of shell measurements from Todorov (2015), *Biodiversity Data Journal* 3: e4297, written with STATISTICA 7.0 | Summary statistics computed from the decoded data **match Table 1 of the published paper** (see below). |

Comparison with the paper (my value / published value):

| Character | n | mean | median | SD | min–max |
|---|---|---|---|---|---|
| Diameter of shell | 75 / 75 | 175.79 / 175.8 | 175 / 175.0 | 7.30 / 7.30 | 159–190 / 159–190 |
| Depth of shell | 43 / 43 | 128.07 / 128.1 | 129 / 129.0 | 4.60 / 4.60 | 118–138 / 118–138 |
| Base of apertural tube | 75 / 75 | 59.85 / 59.8 | 60 / 60.0 | 3.195 / 3.19 | 54–67 / 54–67 |
| Depth of apertural tube | 43 / 43 | 50.02 / 50.0 | 50 / 50.0 | 1.697 / 1.69 | 47–55 / 47–55 |
| Large axis, internal opening | 69 / 69 | 37.06 / 37.1 | 38 / 38.0 | 3.41 / 3.41 | 27–43 / 27–43 |
| Small axis, internal opening | 69 / 69 | 29.23 / 29.2 | 30 / 30.0 | 2.67 / 2.67 | 23–36 / 23–36 |
| Length of teeth | 75 / 75 | 16.25 / 16.2 | 16 / 16.0 | 1.97 / 1.97 | 12–23 / 12–23 |

The counts, medians and ranges are identical. The remaining differences are
all in the last digit and in the same direction (59.853 → 59.8, 1.697 → 1.69),
which fits the paper truncating its figures rather than rounding them.

```
$ python3 sta.py csv samples/PEYNEVL2.STA | head -3
X,Y,NO,DATE,SLOP,ASP,NEWVAR,SOIL,TEXT,STON,LANDUS,TRCOV,...
683330,3135630,1,35219,0.5,240,-0.866025403791243,cl,l,2,wijn,0,...
682880,3135460,2,35219,2.5,320,-0.642787609672639,cl,sl,4,wijn,0,...
```

By default, text labels replace their numeric codes; `--codes` keeps the
codes. Missing values (the per-variable MD code, -9999 in every sample) are
written as empty cells. Numbers are written at full double precision.

## Format notes (`CSS ` layout)

All values are little-endian.

| Offset | Type | Meaning |
|---|---|---|
| `0x00` | 4 bytes | Magic `CSS ` |
| `0x04` | i32 | Negative signature that varies by file: -9869, -9877, -9858. Probably a version or build number. |
| `0x08` | u32 | Number of cases (rows) |
| `0x0C` | u32 | Number of variables (columns) |
| `0x10` | u32 | 8 in all samples (perhaps the case-name width) |
| `0x14` | u32 | Number of u16 section sizes that follow: **7** in the corpus files, **8** in the STATISTICA 7.0 file |
| `0x18` | u16 × n | Section sizes, in order: [20, 80, names 8·v, formats 2·v, missing-data codes 8·v, label counts 2·v, label offsets 4·v, (long-label offsets)] |
| next | u32 × 5 | Lengths of the variable-size areas: formulas, text labels, long labels, unknown (always 0), display block |
| next | 80 bytes | File header or title (empty in all samples) |
| | 8·v | Variable names, space-padded and right-aligned |
| | 2·v | Display format per variable: (width, decimals), e.g. `08 03` |
| | 8·v | Missing-data code per variable, as an f64 |
| | 2·v | Number of text labels per variable |
| | 4·v | Offset of each variable's labels within the label area |
| | (8th section) | Offsets into the long-label area |
| | formulas | Repeated (u16 1-based variable number, u8 length, text), e.g. `=sin((v6/360)*2*Pi)` |
| | text labels | 14 bytes each: f32 numeric code, 8-byte text, u16 offset into the long-label area (`0xFFFF` = none) |
| | long labels | f32 code, u8 length, text (e.g. "Diameter of shell (in µm)") |
| | display block | Its own length first, then font name (`Courier`), RGB colours and the **recent-files list** |
| padding | | The header is padded to a multiple of 1 KB |
| end − 8·cases·v | f64 | The data: **row-major** doubles, at the end of the file |

Text values in cells are stored as numeric codes. Statistica assigns codes
starting at 100, and the label table maps each code back to its text.

## Side findings relevant to archivists

1. **The files leak the creator's recent-files list.** The display block in
   `oo_31598.STA` contains 30+ paths from the author's PC, such as
   `d:\stat\madaga~1\lam_call.dbf`, `c:\stat\lam_call.xls` and
   `d:\articl~1\lamtop~1\box_a.wmf`. They show the data came from a dBase
   file and an Excel sheet, and they name the article folders the figures were
   exported to. That is useful provenance, but also personal information that
   depositors probably do not know they are publishing.
2. **Header padding contains stale data.** `KSBASE.STA` has 74 bytes of an
   earlier copy of its label table in the padding before the data. As with
   the DB/TextWorks files in this repository, the program wrote out a reused
   buffer without clearing it.
3. **Imported header rows became data.** In `KSBASE.STA` the variables are
   still called `A`…`N`. The first two cases are the spreadsheet's header row
   (`OBSERV, DATE, X, …`) and unit row (`(cm)`, `(m/day)`), stored as text
   codes 100 and 101. A naive export would treat them as observations.
4. **Dates are day numbers.** `DATE` = 35219 in `PEYNEVL2.STA` reads as
   1996-06-04 under the 1899-12-30 epoch used by spreadsheets of the period.
   That fits summer fieldwork, but the epoch is not confirmed for Statistica.
   The Dutch land-use labels (`wijn`, `garr`, `maqu`: vineyard, garrigue,
   maquis) and Lambert coordinates suggest a Dutch fieldwork campaign in
   southern France.

## What is still unknown

* **The newer layout.** Of 29 `.sta` files from Zenodo, only one uses `CSS `.
  The rest start with `80 06 00 01` or `80 06 00 00` and come from datasets
  published 2013–2026. That layout is different and is not handled here; it
  is the larger open problem, because researchers still deposit it.
* Case names, the 80-byte title, the meaning of the `0x04` signature and of
  the fourth u32 length, and formulas other than the one sample.
* Graph files (`.STG`) and results "scrollsheets" (`.SCR`). Both are in the
  corpus but have not been decoded.

## Usage

```
python3 sta.py info   FILE.STA            # variables, formats, labels, formulas, embedded strings
python3 sta.py csv    FILE.STA [--codes]  # data as CSV
python3 sta.py verify FILE.STA            # recompute formula columns, check text-label coverage
```

Sample licences: the corpus files are CC0 (Johan van der Knijff, see
`samples/README-opf-corpus.md`). `oo_31598.STA` is CC-BY 4.0, from Todorov M
(2015) *On the morphology, biometry and biogeography of Lamtopyxis callistoma
(Amoebozoa: Arcellinida)*, Biodiversity Data Journal 3: e4297,
doi:10.3897/BDJ.3.e4297, supplementary material 1.
