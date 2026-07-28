# Japanese conjugation generator.
#
# Generates a fixed set of surface forms per lemma. Deliberately covers the
# common single-word inflections plus a few agglutinated auxiliary chains
# (-te iru, -tai, causative-passive) so the form list exercises the same
# "shared stem + shared suffix" structure a real conjugation dictionary has.

# Godan sound rows, keyed by the dictionary-form final kana.
A_ROW = {"う": "わ", "く": "か", "ぐ": "が", "す": "さ", "つ": "た",
         "ぬ": "な", "ぶ": "ば", "む": "ま", "る": "ら"}
I_ROW = {"う": "い", "く": "き", "ぐ": "ぎ", "す": "し", "つ": "ち",
         "ぬ": "に", "ぶ": "び", "む": "み", "る": "り"}
E_ROW = {"う": "え", "く": "け", "ぐ": "げ", "す": "せ", "つ": "て",
         "ぬ": "ね", "ぶ": "べ", "む": "め", "る": "れ"}
O_ROW = {"う": "お", "く": "こ", "ぐ": "ご", "す": "そ", "つ": "と",
         "ぬ": "の", "ぶ": "ぼ", "む": "も", "る": "ろ"}
# Euphonic te-form endings (onbin).
TE = {"う": "って", "つ": "って", "る": "って", "く": "いて", "ぐ": "いで",
      "す": "して", "ぬ": "んで", "ぶ": "んで", "む": "んで"}


def _verb_forms(mizenkei, renyoukei, te, dict_form, ba_stem, potential,
                passive, causative, imperative, volitional):
    """Assemble surface forms from the six traditional stems + derived bases."""
    ta = te[:-1] + ("だ" if te.endswith("で") else "た")
    forms = {
        dict_form,
        mizenkei + "ない", mizenkei + "なかった", mizenkei + "なければ",
        renyoukei + "ます", renyoukei + "ません", renyoukei + "ました",
        renyoukei + "ませんでした", renyoukei + "ましょう",
        te, ta, ta + "ら", ta + "り",
        ba_stem + "ば",
        potential, potential[:-1] + "ない", potential[:-1] + "ます",
        passive, passive[:-1] + "ない", passive[:-1] + "ます",
        causative, causative[:-1] + "ない", causative[:-1] + "ます",
        causative[:-1] + "られる",  # causative-passive
        imperative, volitional,
        dict_form + "な",  # prohibitive
        renyoukei + "たい", renyoukei + "たくない", renyoukei + "たかった",
        renyoukei + "たくなかった",
        renyoukei + "ながら", renyoukei + "そう",
        te + "いる", te + "いた", te + "います", te + "いました",
        te + "しまう", te + "ください",
    }
    return forms


def godan(lemma):
    stem, last = lemma[:-1], lemma[-1]
    te = stem + TE[last]
    if lemma == "行く":  # sole onbin exception
        te = "行って"
    a, i, e, o = (stem + row[last] for row in (A_ROW, I_ROW, E_ROW, O_ROW))
    return _verb_forms(
        mizenkei=a, renyoukei=i, te=te, dict_form=lemma, ba_stem=e,
        potential=e + "る", passive=a + "れる", causative=a + "せる",
        imperative=e, volitional=o + "う")


def ichidan(lemma):
    stem = lemma[:-1]
    return _verb_forms(
        mizenkei=stem, renyoukei=stem, te=stem + "て", dict_form=lemma,
        ba_stem=stem + "れ", potential=stem + "られる",
        passive=stem + "られる", causative=stem + "させる",
        imperative=stem + "ろ", volitional=stem + "よう")


def suru(lemma):
    stem = lemma[:-2]  # strip する
    return _verb_forms(
        mizenkei=stem + "し", renyoukei=stem + "し", te=stem + "して",
        dict_form=lemma, ba_stem=stem + "すれ", potential=stem + "できる",
        passive=stem + "される", causative=stem + "させる",
        imperative=stem + "しろ", volitional=stem + "しよう")


def kuru(lemma):
    # Written with the 来 kanji all forms share the leading character;
    # readings differ (こ/き/く) but the orthographic strings are what a
    # dictionary of written forms stores.
    stem = lemma[:-1]
    return _verb_forms(
        mizenkei=stem, renyoukei=stem, te=stem + "て", dict_form=lemma,
        ba_stem=stem + "れ", potential=stem + "られる",
        passive=stem + "られる", causative=stem + "させる",
        imperative=stem + "い", volitional=stem + "よう")


def i_adjective(lemma):
    stem = lemma[:-1]
    return {
        lemma,
        stem + "くない", stem + "かった", stem + "くなかった",
        stem + "くて", stem + "ければ", stem + "く", stem + "さ",
        stem + "そう", stem + "すぎる", stem + "ければ",
        lemma + "です", stem + "かったです", stem + "くないです",
    }


CONJUGATORS = {
    "godan": godan,
    "ichidan": ichidan,
    "suru": suru,
    "kuru": kuru,
    "i-adj": i_adjective,
}


def all_forms():
    """Yield (lemma, class, forms) for every lemma in the embedded list."""
    import lemmas
    for lemma in dict.fromkeys(lemmas.GODAN):
        yield lemma, "godan", godan(lemma)
    for lemma in dict.fromkeys(lemmas.ICHIDAN):
        yield lemma, "ichidan", ichidan(lemma)
    for lemma in dict.fromkeys(lemmas.SURU_COMPOUND + ["する"]):
        yield lemma, "suru", suru(lemma)
    yield "来る", "kuru", kuru("来る")
    for lemma in dict.fromkeys(lemmas.I_ADJECTIVES):
        yield lemma, "i-adj", i_adjective(lemma)
