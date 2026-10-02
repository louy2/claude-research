#!/usr/bin/env python3
"""Compare pzf.py output against the CSV exports deposited with the same figures.

samples/abdulkareem2024_*.pzf and *.csv come from Zenodo record 13629114
(CC-BY 4.0), where the authors published both the Prism files and CSV
exports of the same data. The CSVs have two header rows; pzf.py writes one.
"""
import csv
import glob
import io
import math
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
total = equal = 0
for pz in sorted(glob.glob(os.path.join(HERE, "samples", "abdulkareem2024_*.pzf"))):
    published = list(csv.reader(open(pz[:-4] + ".csv", encoding="utf-8-sig")))[2:]
    out = subprocess.run([sys.executable, os.path.join(HERE, "pzf.py"), "csv", pz, "0"],
                         capture_output=True, text=True, check=True).stdout
    mine = list(csv.reader(io.StringIO(out)))[1:]
    n = ok = 0
    for a, b in zip(published, mine):
        for k in range(max(len(a), len(b))):
            x = a[k].strip() if k < len(a) else ""
            y = b[k].strip() if k < len(b) else ""
            if not x and not y:
                continue
            n += 1
            try:
                ok += math.isclose(float(x), float(y), rel_tol=1e-6)
            except ValueError:
                ok += x == y
    rows_ok = len(published) == len(mine)
    print(f"{os.path.basename(pz):45} rows {len(mine)}/{len(published)}  cells equal {ok}/{n}")
    total += n
    equal += ok + (0 if rows_ok else -1)
print(f"total: {equal}/{total} cells equal")
sys.exit(0 if equal == total else 1)
