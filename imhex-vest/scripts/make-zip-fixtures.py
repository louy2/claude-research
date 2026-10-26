#!/usr/bin/env python3
"""Writes the ZIP fixtures used by interval-vest/tests/zip_profile.rs.

    scripts/make-zip-fixtures.py [OUT_DIR]   (default: interval-vest/tests/data)

ok-*.zip are expected to be accepted by the strict profile, bad-*.zip to be
rejected. The output is reproducible except for the archive Info-ZIP writes
from stdin, which carries the current time, and for differences between
Info-ZIP versions. Archives from Python's zipfile and from Info-ZIP's `zip` (if
installed) are real-world inputs; the rest come from the small writer below,
which lays out records explicitly so that one property at a time can be
broken.
"""
import io
import os
import shutil
import struct
import subprocess
import sys
import tempfile
import zipfile
import zlib

OUT = sys.argv[1] if len(sys.argv) > 1 else os.path.join(
    os.path.dirname(__file__), "..", "interval-vest", "tests", "data")

FILES = [
    ("hello.txt", b"Hello, interval parsing!\n"),
    ("dir/numbers.bin", bytes(range(256)) * 4),
    ("empty", b""),
]


def local_header(name, info, extra=b""):
    return struct.pack("<4sHHHHHIIIHH", b"PK\x03\x04", info["version"], info["flags"],
                       info["method"], 0, 0x21, info["crc"], len(info["data"]),
                       info["usize"], len(name), len(extra)) + name + extra


def cd_entry(name, info, offset, extra=b"", comment=b""):
    return struct.pack("<4sHHHHHHIIIHHHHHII", b"PK\x01\x02", 0x031e, info["version"],
                       info["flags"], info["method"], 0, 0x21, info["crc"], len(info["data"]),
                       info["usize"], len(name), len(extra), len(comment), 0, 0, 0o644 << 16,
                       offset) + name + extra + comment


def eocd(count, cd_size, cd_offset, comment=b"", total=None):
    total = count if total is None else total
    return struct.pack("<4sHHHHIIH", b"PK\x05\x06", 0, 0, count, total, cd_size, cd_offset,
                       len(comment)) + comment


def entry(name, data, deflate=False, flags=0):
    packed = data
    if deflate:
        c = zlib.compressobj(9, zlib.DEFLATED, -15)
        packed = c.compress(data) + c.flush()
    return name.encode(), {
        "version": 20 if deflate else 10,
        "flags": flags,
        "method": 8 if deflate else 0,
        "crc": zlib.crc32(data),
        "usize": len(data),
        "data": packed,
    }


def build(entries, comment=b"", gap_after=None, order=None, cd_names=None, total=None,
          cd_gap=0, offsets=None):
    """Writes local files, then the central directory, then the end record.

    gap_after: index of a local file followed by 3 padding bytes.
    order: order of the local files in the file (default: central order).
    cd_names: names to put in the central directory instead.
    offsets: override the local header offsets in the central directory.
    """
    out = io.BytesIO()
    pos = {}
    for i in (order or range(len(entries))):
        name, info = entries[i]
        pos[i] = out.tell()
        out.write(local_header(name, info) + info["data"])
        if gap_after == i:
            out.write(b"\0\0\0")
    out.write(b"\0" * cd_gap)
    cd_start = out.tell()
    for i, (name, info) in enumerate(entries):
        cd_name = cd_names[i] if cd_names else name
        out.write(cd_entry(cd_name, info, offsets[i] if offsets else pos[i]))
    cd_size = out.tell() - cd_start
    out.write(eocd(len(entries), cd_size, cd_start, comment, total))
    return out.getvalue()


def write(name, data):
    with open(os.path.join(OUT, name), "wb") as f:
        f.write(data)


def main():
    os.makedirs(OUT, exist_ok=True)
    canon = [entry("hello.txt", FILES[0][1]), entry("dir/numbers.bin", FILES[1][1], True),
             entry("empty", b"")]

    # Real-world writers.
    for method, tag in [(zipfile.ZIP_STORED, "stored"), (zipfile.ZIP_DEFLATED, "deflated")]:
        buf = io.BytesIO()
        with zipfile.ZipFile(buf, "w", method) as z:
            for n, d in FILES:
                z.writestr(zipfile.ZipInfo(n, date_time=(2023, 6, 17, 12, 0, 0)), d, method)
            z.comment = b"written by Python zipfile"
        write(f"ok-python-{tag}.zip", buf.getvalue())
    buf = io.BytesIO()
    zipfile.ZipFile(buf, "w").close()
    write("ok-python-empty.zip", buf.getvalue())
    if shutil.which("zip"):
        with tempfile.TemporaryDirectory() as d:
            for n, data in FILES:
                p = os.path.join(d, "src", n)
                os.makedirs(os.path.dirname(p), exist_ok=True)
                with open(p, "wb") as f:
                    f.write(data)
            for dirpath, _, names in os.walk(os.path.join(d, "src")):
                for n in names + [""]:
                    os.utime(os.path.join(dirpath, n), (1686999600, 1686999600))
            out = os.path.join(d, "a.zip")
            subprocess.run(["zip", "-q", "-r", "-D", out, "."], cwd=os.path.join(d, "src"),
                           check=True)
            with open(out, "rb") as f:
                write("ok-infozip.zip", f.read())
            # Streamed from stdin: Info-ZIP writes a data descriptor (flag bit 3).
            p = subprocess.run(["zip", "-q", "-", "-"], input=FILES[0][1],
                               capture_output=True, check=True)
            write("bad-infozip-stdin-data-descriptor.zip", p.stdout)

    # The small writer, canonical layout.
    good = build(canon, b"archive comment")
    write("ok-canonical.zip", good)

    # One property broken at a time.
    write("bad-prefixed.zip", b"JUNK" + good)  # offsets not adjusted
    write("bad-trailing-byte.zip", good + b"\0")  # end record not at the end
    write("bad-gap-between-files.zip", build(canon, gap_after=0))
    write("bad-gap-before-cd.zip", build(canon, cd_gap=2))
    write("bad-local-order.zip", build(canon, order=[2, 1, 0]))
    write("bad-name-mismatch.zip",
          build(canon, cd_names=[b"hello.txt", b"dir/numbers.bin", b"EMPTY"]))
    write("bad-count-mismatch.zip", build(canon, total=4))
    # Two central entries pointing at the same local file (overlapping files).
    dup = [canon[0], canon[0]]
    write("bad-overlap.zip", build(dup, offsets=[0, 0], order=[0]))
    write("bad-data-descriptor-flag.zip",
          build([entry("hello.txt", FILES[0][1], flags=0x0008)]))
    # An archive whose comment ends with the end record of an empty archive:
    # scanning from the end finds the fake record first.
    write("bad-fake-eocd-in-comment.zip", build(canon, eocd(0, 0, 0)))


if __name__ == "__main__":
    main()
