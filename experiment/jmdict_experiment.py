#!/usr/bin/env python3
"""Full-scale run: compress every conjugable lemma in JMdict.

JMdict (EDRDG, https://www.edrdg.org/jmdict/j_jmdict.html) is the standard
open Japanese lexicon (the one behind Jisho and most J-E apps). Every entry
carries POS tags naming its exact conjugation class (v5k, v1, vs-i, adj-i,
...), so it doubles as a conjugation dictionary once expanded.

Usage:
    curl -O http://ftp.edrdg.org/pub/Nihongo/JMdict_e.gz && gunzip JMdict_e.gz
    python3 jmdict_experiment.py [path/to/JMdict_e] [--no-vs-nouns]
"""

import gzip
import re
import sys
import time

import conjugate
from conjugate import _verb_forms, A_ROW, I_ROW, E_ROW, O_ROW, TE
from dafsa_experiment import build_dafsa, dafsa_states, serialize, contains


# ---------------------------------------------------------- class handlers

def godan(lemma, te_override=None):
    stem, last = lemma[:-1], lemma[-1]
    te = te_override or stem + TE[last]
    a, i, e, o = (stem + row[last] for row in (A_ROW, I_ROW, E_ROW, O_ROW))
    return _verb_forms(a, i, te, lemma, e,
                       potential=e + "る", passive=a + "れる",
                       causative=a + "せる", imperative=e, volitional=o + "う")


def v5k_s(lemma):        # 行く and compounds: te-form 行って not 行いて
    return godan(lemma, te_override=lemma[:-1] + "って")


def v5u_s(lemma):        # 問う, 乞う: te-form 問うて (no っ)
    return godan(lemma, te_override=lemma + "て")


def v5aru(lemma):        # くださる, いらっしゃる: masu-stem/imperative ~い
    stem = lemma[:-1]
    a, e, o = stem + "ら", stem + "れ", stem + "ろ"
    return _verb_forms(a, stem + "い", stem + "って", lemma, e,
                       potential=e + "る", passive=a + "れる",
                       causative=a + "せる", imperative=stem + "い",
                       volitional=o + "う")


def v5r_i(lemma):        # ある: suppletive negative, no passive etc.
    stem = lemma[:-1]
    i, e = stem + "り", stem + "れ"
    neg = lemma[:-2] + "ない"      # ある → ない
    te = stem + "って"
    ta = stem + "った"
    return {lemma, neg, neg[:-1] + "かった", neg[:-1] + "ければ",
            i + "ます", i + "ません", i + "ました", i + "ませんでした",
            te, ta, ta + "ら", ta + "り", e + "ば"}


def ichidan(lemma):
    return conjugate.ichidan(lemma)


def v1_s(lemma):         # くれる: imperative くれ
    forms = conjugate.ichidan(lemma)
    forms.discard(lemma[:-1] + "ろ")
    forms.add(lemma[:-1])
    return forms


def vs(lemma):           # する and compounds
    return conjugate.suru(lemma)


def vs_s(lemma):         # 愛する-type: potential 愛せる, not 愛できる
    stem = lemma[:-2]
    return _verb_forms(stem + "し", stem + "し", stem + "して", lemma,
                       stem + "すれ", potential=stem + "せる",
                       passive=stem + "される", causative=stem + "させる",
                       imperative=stem + "しろ", volitional=stem + "しよう")


def vk(lemma):
    if lemma.endswith("来る") or lemma.endswith("來る"):
        return conjugate.kuru(lemma)
    # kana spelling: ...くる → stem alternates こ/き/く
    stem = lemma[:-2]
    return _verb_forms(stem + "こ", stem + "き", stem + "きて", lemma,
                       stem + "くれ", potential=stem + "こられる",
                       passive=stem + "こられる", causative=stem + "こさせる",
                       imperative=stem + "こい", volitional=stem + "こよう")


def adj_i(lemma):
    return conjugate.i_adjective(lemma)


def adj_ix(lemma):       # いい and compounds (かっこいい): inflect on よ
    base = lemma[:-2] + "よい"
    forms = conjugate.i_adjective(base)
    forms -= {base, base + "です"}
    forms |= {lemma, lemma + "です"}
    return forms


