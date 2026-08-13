# Fossil SCM — 简体中文版 / Simplified Chinese fork

这是 [Fossil SCM](https://fossil-scm.org/) 的一个分支，把它的 **Web 界面**
翻译成了简体中文。除翻译机制本身之外，不改变 Fossil 的任何行为。

This is a fork of [Fossil SCM](https://fossil-scm.org/) whose **web
interface** is translated into Simplified Chinese.  Apart from the
translation machinery itself, nothing about Fossil's behaviour changes.

* 上游基线 / upstream baseline: Fossil 2.29, check-in `7a40eb9748`
  (git mirror commit `6e77be14ccbb80a33e917bcd3f77461cccdf9c4f`)
* 上游许可证 / upstream licence: 2-Clause BSD, see `COPYRIGHT-BSD2.txt`

## 构建 / Building

和上游完全一样 / exactly as upstream:

~~~
  ./configure && make
~~~

翻译目录会被编译进可执行文件，没有运行时依赖。
The translation catalog is compiled into the executable; there is no
runtime dependency on any external file.

## 使用 / Using it

本分支默认使用简体中文。要切回英文：
This fork ships with Chinese as the default.  To go back to English:

~~~
  fossil setting locale en          # 本仓库改回英文
  FOSSIL_LOCALE=en fossil ui repo   # 只对本次运行生效
~~~

查看可用的语言 / list the available languages:

~~~
  fossil test-locale list
~~~

## 翻译是怎么做的 / How the translation works

翻译发生在 HTML 输出的最后一步，而不是修改上游的每一处字符串。
`src/i18n.c` 在 `cgi_printf()` 输出时扫描 HTML，只替换与词条**完全匹配**
的文本节点；没有词条的文本原样输出。因此页面可能只翻译了一部分，但绝不会
被破坏。

Translation happens on the way out rather than by forking every string
upstream.  `src/i18n.c` scans the HTML as `cgi_printf()` emits it and
replaces a text node only when it matches a catalog entry exactly.  Anything
the catalog does not know about is emitted byte for byte as upstream wrote
it, so a page may be partly translated but is never mangled.

* 词条文件 / catalog: `src/locale/zh-CN.txt`
* 机制说明 / design notes: `www/localization.md`
* 校验 / lint the catalog: `fossil test-locale check zh-CN`
* 找出未翻译的文本 / find untranslated text:
  `FOSSIL_LOCALE_TRACE=1 fossil server repo 2>missing.txt`

已翻译的范围：主菜单、页面标题、时间线、文件浏览、签入与工件信息、分支与
标签、差异与逐行追溯、维基、论坛、聊天、工单、搜索、统计、登录与用户管理，
以及管理页面的绝大部分。少量运行时拼接而成的文字（例如时间线上方的
“2 check-ins” 标题）受机制限制仍为英文，详见 `www/localization.md`。

Translated: main menu, page titles, timeline, file browser, check-in and
artifact information, branches and tags, diff and annotation, wiki, forum,
chat, tickets, search, statistics, login and user administration, and most
of the administration pages.  A small amount of text that Fossil composes at
run time (such as the "2 check-ins" heading above the timeline) is out of
reach of this mechanism and stays in English; see `www/localization.md`.

## 与上游的差异 / Differences from upstream

| 文件 / file | 说明 / what changed |
| --- | --- |
| `src/i18n.c` | 新增：翻译引擎、`locale` 设置、`test-locale` 命令 |
| `src/locale/zh-CN.txt` | 新增：简体中文词条 |
| `src/cgi.c` | `cgi_printf()`／`cgi_vprintf()` 经由翻译层输出 |
| `src/style.c` | 页面标题、子菜单、主菜单标签、`<html lang>` |
| `src/th_main.c` | 皮肤模板文字与 TH1 `html` 命令的输出 |
| `src/setup.c` | 管理主页的菜单项 |
| `src/timeline.c` | 时间线每行的 `leaf`／`closed` 前缀 |
| `src/builtin.c` | 新增两个访问内置文件清单的函数 |
| `tools/makemake.tcl` | 把 `i18n.c` 和 `locale/*.txt` 加入构建 |

在英文模式下，本分支的输出与上游逐字节一致（`<html>` 上的 `lang` 属性除外）。

In English mode this fork's output is byte-for-byte identical to upstream,
apart from the `lang` attribute on `<html>`.
