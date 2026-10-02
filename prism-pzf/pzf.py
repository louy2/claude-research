#!/usr/bin/env python3
"""Read GraphPad Prism binary project files (.pzf, magic "PCFFGRA4") without Prism.

Worked out by hex inspection of research-data deposits on Zenodo. See README.md
for the record grammar, how it was validated, and what is still unknown.

Usage:
    pzf.py tree   FILE.pzf             # dump the record tree
    pzf.py tables FILE.pzf             # list data tables and columns
    pzf.py csv    FILE.pzf [N]         # data table N (default: all) as CSV
    pzf.py strings FILE.pzf            # printer, fonts, info-sheet values
"""
import csv
import struct
import sys

MAGIC = b"PCFFGRA4"
FIRST_RECORD = 0x08       # root container 0x8001 (12-byte header) wraps everything

TAG_TABLE = 0x8008       # data table or info sheet
TAG_COLUMN = 0x800F
TAG_TITLE = 0x0009
TAG_CELLS = 0x0010
TAG_TEXTBLOB = 0x0011
TAG_COLINFO = 0x008F
TAG_TABLEINFO = 0x000B   # 560 bytes; u16 at +536 = replicate subcolumns of Y columns
TAG_TEXTINDEX = 0x00B6
TAG_PRINTER = 0x0004
TAG_FONT = 0x0007


def records(b, o=FIRST_RECORD):
    """Yield (tag, payload, offset, depth). Containers (tag & 0x8000) hold the
    records that follow until a bare 2-byte end marker (tag | 0x4000)."""
    stack = []
    while o < len(b):
        tag = struct.unpack_from("<H", b, o)[0]
        if tag & 0x4000:
            if not any((t & 0x3FFF) == (tag & 0x3FFF) for t in stack):
                raise ValueError(f"unbalanced end marker {tag:#x} at {o:#x}")
            # some containers (e.g. a trailing 0x8003) are never closed explicitly;
            # an outer end marker closes them too
            while (stack.pop() & 0x3FFF) != (tag & 0x3FFF):
                pass
            o += 2
            continue
        n = struct.unpack_from("<I", b, o + 2)[0]
        if o + 6 + n > len(b):
            raise ValueError(f"record {tag:#x} at {o:#x} overruns file")
        yield tag, b[o + 6:o + 6 + n], o, len(stack) - 1     # depth 0 = child of root
        o += 6 + n
        if tag & 0x8000:
            stack.append(tag)
    if stack:
        raise ValueError(f"unclosed containers {[hex(t) for t in stack]}")


def text(p):
    return p.split(b"\0")[0].decode("cp1252", "replace")


ROLE_ROWTITLE, ROLE_X, ROLE_Y = 0, 4, 8      # byte 32 of the 0x800F column header


