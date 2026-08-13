# Localizing the Fossil Web Interface

Fossil builds its HTML out of string literals that mix markup with the
English words a user reads:

~~~
  @ <th>Branch Name</th>
~~~

There is no message catalog behind those literals, and translating them by
hand would mean forking every page in the tree.  This build instead
translates the text on its way out, in `src/i18n.c`.  The result is that a
translation is a data file, upstream source stays untouched, and a phrase
nobody has translated yet simply appears in English.

## Turning it on and off

The language comes from the `locale` setting:

~~~
  fossil setting locale zh-CN     # Simplified Chinese (the default here)
  fossil setting locale en        # the original English text
~~~

`FOSSIL_LOCALE` overrides the setting for one server process, which is handy
for comparing the two:

~~~
  FOSSIL_LOCALE=en fossil ui myrepo.fossil
~~~

Every catalog compiled into the executable is listed by:

~~~
  fossil test-locale list
~~~

The setting is repository-wide.  Fossil caches pages by ETag, and content
negotiation per visitor would make those cache entries wrong, so there is no
`Accept-Language` support: everyone browsing one repository sees one
language.

## How a page gets translated

`cgi_printf()` is the funnel that every web page passes through.  This build
routes its format string through `i18n_format()` first (`src/cgi.c`), which
walks the HTML and replaces the natural language in it.  Three more places
feed the same machinery, because their text arrives as a printf *argument*
rather than as part of the format:

  *  `style_header()` and the `style_submenu_*()` widgets in `src/style.c`
  *  the main menu, whose labels are translated inside
     `style_get_mainmenu()` while the URLs and capability expressions are
     left alone
  *  literal text in a skin's TH1 header and footer, and anything a skin
     writes with the TH1 `html` command (`src/th_main.c`)

Only replies whose content type is `text/html` are touched, so RSS, JSON and
raw file downloads come out byte for byte as upstream produced them.

## What is safe, and why

The translator is exact-match.  A phrase is replaced only when its
whitespace-normalized text is a key in the catalog; anything else is copied
through unchanged.  A page can therefore end up partly translated, but it
cannot end up mangled.

Three further rules keep it out of trouble:

  1. **Markup is never rewritten.**  Text is taken only from between tags.
     Inside a tag, only `title=`, `placeholder=`, `alt=`, `aria-label=` and
     `value=` on a push button are eligible, and only when the translation
     does not contain the quote character that delimits the value.

  2. **`<script>`, `<style>`, `<pre>` and `<textarea>` are opaque.**  Fossil
     often opens such an element in one `cgi_printf()` call and closes it in
     another, so the translator carries that state across calls; the same is
     done for a tag that is split across two calls.

  3. **printf conversions must line up.**  A cgi_printf format string is
     being rewritten, so a translation that dropped or reordered a `%s`
     would read the wrong argument off the stack.  A translation is used
     only when its sequence of conversions is identical to the source's.
     `fossil test-locale check` reports violations, and the same test runs
     again at display time.

To confirm the mechanism itself is inert when it has nothing to say, compare
a page from this build in English mode against upstream Fossil: the bytes
are identical apart from the `lang` attribute on `<html>`.  Comparing the
Chinese and English renderings of the same page shows every tag in the same
order with the same attributes - only the text between them differs.

## Writing a translation

A catalog is a builtin file named `src/locale/LOCALE.txt`.  Each record is
a `msgid` line holding the text exactly as the source produces it, followed
by a `msgstr` line holding the replacement:

~~~
  # Comments run to the end of the line.
  msgid  Branch Name
  msgstr 分支名称
~~~

Whitespace is normalized before lookup, so a phrase that the source wraps
over three lines is written here on one.  An empty `msgstr` means "not
translated yet" and leaves the English in place, which makes it convenient
to commit a stub catalog and fill it in over time.

Inline markup does not break a phrase.  Given

~~~
  @ Click <a href="%R/setup">here</a> to continue.
~~~

the catalog key is

~~~
  msgid  Click <1>here</1> to continue.
  msgstr 点击<1>这里</1>继续。
~~~

`<1>`, `</1>` and `<1/>` stand for the first inline element in the phrase,
`<2>` for the second, and so on, up to nine.  Put them wherever the target
language needs them; the original tag - URL, attributes and all - is
substituted back in.  If a sentence like that has no translation, each of
its plain-text fragments is looked up on its own, so a partial catalog is
still useful.

HTML entities are literal text in a key.  `Repository&nbsp;Size:` is matched
with the `&nbsp;` in it, and the translation is free to drop it, because the
whole text node is replaced rather than patched.

### Finding what is left

Run a server with tracing on and browse it.  Every phrase that has no
translation is written to stderr in catalog format, once each:

~~~
  FOSSIL_LOCALE_TRACE=1 fossil server myrepo.fossil 2>missing.txt
~~~

Paste the interesting records into the catalog and fill in the `msgstr`
lines.  The output does need a human pass: text that Fossil assembles from
fragments shows up as fragments, and a few entries are format-string
plumbing rather than prose.

### Adding a new language

Drop `src/locale/xx-YY.txt` next to the existing catalogs and regenerate the
makefiles so that the new file is compiled in:

~~~
  cd src
  tclsh ../tools/makemake.tcl     # or ../jimsh0, built by ./configure
~~~

`src/locale/*.txt` is already in the `extra_files` list, so nothing else has
to change.  Then `fossil setting locale xx-YY`.

## Known limits

  *  Text that Fossil composes at run time cannot be matched.  The heading
     over the timeline, for instance, is built up a fragment at a time
     (`"%d "` + event type + a plural `"s"`), so it stays in English.
     Translating it means restructuring that code upstream, not extending
     the catalog.

  *  Values that arrive as printf arguments - user names, branch names,
     check-in comments - are content, not interface, and are never
     translated.  The few argument-passed strings that *are* interface, such
     as the admin menu labels, are translated at their call site with
     `i18n_text()` or `i18n_markup()`.

  *  A catalog key is the English text.  One English word that needs two
     different translations depending on where it appears cannot express
     that yet; a context qualifier would be the natural extension.

  *  Command-line output is not translated.  Only the web interface is.
