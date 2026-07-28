# Compressing a Japanese conjugation dictionary with a finite-state machine

**Question:** Finnish morphology can be represented with a finite state machine
exploiting its prefix–fuzzy–suffix structure. Japanese also has this structure —
can a Japanese conjugation dictionary be compressed the same way?

**Answer: yes — and it works *better* for Japanese than for Finnish.**

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
  size comparison above