# tag -> (handler, required-ending regex)
CLASSES = {
    "v5u": (godan, "う"), "v5k": (godan, "く"), "v5g": (godan, "ぐ"),
    "v5s": (godan, "す"), "v5t": (godan, "つ"), "v5n": (godan, "ぬ"),
    "v5b": (godan, "ぶ"), "v5m": (godan, "む"), "v5r": (godan, "る"),
    "v5k-s": (v5k_s, "く"), "v5u-s": (v5u_s, "う"), "v5aru": (v5aru, "る"),
    "v5r-i": (v5r_i, "る"),
    "v1": (ichidan, "る"), "v1-s": (v1_s, "る"),
    "vs-i": (vs, "する"), "vs-s": (vs_s, "する"),
    "vk": (vk, "(来る|來る|くる)"),
    "adj-i": (adj_i, "い"), "adj-ix": (adj_ix, "いい"),
}

ENTRY_RE = re.compile(r"<entry>(.*?)</entry>", re.S)
KEB_RE = re.compile(r"<keb>(.*?)</keb>")
REB_RE = re.compile(r"<reb>(.*?)</reb>")
POS_RE = re.compile(r"<pos>&([a-z0-9-]+);</pos>")


def collect_lemmas(xml_text, include_vs_nouns=True):
    """Yield (lemma, class_tag) pairs from JMdict XML."""
    seen = set()
    skipped = 0
    for m in ENTRY_RE.finditer(xml_text):
        entry = m.group(1)
        keb = KEB_RE.search(entry)
        reb = REB_RE.search(entry)
        lemma = keb.group(1) if keb else reb.group(1) if reb else None
        if not lemma:
            continue
        tags = set(POS_RE.findall(entry))
        for tag in tags:
            if tag in CLASSES:
                _, ending = CLASSES[tag]
                if not re.search(ending + "$", lemma):
                    skipped += 1
                    continue
                if (lemma, tag) not in seen:
                    seen.add((lemma, tag))
                    yield lemma, tag
            elif tag == "vs" and include_vs_nouns:
                # noun that conjugates with する: 勉強 → 勉強する
                key = (lemma + "する", "vs-i")
                if key not in seen:
                    seen.add(key)
                    yield key
    print(f"  (skipped {skipped} lemmas whose spelling doesn't match "
          f"their conjugation class)", file=sys.stderr)


def sorted_trie_nodes(sorted_words):
    """Trie node count from sorted input in O(1) memory: each word adds one
    node per byte past its common prefix with the previous word."""
    nodes = 1
    prev = b""
    for w in sorted_words:
        common = 0
        while common < len(prev) and common < len(w) and w[common] == prev[common]:
            common += 1
        nodes += len(w) - common
        prev = w
    return nodes


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    include_vs = "--no-vs-nouns" not in sys.argv
    path = args[0] if args else "JMdict_e"
    with open(path, encoding="utf-8") as f:
        xml_text = f.read()

    t0 = time.time()
    lemmas = list(collect_lemmas(xml_text, include_vs))
    by_class = {}
    forms = set()
    for lemma, tag in lemmas:
        by_class[tag] = by_class.get(tag, 0) + 1
        forms.update(CLASSES[tag][0](lemma))
    words = sorted(f.encode("utf-8") for f in forms)
    t_gen = time.time() - t0

    raw = b"\n".join(words) + b"\n"
    gz = gzip.compress(raw, 9)
    trie_nodes = sorted_trie_nodes(words)

    t0 = time.time()
    root = build_dafsa(words)
    t_build = time.time() - t0
    states = dafsa_states(root)
    n_edges = sum(len(s.edges) for s in states)
    for w in words[::97]:           # spot-check membership
        assert contains(root, w), w
    for bad in ["食べます行く", "食べれない", "勉強するない", "来ない来"]:
        assert not contains(root, bad.encode("utf-8")), bad
    dafsa_bytes = serialize(root)

    print(f"conjugable lemmas:       {len(lemmas):>12,}")
    for tag, n in sorted(by_class.items(), key=lambda kv: -kv[1]):
        print(f"    {tag:<8} {n:>8,}")
    print(f"surface forms:           {len(words):>12,}  "
          f"(avg {len(words)/len(lemmas):.1f}/lemma, generated in {t_gen:.1f}s)")
    print()
    print(f"raw UTF-8 list:          {len(raw):>12,} B")
    print(f"gzip -9:                 {len(gz):>12,} B  ({len(raw)/len(gz):6.1f}x vs raw)")
    print(f"prefix trie:             {trie_nodes:>12,} nodes")
    print(f"DAFSA:                   {len(states):>12,} states / {n_edges:,} edges  "
          f"(built in {t_build:.1f}s)")
    print(f"  edge reduction vs trie: {(trie_nodes - 1)/n_edges:10.1f}x")
    print(f"DAFSA serialized:        {dafsa_bytes:>12,} B  ({len(raw)/dafsa_bytes:6.1f}x vs raw)")


if __name__ == "__main__":
    sys.setrecursionlimit(100_000)
    main()
