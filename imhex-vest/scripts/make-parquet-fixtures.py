#!/usr/bin/env python3
"""Writes the Parquet fixtures used by interval-vest/tests/parquet_profile.rs.

    scripts/make-parquet-fixtures.py [OUT_DIR]   (default: interval-vest/tests/data)

Needs pyarrow; duckdb, polars and fastparquet are used when installed.
ok-*.parquet are expected to be accepted by the strict profile,
bad-*.parquet to be rejected. The bad files are made from a pyarrow file by
decoding its FileMetaData, changing one thing, and writing it back, so that
everything else stays consistent.
"""
import io
import os
import struct
import sys

import pyarrow as pa
import pyarrow.parquet as pq

OUT = sys.argv[1] if len(sys.argv) > 1 else os.path.join(
    os.path.dirname(__file__), "..", "interval-vest", "tests", "data")

# ---------------------------------------------------------------------------
# Thrift compact, with types kept so that values can be written back.
# A struct is a list of [field id, (type code, payload)].

STRUCT, LIST, I32, I64, BINARY = 12, 9, 5, 6, 8


def rd_varint(s, i):
    v, shift = 0, 0
    while True:
        b = s[i]
        i += 1
        v |= (b & 0x7F) << shift
        shift += 7
        if b < 0x80:
            return i, v


def unzz(u):
    return (u >> 1) ^ -(u & 1)


def zz(x):
    return 2 * x if x >= 0 else -2 * x - 1


def rd_value(s, i, t):
    if t in (1, 2):
        return i + 1, s[i] == 1
    if t == 3:
        return i + 1, s[i]
    if t in (4, 5, 6):
        i, u = rd_varint(s, i)
        return i, unzz(u)
    if t == 7:
        return i + 8, s[i:i + 8]
    if t == 8:
        i, n = rd_varint(s, i)
        return i + n, s[i:i + n]
    if t == 9:
        h = s[i]
        et, n = h & 15, h >> 4
        i += 1
        if n == 15:
            i, n = rd_varint(s, i)
        xs = []
        for _ in range(n):
            i, x = rd_value(s, i, et)
            xs.append(x)
        return i, (et, xs)
    if t == 12:
        return rd_struct(s, i)
    raise ValueError(t)


def rd_struct(s, i):
    last, fs = 0, []
    while True:
        b = s[i]
        i += 1
        if b == 0:
            return i, fs
        t, delta = b & 15, b >> 4
        if delta:
            fid = last + delta
        else:
            i, u = rd_varint(s, i)
            fid = unzz(u)
        if t in (1, 2):
            v = t == 1
        else:
            i, v = rd_value(s, i, t)
        fs.append([fid, (t, v)])
        last = fid


def wr_varint(v, pad=False):
    out = bytearray()
    while True:
        if v < 0x80:
            out.append(v)
            break
        out.append((v & 0x7F) | 0x80)
        v >>= 7
    if pad:  # a non-minimal encoding of the same value
        out[-1] |= 0x80
        out.append(0)
    return bytes(out)


def wr_value(t, v, quirks):
    if t in (1, 2):
        return bytes([1 if v else 2])
    if t == 3:
        return bytes([v])
    if t in (4, 5, 6):
        return wr_varint(zz(v))
    if t == 7:
        return v
    if t == 8:
        return wr_varint(len(v), quirks.pop("pad_length", False)) + v
    if t == 9:
        et, xs = v
        h = bytes([len(xs) << 4 | et]) if len(xs) < 15 else bytes([0xF0 | et]) + wr_varint(len(xs))
        return h + b"".join(wr_value(et, x, quirks) for x in xs)
    if t == 12:
        return wr_struct(v, quirks)
    raise ValueError(t)


def wr_struct(fs, quirks):
    out, last = bytearray(), 0
    order = list(fs)
    if quirks.pop("swap_first_fields", False):
        order[0], order[1] = order[1], order[0]
    for fid, (t, v) in order:
        delta = fid - last
        if 0 < delta <= 15 and not quirks.pop("long_header", False):
            out.append(delta << 4 | t)
        else:
            out.append(t)
            out += wr_varint(zz(fid))
        if t not in (1, 2):
            out += wr_value(t, v, quirks)
        last = fid
    out.append(0)
    return bytes(out)


def field(fs, fid):
    for f in fs:
        if f[0] == fid:
            return f
    return None


def get(fs, fid):
    f = field(fs, fid)
    return f[1][1] if f else None


def put(fs, fid, t, v):
    f = field(fs, fid)
    if f:
        f[1] = (t, v)
    else:
        fs.append([fid, (t, v)])
        fs.sort(key=lambda x: x[0])


# ---------------------------------------------------------------------------
# Parquet files as (body, FileMetaData).

def split(b):
    n = int.from_bytes(b[-8:-4], "little")
    meta = b[len(b) - 8 - n:len(b) - 8]
    return bytearray(b[:len(b) - 8 - n]), rd_struct(meta, 0)[1]


def join(body, md, quirks=None):
    meta = wr_struct(md, dict(quirks or {}))
    return bytes(body) + meta + struct.pack("<I", len(meta)) + b"PAR1"


def chunks(md):
    for rg in get(md, 4)[1]:
        for cc in get(rg, 1)[1]:
            yield rg, cc


def shift_offsets(md, at, by):
    """Moves every offset at or after `at` by `by`."""
    for _, cc in chunks(md):
        m = get(cc, 3)
        for fs, k in ((m, 9), (m, 11), (m, 14), (cc, 2), (cc, 4), (cc, 6)):
            v = get(fs, k)
            if v is not None and v >= at:
                put(fs, k, I64, v + by)


def roundtrip_ok(b):
    body, md = split(b)
    assert join(body, md) == b, "the fixture encoder is not canonical for this file"


