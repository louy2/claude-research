#!/usr/bin/env python3
"""Read Inmagic DB/TextWorks textbases (tested on 3.0 and 7.0.1) without the software.

Everything here was worked out from two small sample textbases by hex
inspection; nothing comes from vendor documentation. See README.md for the
format notes and for what is still unknown.

Usage:
    dbtw.py info    <textbase dir or any member file>
    dbtw.py csv     <textbase>            # records, full field names, to stdout
    dbtw.py verify  <textbase>            # cross-check records against the indexes
    dbtw.py residue <textbase>            # stale bytes left in record slack
"""
import csv
import glob
import os
import re
import struct
import sys

HEADER_RE = re.compile(rb"^([A-Z]{3}) (\d{3}) (\d\d/\d\d/\d\d)$")
SLOT = 0x40          # allocation unit seen in .DBR / .OCC
BTX_PAGE = 0x400     # page size of the .BTX index file


def u16(b, o):
    return struct.unpack_from("<H", b, o)[0]


def u32(b, o):
    return struct.unpack_from("<I", b, o)[0]


def i32(b, o):
    return struct.unpack_from("<i", b, o)[0]


def header(b):
    """Every binary member starts with 16 ASCII bytes: 'TAG rev mm/dd/yy'."""
    m = HEADER_RE.match(b[:16])
    if not m:
        return None
    return m.group(1).decode(), int(m.group(2)), m.group(3).decode()


class Textbase:
    def __init__(self, path):
        if os.path.isdir(path):
            tbas = glob.glob(os.path.join(path, "*.[tT][bB][aA]"))
            if not tbas:
                raise SystemExit(f"no .tba file in {path}")
            path = tbas[0]
        self.base = os.path.splitext(path)[0]
        self.files = {}
        for f in glob.glob(glob.escape(self.base) + ".*"):
            self.files[os.path.splitext(f)[1][1:].lower()] = open(f, "rb").read()
        self.fields = []
        self.parse_dbs()

    # ---------- .DBS: structure ----------
    def parse_dbs(self):
        b = self.files["dbs"]
        self.next_autonumber = u32(b, 0x410)
        self.record_count = u32(b, 0x414)
        self.dbs_tags = []
        o = 0x41D
        field_defs = []
        while o < len(b) - 16:
            tag = b[o]
            n = u16(b, o + 1)
            o += 3
            if tag == 0x32:            # container: n = number of field definitions
                self.dbs_tags.append((tag, n, b""))
                continue
            data = b[o:o + n]
            o += n
            self.dbs_tags.append((tag, n, data))
            if tag == 0x33:            # one field definition (18 bytes)
                field_defs.append({"type": u16(data, 4), "flags": data[8:10].hex(),
                                   "extra": None})
            elif tag == 0x46:          # optional extra block for the preceding field
                field_defs[-1]["extra"] = data.hex()
            elif tag == 0x35:          # NUL-separated field names, in field order
                names = data.split(b"\0")[:len(field_defs)]
                for fd, nm in zip(field_defs, names):
                    fd["name"] = nm.decode("cp1252")
            elif tag in (0x3E, 0x3F, 0x40, 0x41):
                setattr(self, {0x3E: "created_date", 0x3F: "created_time",
                               0x40: "modified_date", 0x41: "modified_time"}[tag],
                        data.split(b"\0")[0].decode())
            elif tag in (0x44, 0x45):  # word lists: u16 count, count x u32 heap pointer, strings
                cnt = u16(data, 0)
                words = data[2 + 4 * cnt:].split(b"\0")[:cnt]
                setattr(self, "stop_words" if tag == 0x44 else "leading_articles",
                        [w.decode("cp1252") for w in words])
            elif tag == 0x36:          # end of structure; file repeats its header after it
                break
        for i, fd in enumerate(field_defs, 1):
            fd["id"] = i
        self.fields = field_defs

    # ---------- .DBO + .DBR: records ----------
    def record_offsets(self):
        b = self.files["dbo"]
        out = []
        for recno, o in enumerate(range(0x10, len(b), 6)):
            status, off = u16(b, o), u32(b, o + 2)
            if recno == 0:
                continue                 # entry 0 is unused
            out.append((recno, status, off))
        return out

    def read_slot(self, off):
        b = self.files["dbr"]
        assert u32(b, off) == off, f"slot at {off:#x} does not point to itself"
        kind = b[off + 0x0E:off + 0x10].hex()
        recno = u32(b, off + 0x10)
        plen = u16(b, off + 0x16)
        p, end = off + 0x18, off + 0x18 + plen
        values = {}
        while p < end:
            fid, ln = u16(b, p), u16(b, p + 2)
            values.setdefault(fid, []).append(b[p + 4:p + 4 + ln].decode("cp1252"))
            p += 4 + ln
        return {"offset": off, "kind": kind, "recno": recno,
                "unk14": u16(b, off + 0x14), "payload_len": plen,
                "payload_end": end, "values": values}

    def records(self):
        for recno, status, off in self.record_offsets():
            r = self.read_slot(off)
            r["dbo_status"] = status
            yield r

    # ---------- .BTX + .OCC: indexes ----------
    def btx_page(self, pageno):
        b = self.files["btx"]
        base = 0x1000 + (pageno - 1) * BTX_PAGE
        pid, kind, cnt = u32(b, base), b[base + 4], u16(b, base + 5)
        entries = []
        for i in range(cnt):
            e = base + 7 + 10 * i
            soff, n, val = u16(b, e), u32(b, e + 2), i32(b, e + 6)
            s = b[base + soff:b.index(b"\0", base + soff)].decode("cp1252")
            entries.append((s, n, val))
        return pid, kind, entries

    def index_roots(self):
        b = self.files["btx"]
        term = {f["id"]: u32(b, 0x400 + 4 * f["id"]) for f in self.fields}
        word = {f["id"]: u32(b, 0x800 + 4 * f["id"]) for f in self.fields}
        return ({k: v for k, v in term.items() if v}, {k: v for k, v in word.items() if v})

    def occ_recnos(self, off):
        b = self.files["occ"]
        assert u32(b, off) == off
        count = u32(b, off + 0x13)
        return [u32(b, off + 0x17)] if count == 1 else None, count