class Column:
    def __init__(self, header, table):
        self.id = struct.unpack_from("<I", header, 0)[0]
        self.role = header[32]                       # 0 row titles, 4 X, 8 Y (+ flag bits)
        self.table = table
        self.codepage = 1252
        self.title = ""
        self.cells = []          # (present, float32 value, raw 12 bytes)
        self.texts = {}          # row -> text, for text (row-title / info) columns
        self.subcolumns = 1
        self.encoding = 0

    def set_colinfo(self, p):
        """0x8F: u32 version (2,3,5,6,7 seen). From v5: u32[6] kind (1 data, 2 text),
        u32[7] subcolumns per row; from v6: u32[13] Windows codepage of the text."""
        ver = struct.unpack_from("<I", p, 0)[0]
        if ver >= 5 and len(p) >= 32:
            self.subcolumns = max(1, struct.unpack_from("<I", p, 28)[0])
        elif self.role & ROLE_Y:
            self.subcolumns = max(1, self.table.get("replicates", 1))
        if ver >= 6 and len(p) >= 56:
            cp = struct.unpack_from("<I", p, 52)[0]
            if cp:
                self.codepage = cp

    def set_cells(self, p):
        """0x10: u16 encoding, u32 count, then cells. Encoding 0: 12-byte cells
        (u32 flags, u32 display info, float32). Encoding 1: 16-byte cells
        (u32 flags, u32, float64). Flag bit 0 = value present."""
        enc, n = struct.unpack_from("<HI", p, 0)
        size, fmt = (12, "<f") if enc == 0 else (16, "<d")
        self.encoding = enc
        for i in range(n):
            q = 6 + size * i
            if q + size > len(p):
                break
            flags = struct.unpack_from("<I", p, q)[0]
            value = struct.unpack_from(fmt, p, q + 8)[0]
            self.cells.append((flags, value, p[q:q + size]))

    def set_texts(self, blob, index):
        cnt = struct.unpack_from("<I", index, 0)[0]
        data = blob[4:]
        for i in range(cnt):
            row, off, n = struct.unpack_from("<III", index, 4 + 12 * i)
            self.texts[row] = data[off:off + n].split(b"\0")[0].decode(
                f"cp{self.codepage}", "replace")

    @property
    def is_text(self):
        return bool(self.texts)

    def value(self, row, sub=0):
        if self.is_text:
            return self.texts.get(row, "")
        i = row * self.subcolumns + sub
        if i >= len(self.cells) or not self.cells[i][0] & 1:
            return ""
        flags, v, _ = self.cells[i]
        if abs(v) >= 3.4e38:                         # FLT_MAX sentinel seen in some cells
            return ""
        return format(v, ".7g" if self.encoding == 0 else ".15g")

    @property
    def nrows(self):
        if self.is_text:
            return max(self.texts, default=-1) + 1
        return -(-len(self.cells) // self.subcolumns)


class Pzf:
    def __init__(self, path):
        b = self.b = open(path, "rb").read()
        if b[:8] != MAGIC:
            raise SystemExit(f"{path}: not a Prism .pzf file (magic {b[:8]!r})")
        self.tables, self.fonts, self.printer = [], [], None
        table = col = blob = None
        for tag, p, o, depth in records(b):
            if tag == TAG_TABLE:
                table = {"title": None, "columns": [], "offset": o}
                self.tables.append(table)
                col = None
            elif tag == TAG_TABLEINFO and table is not None and "replicates" not in table \
                    and len(p) >= 538:
                table["replicates"] = struct.unpack_from("<H", p, 536)[0]
            elif tag == TAG_COLUMN and table is not None:
                col = Column(p, table)
                table["columns"].append(col)
                blob = None
            elif tag == TAG_TITLE:
                if col is not None and depth >= 2:
                    col.title = p.split(b"\0")[0].decode(f"cp{col.codepage}", "replace")
                elif table is not None and table["title"] is None:
                    table["title"] = text(p)
            elif tag == TAG_CELLS and col is not None:
                col.set_cells(p)
            elif tag == TAG_TEXTBLOB and col is not None:
                blob = p
            elif tag == TAG_TEXTINDEX and col is not None and blob is not None:
                col.set_texts(blob, p)
            elif tag == TAG_COLINFO and col is not None:
                col.set_colinfo(p)
            elif tag == TAG_PRINTER:
                self.printer = text(p)
            elif tag == TAG_FONT:
                self.fonts.append(text(p[6:]))
            elif tag & 0x8000 and tag not in (TAG_TABLE, TAG_COLUMN) and depth == 0:
                table = col = None          # left the data-table section

    def table_rows(self, t):
        cols = [c for c in t["columns"] if c.cells or c.texts]
        header = []
        for c in cols:
            header += [c.title] if c.subcolumns == 1 else \
                      [f"{c.title} [{k + 1}]" for k in range(c.subcolumns)]
        nrows = max((c.nrows for c in cols), default=0)
        rows = []
        for r in range(nrows):
            row = []
            for c in cols:
                row += [c.value(r, k) for k in range(c.subcolumns)]
            rows.append(row)
        return header, rows


def main():
    if len(sys.argv) < 3 or sys.argv[1] not in ("tree", "tables", "csv", "strings"):
        raise SystemExit(__doc__)
    cmd, path = sys.argv[1], sys.argv[2]
    if cmd == "tree":
        b = open(path, "rb").read()
        for tag, p, o, d in records(b):
            extra = repr(text(p)) if tag == TAG_TITLE else p[:20].hex(" ")
            print(f"{o:#08x} {'  ' * d}{tag:#06x} len {len(p):<6} {extra}")
        return
    f = Pzf(path)
    if cmd == "tables":
        for i, t in enumerate(f.tables):
            print(f"[{i}] {t['title']!r}")
            for c in t["columns"]:
                kind = "text" if c.is_text else f"{len(c.cells)} cells x{c.subcolumns}"
                role = {0: "rows", 4: "X"}.get(c.role, "Y" if c.role & 8 else f"?{c.role}")
                print(f"     col {c.id:<3} {role:4} {c.title!r:40} {kind}")
    elif cmd == "csv":
        which = [int(sys.argv[3])] if len(sys.argv) > 3 else range(len(f.tables))
        w = csv.writer(sys.stdout)
        for i in which:
            header, rows = f.table_rows(f.tables[i])
            if len(which) > 1:
                w.writerow([f"# table {i}: {f.tables[i]['title']}"])
            w.writerow(header)
            w.writerows(rows)
    elif cmd == "strings":
        print("printer:", f.printer)
        print("fonts:", sorted(set(f.fonts)))
        for t in f.tables:
            if t["title"] and t["title"].startswith("Project info"):
                header, rows = f.table_rows(t)
                for r in rows:
                    print("  ", " = ".join(x for x in r if x))


if __name__ == "__main__":
    import signal
    signal.signal(signal.SIGPIPE, signal.SIG_DFL)
    main()
