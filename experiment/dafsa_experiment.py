#!/usr/bin/env python3
"""Measure how well a minimal acyclic FSA (DAFSA) compresses a Japanese
conjugation dictionary, versus a raw list, gzip, a prefix-only trie, and a
lemma+class factored scheme.

The DAFSA is built with Daciuk et al.'s incremental algorithm for sorted
input — the same construction used by Lucene's FST and the Rust `fst` crate —
over UTF-8 bytes, and serialized to a compact flat encoding to get an honest
byte count.

Run: python3 dafsa_experiment.py
"""

import gzip
import io
import sys

import conjugate


# ---------------------------------------------------------------- DAFSA

class State:
    __slots__ = ("edges", "final", "id")
    _next_id = 0

    def __init__(self):
        self.edges = {}   # byte -> State
        self.final = False
        self.id = State._next_id
        State._next_id += 1

    def signature(self):
        return (self.final, tuple((b, s.id) for b, s in sorted(self.edges.items())))


def build_dafsa(sorted_words):
    """Daciuk's algorithm 1: incremental construction from sorted input."""
    register = {}
    root = State()
    prev = b""

    def replace_or_register(state):
        # Minimize the last child of `state`, recursively.
        if not state.edges:
            return
        last_byte = max(state.edges)
        child = state.edges[last_byte]
        replace_or_register(child)
        sig = child.signature()
        existing = register.get(sig)
        if existing is not None and existing is not child:
            state.edges[last_byte] = existing
        else:
            register[sig] = child

    for word in sorted_words:
        assert word > prev, "input must be sorted and unique"
        # Walk the common prefix.
        common = 0
        state = root
        while common < len(word) and common < len(prev) and word[common] == prev[common]:
            state = state.edges[word[common]]
            common += 1
        # Anything past the diverging edge of the previous word is finished:
        # minimize it before adding the new suffix.
        replace_or_register(state)
        for b in word[common:]:
            new = State()
            state.edges[b] = new
            state = new
        state.final = True
        prev = word

    replace_or_register(root)
    return root


def dafsa_states(root):
    seen = {}
    stack = [root]
    while stack:
        s = stack.pop()
        if s.id in seen:
            continue
        seen[s.id] = s
        stack.extend(s.edges.values())
    return list(seen.values())


def contains(root, word):
    s = root
    for b in word:
        s = s.edges.get(b)
        if s is None:
            return False
    return s.final


def serialize(root):
    """Flat encoding: states in topological-ish order, each edge as
    (label byte, flags, varint delta to target). Mirrors the layout of
    production FST libraries closely enough for a size estimate."""
    states = dafsa_states(root)
    # Order states by DFS so edge targets are usually nearby (small varints).
    order = {}
    stack = [root]
    while stack:
        s = stack.pop()
        if s.id in order:
            continue
        order[s.id] = len(order)
        for b in sorted(s.edges, reverse=True):
            stack.append(s.edges[b])

    def varint(n):
        out = bytearray()
        while True:
            out.append((n & 0x7F) | (0x80 if n > 0x7F else 0))
            n >>= 7
            if not n:
                return bytes(out)

    # First pass with 2-byte target guesses to fix positions, then iterate
    # until offsets stabilize.
    positions = {sid: i * 3 for sid, i in order.items()}  # rough seed
    for _ in range(10):
        buf = {}
        pos = 0
        changed = False
        for s in sorted(states, key=lambda s: order[s.id]):
            b = bytearray()
            edges = sorted(s.edges.items())
            for i, (label, target) in enumerate(edges):
                flags = 0
                if i == len(edges) - 1:
                    flags |= 1          # last edge of this state
                if target.final:
                    flags |= 2
                b.append(label)
                b.append(flags)
                b += varint(abs(positions[target.id]))
            if not edges:
                b.append(0)  # terminal marker for edgeless final state
            buf[s.id] = bytes(b)
            if positions[s.id] != pos:
                positions[s.id] = pos
                changed = True
            pos += len(b)
        if not changed:
            break
    total = sum(len(v) for v in buf.values())
    return total


# ---------------------------------------------------------------- Trie

def trie_stats(sorted_words):
    """Prefix-only trie (no suffix sharing) node/edge count for comparison."""
    root = {}
    nodes = 1
    for word in sorted_words:
        cur = root
        for b in word:
            nxt = cur.get(b)
            if nxt is None:
                nxt = cur[b] = {}
                nodes += 1
            cur = nxt
    edges = nodes - 1
    return nodes, edges


# ---------------------------------------------------------------- Main

def main():
    entries = list(conjugate.all_forms())
    forms = set()
    for _, _, fs in entries:
        forms.update(fs)
    words = sorted(f.encode("utf-8") for f in forms)

    raw = b"\n".join(words) + b"\n"
    gz = gzip.compress(raw, 9)

    root = build_dafsa(words)
    states = dafsa_states(root)
    n_edges = sum(len(s.edges) for s in states)

    # Sanity: automaton accepts exactly the input set (spot-check rejects).
    for w in words:
        assert contains(root, w), w
    for bad in ["食べます行く", "食べれない", "高いかった", "書きて", "泳いて"]:
        assert not contains(root, bad.encode("utf-8")), bad
    dafsa_bytes = serialize(root)

    trie_nodes, trie_edges = trie_stats(words)

    # Factored scheme: lemma list + class id per lemma + shared suffix tables.
    lemma_bytes = sum(len(l.encode("utf-8")) + 2 for l, _, _ in entries)
    n_classes = len({c for _, c, _ in entries})
    # Suffix tables: measure actual distinct (class, suffix-pattern) payload.
    suffix_table_bytes = 0
    for cls, fn in conjugate.CONJUGATORS.items():
        probe = {"godan": "書く", "ichidan": "食べる", "suru": "勉強する",
                 "kuru": "来る", "i-adj": "高い"}[cls]
        stem_len = {"godan": 1, "ichidan": 1, "suru": 2, "kuru": 1, "i-adj": 1}[cls]
        stem = probe[:-stem_len]
        for form in fn(probe):
            # store the per-form replacement tail
            tail = form[len(stem):] if form.startswith(stem) else form
            suffix_table_bytes += len(tail.encode("utf-8")) + 1
    factored = lemma_bytes + suffix_table_bytes

    n_lemmas = len(entries)
    print(f"lemmas:                  {n_lemmas:>10,}  ({n_classes} conjugation classes)")
    print(f"surface forms:           {len(words):>10,}  (avg {len(words)/n_lemmas:.1f}/lemma)")
    print()
    print(f"raw UTF-8 list:          {len(raw):>10,} B")
    print(f"gzip -9:                 {len(gz):>10,} B  ({len(raw)/len(gz):5.1f}x vs raw)")
    print(f"prefix trie:             {trie_nodes:>10,} nodes / {trie_edges:,} edges")
    print(f"DAFSA:                   {len(states):>10,} states / {n_edges:,} edges")
    print(f"  edge reduction vs trie: {trie_edges/n_edges:9.1f}x")
    print(f"DAFSA serialized:        {dafsa_bytes:>10,} B  ({len(raw)/dafsa_bytes:5.1f}x vs raw)")
    print(f"lemma+class factored:    {factored:>10,} B  ({len(raw)/factored:5.1f}x vs raw)")
    print(f"  = {lemma_bytes:,} B lemma table + {suffix_table_bytes:,} B suffix tables")


if __name__ == "__main__":
    sys.setrecursionlimit(100_000)
    main()
