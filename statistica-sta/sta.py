#!/usr/bin/env python3
"""Read StatSoft Statistica / CSS data spreadsheets (.STA, "CSS " magic) without Statistica.

Worked out by hex inspection of two files from the OPF format corpus
(Statistica for Windows, probably version 4 or 5). See README.md for the
layout and for what is still unknown.

Usage:
    sta.py info   FILE.STA
    sta.py csv    FILE.STA [--codes]   # text labels replace codes unless --codes
    sta.py verify FILE.STA             # recompute formula variables, check labels
"""
import csv
import datetime
import math
import re
import struct
import sys

MAGIC = b"CSS "


class Sta:
    def __init__(self, path):
        b = self.b = open(path, "rb").read()
        if b[:4] != MAGIC:
            raise SystemExit(f"{path}: not a CSS/Statistica data file")
        self.signature = struct.unpack_from("<i", b, 4)[0]
        self.ncases, self.nvars = struct.unpack_from("<II", b, 8)
        # 0x14: number of u16 section sizes that follow at 0x18 (7 or 8 seen)
        self.nsections = struct.unpack_from("<I", b, 0x14)[0]
        secs = struct.unpack_from(f"<{self.nsections}H", b, 0x18)
        o = 0x18 + 2 * self.nsections
        # first section: block of u32 lengths for the variable-size areas
        (self.formula_len, self.labels_len, self.longlabels_len, self.unk_len,
         self.display_len) = struct.unpack_from("<5I", b, o)
        o += secs[0]
        title_len, names_len, fmt_len, md_len, nlab_len, laboff_len = secs[1:7]
        v = self.nvars
        assert (names_len, fmt_len, md_len, nlab_len, laboff_len) == (8 * v, 2 * v, 8 * v, 2 * v, 4 * v)

        self.title = b[o:o + title_len].split(b"\0")[0].decode("cp1252").strip()
        o += title_len
        self.names = [b[o + 8 * i:o + 8 * i + 8].decode("cp1252").strip() for i in range(v)]
        o += names_len
        self.formats = [(b[o + 2 * i], b[o + 2 * i + 1]) for i in range(v)]   # display hint
        o += fmt_len
        self.missing = list(struct.unpack_from(f"<{v}d", b, o))
        o += md_len
        nlabels = struct.unpack_from(f"<{v}H", b, o)
        o += nlab_len
        laboffs = struct.unpack_from(f"<{v}I", b, o)
        o += laboff_len
        o += sum(secs[7:])               # 8th section: offsets into the long-label block (unused here)

        # formula block: repeated (u16 1-based var, u8 len, text)
        self.formulas = {}
        end = o + self.formula_len
        while o < end:
            var, n = struct.unpack_from("<HB", b, o)
            self.formulas[var] = b[o + 3:o + 3 + n].decode("cp1252")
            o += 3 + n

        # text labels: per variable, nlabels x (f32 code, 8-char text, u16 offset into
        # the long-label block or 0xFFFF). Long labels: f32 code, u8 len, text.
        lab_base = o
        long_base = lab_base + self.labels_len
        self.labels, self.long_labels = [], []
        for i in range(v):
            lab, longlab = {}, {}
            p = lab_base + laboffs[i]
            for _ in range(nlabels[i]):
                code = struct.unpack_from("<f", b, p)[0]
                lab[code] = b[p + 4:p + 12].split(b"\0")[0].decode("cp1252").strip()
                lo = struct.unpack_from("<H", b, p + 12)[0]
                if lo != 0xFFFF and self.longlabels_len:
                    q = long_base + lo
                    n = b[q + 4]
                    longlab[code] = b[q + 5:q + 5 + n].decode("cp1252")
                p += 14
            self.labels.append(lab)
            self.long_labels.append(longlab)
        self.labels_end = long_base + self.longlabels_len
        self.display = b[self.labels_end:self.labels_end + self.display_len]

        self.data_offset = len(b) - 8 * self.ncases * v
        assert self.data_offset >= self.labels_end + self.display_len, "data overlaps header"
        flat = struct.unpack_from(f"<{self.ncases * v}d", b, self.data_offset)
        self.rows = [list(flat[r * v:(r + 1) * v]) for r in range(self.ncases)]

    def cell(self, r, c, codes=False):
        x = self.rows[r][c]
        if x == self.missing[c]:
            return ""
        if not codes and x in self.labels[c]:
            return self.labels[c][x]
        return format(x, ".15g")      # full precision; the display format is not applied