def cmd_info(tb):
    print(f"textbase: {tb.base}")
    for ext, data in sorted(tb.files.items()):
        h = header(data)
        print(f"  .{ext:<4} {len(data):6d} bytes  {'%s rev %03d dated %s' % h if h else '(text)'}")
    print(f"created {tb.created_date} {tb.created_time}; "
          f"modified {tb.modified_date} {tb.modified_time}")
    print(f"records: {tb.record_count}; next automatic number: {tb.next_autonumber}")
    for f in tb.fields:
        print(f"  field {f['id']}: {f['name']!r} type={f['type']:#04x} flags={f['flags']}"
              + (f" extra={f['extra']}" if f["extra"] else ""))
    print("stop words:", getattr(tb, "stop_words", []))
    print("leading articles:", getattr(tb, "leading_articles", []))


def cmd_csv(tb):
    w = csv.writer(sys.stdout)
    w.writerow([f["name"] for f in tb.fields])
    for r in tb.records():
        w.writerow(["|".join(r["values"].get(f["id"], [])) for f in tb.fields])


def cmd_verify(tb):
    recs = {r["recno"]: r for r in tb.records()}
    ok = bad = 0

    def check(cond, msg):
        nonlocal ok, bad
        if cond:
            ok += 1
        else:
            bad += 1
            print("MISMATCH:", msg)

    check(len(recs) == tb.record_count, f"DBS says {tb.record_count} records, DBO has {len(recs)}")
    check(max(recs) + 1 == tb.next_autonumber, "next automatic number != last recno + 1")
    term_roots, word_roots = tb.index_roots()
    for fid, page in term_roots.items():
        _, kind, entries = tb.btx_page(page)
        for term, n, val in entries:
            vals = recs[val]["values"].get(fid, []) if val in recs else []
            if term.startswith("\x1d"):          # numeric term: 0x1D, 'A'+digits-1, digits
                m = re.fullmatch("\x1d([A-Z])(\\d+)", term)
                check(m and ord(m[1]) - 64 == len(m[2]) and m[2] in vals,
                      f"field {fid} numeric term {term!r} -> rec {val}: {vals}")
            else:
                check(term.lower() in (v.lower() for v in vals),
                      f"field {fid} term {term!r} -> rec {val}: {vals}")
    for fid, page in word_roots.items():
        _, kind, entries = tb.btx_page(page)
        for word, n, val in entries:
            got, cnt = tb.occ_recnos(-val)
            check(cnt == n and got is not None, f"word {word!r}: OCC count {cnt} vs {n}")
            for rn in got or []:
                text = " ".join(recs[rn]["values"].get(fid, [])).lower()
                check(word in re.findall(r"\w+", text), f"word {word!r} -> rec {rn}: {text!r}")
    print(f"{ok} checks passed, {bad} failed")
    return bad == 0


def cmd_residue(tb):
    b = tb.files["dbr"]
    for r in tb.records():
        slot_end = r["offset"] + SLOT
        slack = b[r["payload_end"]:slot_end]
        junk = slack.rstrip(b"\0")
        if junk:
            print(f"rec {r['recno']}: live={r['values']}  slack after payload: {junk!r}")
    occ = tb.files.get("occ", b"")
    for m in re.finditer(rb"[\x20-\x7e]{4,}", occ[16:]):
        print(f".occ @{m.start() + 16:#x}: {m.group().decode()!r}")


if __name__ == "__main__":
    if len(sys.argv) != 3 or sys.argv[1] not in ("info", "csv", "verify", "residue"):
        raise SystemExit(__doc__)
    tb = Textbase(sys.argv[2])
    r = {"info": cmd_info, "csv": cmd_csv, "verify": cmd_verify,
         "residue": cmd_residue}[sys.argv[1]](tb)
    sys.exit(0 if r in (None, True) else 1)