def main():
    os.makedirs(OUT, exist_ok=True)
    n = 200
    t = pa.table({
        "id": pa.array(range(n), pa.int64()),
        "name": [f"n{i % 7}" for i in range(n)],
        "x": [i * 0.25 for i in range(n)],
        "flag": [i % 3 == 0 for i in range(n)],
        "maybe": [None if i % 5 == 0 else i for i in range(n)],
    })

    def pyarrow(**kw):
        buf = io.BytesIO()
        pq.write_table(t, buf, **kw)
        return buf.getvalue()

    def write(name, data):
        with open(os.path.join(OUT, name), "wb") as f:
            f.write(data)

    base = pyarrow(compression="NONE")
    roundtrip_ok(base)
    write("ok-pyarrow.parquet", base)
    write("ok-pyarrow-snappy.parquet", pyarrow(compression="SNAPPY"))
    write("ok-pyarrow-v2pages.parquet", pyarrow(data_page_version="2.0"))
    write("ok-pyarrow-rowgroups.parquet", pyarrow(row_group_size=50))
    write("ok-pyarrow-nodict.parquet", pyarrow(use_dictionary=False))
    write("ok-pyarrow-pageindex.parquet", pyarrow(write_page_index=True))
    write("ok-pyarrow-bloom.parquet", pyarrow(bloom_filter_options={"name": {"ndv": 10}})
          if "bloom_filter_options" in pq.write_table.__code__.co_varnames else base)
    buf = io.BytesIO()
    pq.write_table(t.slice(0, 0), buf)
    write("ok-pyarrow-empty.parquet", buf.getvalue())
    try:
        import duckdb
        con = duckdb.connect()
        con.register("t", t)
        path = os.path.join(OUT, "ok-duckdb.parquet")
        con.sql(f"COPY t TO '{path}' (FORMAT parquet)")
    except ImportError:
        pass
    try:
        import polars as pl
        buf = io.BytesIO()
        pl.from_arrow(t).write_parquet(buf)
        # Polars writes a ColumnMetaData copy after every chunk but sets
        # file_offset to data_page_offset + total_compressed_size, which
        # misses the copy when the chunk starts with a dictionary page.
        write("bad-polars-unreferenced-copy.parquet", buf.getvalue())
    except ImportError:
        pass
    try:
        import fastparquet
        path = os.path.join(OUT, "ok-fastparquet.parquet")
        fastparquet.write(path, t.to_pandas())
    except ImportError:
        pass

    # A legacy layout: a copy of each ColumnChunk after its data, at
    # file_offset (Impala, parquet-cpp 1.x, parquet-rs 9).
    body, md = split(base)
    out = bytearray(body[:4])
    pos = 4
    for _, cc in list(chunks(md)):
        m = get(cc, 3)
        start = get(m, 11) if get(m, 11) is not None else get(m, 9)
        size = get(m, 7)
        delta = len(out) - start
        for k in (9, 11):
            if get(m, k) is not None:
                put(m, k, I64, get(m, k) + delta)
        out += body[start:start + size]
        put(cc, 2, I64, len(out))
        out += wr_struct([[2, (I64, len(out))], [3, (STRUCT, m)]], {})
        pos = start + size
    legacy = join(out, md)
    write("ok-legacy-inline-copy.parquet", legacy)

    # One property broken at a time.
    write("bad-trailing-byte.parquet", base + b"\0")
    write("bad-no-head.parquet", b"PAR0" + base[4:])

    body, md = split(base)
    first = next(chunks(md))[1]
    m = get(first, 3)
    end0 = (get(m, 11) if get(m, 11) is not None else get(m, 9)) + get(m, 7)
    gap = body[:end0] + b"\0\0\0" + body[end0:]
    shift_offsets(md, end0, 3)
    write("bad-gap-between-chunks.parquet", join(gap, md))

    body, md = split(base)
    cs = [cc for _, cc in chunks(md)]
    m0, m1 = get(cs[0], 3), get(cs[1], 3)
    for k in (7, 9, 11, 6):
        if get(m0, k) is not None:
            put(m1, k, I64, get(m0, k))
    write("bad-overlapping-chunks.parquet", join(body, md))

    body, md = split(base)
    put(md, 3, I64, get(md, 3) + 1)
    write("bad-num-rows.parquet", join(body, md))

    body, md = split(base)
    m = get(next(chunks(md))[1], 3)
    put(m, 5, I64, get(m, 5) + 1)
    write("bad-num-values.parquet", join(body, md))

    body, md = split(base)
    m = get(next(chunks(md))[1], 3)
    put(m, 9, I64, get(m, 9) + 1)
    write("bad-data-page-offset.parquet", join(body, md))

    body, md = split(base)
    m = get(next(chunks(md))[1], 3)
    put(m, 6, I64, get(m, 6) + 1)
    write("bad-uncompressed-size.parquet", join(body, md))

    # Encodings of the same FileMetaData that Thrift readers accept but the
    # canonical profile does not.
    body, md = split(base)
    write("bad-nonminimal-varint.parquet", join(body, md, {"pad_length": True}))
    write("bad-long-field-header.parquet", join(body, md, {"long_header": True}))
    write("bad-field-order.parquet", join(body, md, {"swap_first_fields": True}))

    # A legacy copy that disagrees with the footer.
    body, md = split(legacy)
    cc = next(chunks(md))[1]
    fo = get(cc, 2)
    p, copy = rd_struct(body, fo)
    cm = get(copy, 3)
    put(cm, 5, I64, get(cm, 5) + 1)
    new = wr_struct(copy, {})
    assert len(new) == p - fo
    body[fo:p] = new
    write("bad-inline-copy-mismatch.parquet", join(body, md))


if __name__ == "__main__":
    main()
