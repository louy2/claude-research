#!/usr/bin/env python3
"""Compress the UniDic lexicon's surface vocabulary with a DAFSA.

UniDic (NINJAL, https://clrd.ninjal.ac.jp/unidic/) is the reference lexicon
for Japanese morphological analysis — the dictionary MeCab/Sudachi-style
analyzers actually ship. Unlike JMdict, it *already enumerates* conjugated
forms: each verb/adjective surface appears as its own row tagged with
conjugation type and form (活用型/活用形), e.g. 食べる×終止形, 食べる×連体形,
食べさせ×命令形. So there is nothing to expand — we compress the surface
column of a production dictionary as-is, and report the conjugable subset
(rows with a conjugation type) separately for comparison with the JMdict run.

Usage:
    curl -O https://clrd.ninjal.ac.jp/unidic_archive/cwj/3.1.1/unidic-cwj-3.1.1.zip
    unzip unidic-cwj-3.1.1.zip unidic-cwj-3.1.1/lex_3_1.csv
    python3 unidic_experiment.py unidic-cwj-3.1.1/lex_3_1.csv
"""

import csv
import gzip
import sys
import time

from dafsa_experiment import build_dafsa, dafsa_states, serialize, contains
from jmdict_experiment import sorted_trie_nodes

POS1, CTYPE = 4, 8


def measure(name, words):
    words = sorted(words)
    raw_len = sum(len(w) + 1 for w in words)
    gz = len(gzip.compress(b"\n".join(words) + b"\n", 9))
    trie_nodes = sorted_trie_nodes(words)

    t0 = time.time()
    root = build_dafsa(words)
    t_build = time.time() - t0
    states = dafsa_states(root)
    n_edges = sum(len(s.edges) for s in states)
    for w in words[::199]:
        assert contains(root, w), w
    nbytes = serialize(root)

    print(f"[{name}]  {len(words):,} unique surface strings")
    print(f"  raw UTF-8 list:    {raw_len:>12,} B")
    print(f"  gzip -9:           {gz:>12,} B  ({raw_len/gz:6.1f}x vs raw)")
    print(f"  prefix trie:       {trie_nodes:>12,} nodes")
    print(f"  DAFSA:             {len(states):>12,} states / {n_edges:,} edges"
          f"  (built in {t_build:.1f}s)")
    print(f"    edge reduction vs trie: {(trie_nodes - 1)/n_edges:8.1f}x")
    print(f"  DAFSA serialized:  {nbytes:>12,} B  ({raw_len/nbytes:6.1f}x vs raw)")
    print()


def main():
    path = sys.argv[1] if len(sys.argv) > 1 else "lex_3_1.csv"
    n_rows = 0
    all_surfaces = set()
    conjugable = set()
    pos_counts = {}
    with open(path, encoding="utf-8", newline="") as f:
        for row in csv.reader(f):
            n_rows += 1
            surface = row[0].encode("utf-8")
            if not surface:
                # one malformed row in 3.1.1: the ASCII '"' symbol entry
                # has an empty surface field
                continue
            all_surfaces.add(surface)
            pos_counts[row[POS1]] = pos_counts.get(row[POS1], 0) + 1
            if row[CTYPE] != "*":
                conjugable.add(surface)

    print(f"lexicon rows: {n_rows:,}   (top POS: "
          + ", ".join(f"{p} {n:,}"
                      for p, n in sorted(pos_counts.items(),
                                         key=lambda kv: -kv[1])[:6]) + ")")
    print()
    measure("conjugable rows only (活用型 != *)", conjugable)
    measure("entire lexicon", all_surfaces)


if __name__ == "__main__":
    sys.setrecursionlimit(100_000)
    main()
