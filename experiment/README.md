# Compressing a Japanese conjugation dictionary with a finite-state machine

**Question:** Finnish morphology can be represented with a finite state machine
exploiting its prefix–fuzzy–suffix structure. Japanese also has this structure —
can a Japanese conjugation dictionary be compressed the same way?

**Answer: yes — and it works *better* for Japanese than for Finnish.**

## Inspiration

Andrew Quinn, [*Replacing a 3 GB SQLite database with a 7 MB FST (finite state
transducer) binary*](https://til.andrew-quinn.me/posts/replacing-a-3-gb-sqlite-database-with-a-7-mb-fst-finite-state-trandsucer-binary/):
his Finnish–English dictionary app (Taskusanakirja) needed autocomplete over
tens of millions of inflected Finnish forms. A plain trie didn't scale past
~400k items and an SQLite FTS index ballooned to a 3 GB download; rebuilding
the lookup on Rust's [`fst`](https://github.com/BurntSushi/fst) crate — which,
unlike a trie, shares suffixes as well as prefixes — collapsed it to a
single-digit-MB binary, a ~300× reduction. The question this repo answers is
whether Japanese conjugation has enough of that shared prefix/suffix structure
to get the same payoff. (It does — see the JMdict numbers below.)

## Why Japanese is a friendly case

The "fuzzy" middle in Finnish comes from stem-internal alternations: consonant
gradation (*katto → katon*), vowel harmony choosing between suffix variants
(*-ssa/-ssä*), and stem-final mutations. A Finnish FSM has to encode a lot of
machinery in the middle of the word.

Japanese inflection has almost none of that:

- **No inflectional prefixes.** Everything happens at the right edge
  (the honorific お/ご is derivational, not part of conjugation).
- **No vowel harmony, no stem-internal gradation.** The stem is a literal
  constant prefix of every form. In the usual orthography the kanji stem is
  *character-identical* across all forms (書く, 書かない, 書いて…) — only the
  okurigana tail varies.
- **Alternation is confined to the last mora and is fully class-predictable.**
  Godan verbs swap the final kana across the five vowel rows (か/き/く/け/こ),
  ichidan verbs just drop る, and the euphonic te-form changes (く→いて,
  ぶ/む/ぬ→んで, う/つ/る→って) follow from the same final kana. There are only
  two irregular verbs (する, 来る) and one lexical exception (行く→行って).
- **Suffixes are agglutinative and heavily shared.** Every one of thousands of
  verbs ends in one of ~9 kana, and each class shares one suffix set —
  so the automaton's suffix tail is reused across the entire lexicon.

So the structure is really *prefix–(one predictable mora)–suffix*: the fuzzy
zone is a single character wide. That is close to the best case for automaton
minimization.

## Two ways to do it

### 1. Minimal acyclic FSA (DAFSA/DAWG) over the enumerated forms

Expand every lemma into its surface forms, sort, and build a minimal acyclic
automaton with Daciuk et al.'s incremental algorithm — the construction used by
Lucene's FST and the Rust [`fst`](https://github.com/BurntSushi/fst) crate.
Common prefixes merge going in (the trie part), and minimization merges the
identical suffix trees coming out — the ~35 endings of every godan -く verb
collapse into one shared tail.

- **Pros:** generic (no linguistics in the data structure), gives O(length)
  membership, ordered iteration, and fuzzy/prefix search for free.
- **Cons:** the form list must be finite — you pre-decide how deep the
  auxiliary chains go (食べさせられたくなかった…). If you need *form → lemma +
  features* rather than just membership, use a transducer (FST with outputs);
  in Lucene/`fst` the output weights add modest overhead.

### 2. Lexical FST: lemma + conjugation class + shared suffix automata

Store only lemmas tagged with a class ID, plus one small suffix automaton per
class (5–15 classes cover Japanese: godan ×9 endings collapse to one
parameterized class, ichidan, する, 来る, i-adjectives, な-adjective/copula).
This is the classic two-level morphology approach used for Finnish
(Koskenniemi → HFST/Omorfi), buildable with [foma](https://fomafst.github.io/)
or HFST. Because suffix chains can loop, the automaton can be *cyclic* and
recognize unbounded agglutination (〜させられてしまっていた…), which no
enumerated list can. It also naturally emits analyses (lemma, tense, polarity,
politeness). This is essentially how Japanese analyzers work in practice:
MeCab/Sudachi dictionaries store stems + conjugation-type tables, and Kuromoji
stores its lexicon in a Lucene FST.

## Empirical result (this repo's experiment)

`dafsa_experiment.py` conjugates 333 common lemmas (godan, ichidan,
する-compounds, 来る, i-adjectives) into ~35 forms each — including
agglutinated chains like 〜ていました and causative-passives — then builds a
byte-level DAFSA and serializes it:

| representation | size | vs raw |
|---|---:|---:|
| raw UTF-8 form list (11,128 forms) | 178,186 B | 1.0× |
| gzip -9 | 32,509 B | 5.5× |
| prefix trie | 53,779 nodes | — |
| **minimal DAFSA** | **625 states / 1,162 edges → 4,580 B** | **38.9×** |
| lemma + class + suffix tables | 5,053 B | 35.3× |

Two things worth noting:

- **Suffix sharing does the heavy lifting:** minimization cuts the trie's
  53,778 edges to 1,162 — a 46× reduction. That *is* the
  prefix-fuzzy-suffix payoff, and it confirms the Finnish trick transfers.
- The generic DAFSA (38.9×) matches the hand-factored linguistic scheme
  (35.3×) — the automaton *discovers* the class structure on its own. And the
  gap over gzip widens with scale: with a full JMdict-sized lexicon
  (~10⁵ verbs) the suffix tails are already in the machine, so each new lemma
  costs roughly only its stem bytes.

Run it: `python3 dafsa_experiment.py` (stdlib only).

## Full-scale result: all of JMdict

[JMdict](https://www.edrdg.org/jmdict/j_jmdict.html) (EDRDG, CC BY-SA — the
open lexicon behind Jisho and most Japanese apps) is the natural corpus for
this: every entry is POS-tagged with its exact conjugation class (`v5k`, `v1`,
`vs-i`, `adj-i`, …), so it doubles as a conjugation dictionary once expanded.
`jmdict_experiment.py` extracts every conjugable lemma — including the
long-tail classes (行く-type `v5k-s`, 問う-type `v5u-s`, honorific くださる-type
`v5aru`, suppletive ある `v5r-i`, いい/かっこいい `adj-ix`, kana-spelled 来る) and
the ~14k `vs` nouns with する attached — and runs the same pipeline:

| | small demo | full JMdict |
|---|---:|---:|
| lemmas | 333 | 28,976 |
| surface forms | 11,128 | 1,051,733 |
| raw UTF-8 list | 174 KB | 22.6 MB |
| gzip -9 | 5.5× | 7.2× |
| DAFSA states / edges | 625 / 1,162 | 44,632 / 79,952 |
| edge reduction vs trie | 46× | 65× |
| **DAFSA serialized** | **4.5 KB (38.9×)** | **372 KB (60.9×)** |

The key observation: the compression ratio *improves* with scale (38.9× →
60.9×), exactly as predicted — once the shared suffix machinery is in the
automaton, each additional lemma costs little more than its stem bytes. A
million-form dictionary fits in ~372 KB (pure-Python build time: ~10 s), and
this is the same regime as the Finnish result above: raw enumeration grows
linearly, the automaton grows roughly with lexicon novelty.

Reproduce:

```
curl -O http://ftp.edrdg.org/pub/Nihongo/JMdict_e.gz && gunzip JMdict_e.gz
python3 jmdict_experiment.py JMdict_e
```

## Sudachi (and MeCab/Kuromoji) already work this way

Production Japanese morphological analyzers are living proof of the
approach — they just factor the automaton differently:

- **The lexicon is a compressed automaton over surface strings.**
  [Sudachi](https://github.com/WorksApplications/Sudachi) stores its lexicon
  in a **double-array trie** (darts-clone) mapping surface forms to word IDs,
  as does MeCab; Lucene's Kuromoji goes further and stores its lexicon in a
  **Lucene FST** — literally the structure measured here.
- **Conjugation lives in the entries + a finite-state grammar, not as
  enumerated full forms.** SudachiDict/UniDic don't list 食べさせられていました;
  they list morpheme-granular entries — verb *stems* annotated with
  conjugation type and form (五段-カ行, 連用形-促音便, …) plus the auxiliary
  morphemes (させ, られ, て, い, まし, た) as their own entries.
- **The connection matrix is the suffix automaton.** Each entry carries a
  left/right context ID, and a `matrix.def` of connection costs defines which
  morpheme classes may follow which. That table *is* a finite-state
  transition function over morpheme categories — a cyclic one, so unbounded
  agglutinative chains come for free. At analysis time the tokenizer builds a
  lattice of dictionary hits and runs Viterbi over the connection costs,
  i.e. it *composes* stem-automaton ∘ suffix-grammar on the fly instead of
  precomputing the composition like our DAFSA does.

So the two designs bracket the space: enumerate-then-minimize (this repo's
DAFSA; simple, finite, great for membership/autocomplete) versus
stems + finite-state connection grammar (Sudachi/MeCab; unbounded chains,
weighted disambiguation, produces analyses). Both are the
prefix–fuzzy–suffix FSM idea; Sudachi just keeps the suffix automaton
factored out and weighted.

## Practical recommendations

- **Just need a compact set of valid forms** (spell-check, autocomplete,
  membership): enumerate forms from JMdict + a conjugator, feed the sorted
  list to the `fst` crate or Lucene's FST builder. Done — expect tens-of-×
  compression, with lookup faster than decompress-and-search.
- **Need analysis (form → lemma/features) or unbounded auxiliary chains:**
  write a small lexc/foma grammar (a weekend project for Japanese — the rule
  set above is nearly the whole language) or reuse MeCab/Sudachi dictionary
  machinery.
- **Watch orthographic variance**, which hurts more than conjugation does:
  kanji vs kana spellings (たべる/食べる), okurigana variants (行なう/行う).
  Store one canonical spelling per lemma and normalize at query time, or add
  variants as extra lemmas — the FSA absorbs them cheaply since they share
  suffixes too.

## Files

- `lemmas.py` — ~330 common lemmas tagged by conjugation class
- `conjugate.py` — the conjugator (godan rows, onbin te-forms, ichidan, する,
  来る, i-adjectives)
- `dafsa_experiment.py` — DAFSA construction (Daciuk), serialization, and the
  small-demo size comparison
- `jmdict_experiment.py` — full-scale run over every conjugable lemma in
  JMdict (download instructions in the file header; the corpus itself is not
  committed)