def statistica_date(serial):
    """Assumes the 1899-12-30 epoch shared with spreadsheets of the period (unverified)."""
    return datetime.date(1899, 12, 30) + datetime.timedelta(days=int(serial))


def cmd_info(s):
    print(f"signature {s.signature}  sections {s.nsections}  cases {s.ncases}  variables {s.nvars}  "
          f"data at {s.data_offset:#x}  title {s.title!r}")
    for i, n in enumerate(s.names):
        w, d = s.formats[i]
        extra = []
        if s.labels[i]:
            extra.append("labels " + ", ".join(
                f"{int(k)}={t}" + (f" ({s.long_labels[i][k]})" if k in s.long_labels[i] else "")
                for k, t in s.labels[i].items()))
        if i + 1 in s.formulas:
            extra.append("formula " + s.formulas[i + 1])
        print(f"  v{i + 1:<3}{n:<9} fmt {w}.{d}  MD {s.missing[i]:g}  " + "; ".join(extra))
    strs = re.findall(rb"[ -~]{6,}", s.display)
    if strs:
        print("strings in display block:", [x.decode() for x in strs])
    slack = s.b[s.labels_end + s.display_len:s.data_offset].rstrip(b"\0")
    if slack:
        print(f"non-zero padding before data: {len(slack)} bytes, e.g. "
              f"{re.findall(rb'[ -~]{4,}', slack)[:4]}")


def cmd_csv(s, codes):
    w = csv.writer(sys.stdout)
    w.writerow(s.names)
    for r in range(s.ncases):
        w.writerow([s.cell(r, c, codes) for c in range(s.nvars)])


def cmd_verify(s):
    ok = bad = 0
    for var, f in s.formulas.items():
        m = re.fullmatch(r"=sin\(\(v(\d+)/360\)\*2\*Pi\)", f)
        if not m:
            print(f"formula v{var} {f!r}: not evaluated (only the sample's form is supported)")
            continue
        src = int(m[1]) - 1
        for r, row in enumerate(s.rows):
            x, y = row[src], row[var - 1]
            if x == s.missing[src]:
                good = y == s.missing[var - 1]
            else:
                good = math.isclose(y, math.sin(x / 360 * 2 * math.pi), abs_tol=1e-9)
            ok += good
            bad += not good
        print(f"formula v{var} = {f}: recomputed over {s.ncases} cases")
    # a variable whose values are mostly label codes should use only label codes
    for c in range(s.nvars):
        vals = [row[c] for row in s.rows if row[c] != s.missing[c]]
        if not s.labels[c] or sum(x in s.labels[c] for x in vals) * 2 <= len(vals):
            continue
        for x in vals:
            good = x in s.labels[c]
            ok += good
            bad += not good
            if not good:
                print(f"v{c + 1} {s.names[c]}: value {x:g} has no text label")
    print(f"{ok} checks passed, {bad} failed")
    return bad == 0


if __name__ == "__main__":
    import signal
    signal.signal(signal.SIGPIPE, signal.SIG_DFL)
    if len(sys.argv) < 3 or sys.argv[1] not in ("info", "csv", "verify"):
        raise SystemExit(__doc__)
    s = Sta(sys.argv[2])
    if sys.argv[1] == "info":
        cmd_info(s)
    elif sys.argv[1] == "csv":
        cmd_csv(s, "--codes" in sys.argv)
    else:
        sys.exit(0 if cmd_verify(s) else 1)
