//! `hexpat lint` (`hexpat::lint`): what each check reports and what it
//! leaves alone.

use hexpat::lint::{lint, lint_data, Finding, Level};
use hexpat::Runtime;
use std::path::Path;

fn findings(src: &str) -> Vec<Finding> {
    lint(src, Path::new("test.hexpat"), &[], false).unwrap_or_else(|e| panic!("{}\n{}", e, src))
}

/// The findings of one check, as "line: level: message".
fn check(src: &str, check: &str) -> Vec<String> {
    findings(src)
        .iter()
        .filter(|f| f.check == check)
        .map(|f| format!("{}: {}: {}", f.line, if f.level == Level::Warning { "warning" } else { "note" }, f.message))
        .collect()
}

#[track_caller]
fn warns(src: &str, name: &str, text: &str) {
    let l = check(src, name);
    assert!(l.iter().any(|x| x.contains("warning") && x.contains(text)), "no {} warning `{}` in {:#?}\n{}", name, text, l, src);
}

#[track_caller]
fn notes(src: &str, name: &str, text: &str) {
    let l = check(src, name);
    assert!(l.iter().any(|x| x.contains("note") && x.contains(text)), "no {} note `{}` in {:#?}\n{}", name, text, l, src);
}

#[track_caller]
fn quiet(src: &str, name: &str) {
    let l = check(src, name);
    assert!(l.is_empty(), "unexpected {} findings {:#?}\n{}", name, l, src);
}

#[test]
fn a_clean_pattern_has_no_findings() {
    let src = "enum Kind : u8 { A, B }; struct H { u32 magic; Kind kind; u16 n; }; struct F { H h; u8 body[h.n]; }; F f @ 0;";
    assert!(findings(src).is_empty(), "{:#?}", findings(src));
}

#[test]
fn duplicate_members_on_one_path() {
    let dup = "duplicate-member";
    warns("struct S { u8 a; u16 a; }; S s @ 0;", dup, "`a` may already be declared (line 1)");
    // Declared in a branch, and again after it.
    warns("struct S { u8 t;\nif (t == 1) u8 x;\nu8 x; }; S s @ 0;", dup, "3: warning: `x` may already be declared (line 2)");
    // In an arm of one match, and again in an arm of a later match: the
    // case of gvas.hexpat.
    warns(
        "struct S { u8 t;\nmatch (t) { (1): u8 kind; (2): u16 other; }\nmatch (t) { (3): u32 kind; } }; S s @ 0;",
        dup,
        "3: warning: `kind` may already be declared (line 2)",
    );
    // A local and a member share the names of a type body.
    warns("struct S { u32 n = 0; u8 t; if (t == 1) u32 n; }; S s @ 0;", dup, "`n` may already be declared");
    // A parent's member.
    warns("struct P { u8 id; }; struct S : P { u8 id; }; S s @ 0;", dup, "`id` may already be declared");
    // A later place is listed once.
    let l = check("struct S { u8 a;\nu8 a;\nu8 a; }; S s @ 0;", dup);
    assert_eq!(l.len(), 1, "{:#?}", l);
    assert!(l[0].contains("and at 1 more place(s): line 3"), "{}", l[0]);
}

#[test]
fn branches_that_exclude_each_other_are_not_duplicates() {
    let dup = "duplicate-member";
    quiet("struct S { u8 t; if (t == 1) u8 x; else u16 x; }; S s @ 0;", dup);
    quiet("struct S { u8 t; match (t) { (1): u8 x; (2): u16 x; (_): u32 x; } }; S s @ 0;", dup);
    // `c` and later `!c`.
    quiet("bool legacy = true; struct S { if (legacy) { u8 guid; } u8 mid; if (!legacy) { u16 guid; } }; S s @ 0;", dup);
    // Ranges of one value that do not meet.
    quiet("struct S { u16 v; if ((v >= 420) && (v < 600)) u8 rate; if (v >= 600) u16 rate; }; S s @ 0;", dup);
    warns("struct S { u16 v; if (v >= 420) u8 rate; if (v >= 600) u16 rate; }; S s @ 0;", dup, "`rate` may already be declared");
    // A branch that leaves the body.
    quiet("struct S { u8 t; if (t == 0) { u8 x; break; } u8 x; }; S es[while(true)] @ 0;", dup);
}

#[test]
fn duplicates_through_try_and_loops() {
    let dup = "duplicate-member";
    // The handler runs after the body failed: what the body declared
    // before its last statement is still there.
    warns("struct S { try { u8 a; u32 b; } catch { u8 a; } }; S s @ 0;", dup, "`a` may already be declared");
    quiet("struct S { try { u32 value; } catch { u8 value; } }; S s @ 0;", dup);
    // A member declared in a loop exists once per round.
    warns("struct S { u8 n; while ($ < 4) { u8 item; } }; S s @ 0;", dup, "`item` is declared inside a loop");
    // A local in a loop is the usual way to count.
    quiet("struct S { u8 n; u8 t; while ($ < 4) { u8 seen = 0; $ += 1; } }; S s @ 0;", dup);
}

#[test]
fn loop_counters_that_can_wrap() {
    let c = "narrow-counter";
    warns("u32 t = 0; for (u8 i = 0, i < 300, i += 1) { t += 1; }", c, "`i` is u8 and wraps at 255, but its bound can be as large as 300");
    // The bound is a wider field.
    warns(
        "struct H { u32 count; }; H h @ 0; u32 t = 0; for (u8 i = 0, i < h.count, i += 1) { t += 1; }",
        c,
        "its bound can be as large as 4294967295",
    );
    // An unknown bound, and a condition that only uses the counter: the
    // loop of ne.hexpat.
    warns("fn f(auto size) { for (u8 i = 0, i < size, i = i + 1) { } }; f(3);", c, "the condition does not keep it below that");
    warns("fn f() { for (u8 i = 0, $ + i < 1000, i = i + 1) { } }; f();", c, "the condition does not keep it below that");
    // `i <= n` with a counter as wide as n never ends when n is the largest value.
    warns("struct H { u8 n; }; H h @ 0; u32 t = 0; for (u8 i = 0, i <= h.n, i += 1) { t += 1; }", c, "wraps at 255");
    // Counting down to zero with an unsigned counter.
    warns("u32 t = 0; for (u32 i = 3, i >= 0, i -= 1) { t += 1; }", c, "`i >= 0` always holds");
    // A while loop takes the type from the declaration.
    warns("fn f(str s) { u8 index = 0; while (index < std::string::length(s)) { index = index + 1; } }; f(\"a\");", c, "`index` is u8");
}

#[test]
fn loop_counters_that_fit() {
    let c = "narrow-counter";
    quiet("u32 t = 0; for (u8 i = 0, i < 255, i += 1) { t += 1; }", c);
    quiet("struct H { u8 n; }; H h @ 0; u32 t = 0; for (u8 i = 0, i < h.n, i += 1) { t += 1; }", c);
    quiet("u32 t = 0; for (u16 i = 0, i < std::mem::read_unsigned(0, 1), i += 1) { t += 1; }", c);
    // Wide counters are left alone unless a literal shows the problem.
    quiet("fn f(auto n) { for (u32 i = 0, i < n, i += 1) { } }; f(3);", c);
    quiet("u32 t = 0; for (s32 i = 3, i >= 0, i -= 1) { t += 1; }", c);
    quiet("u32 t = 0; for (u32 i = 3, i > 0, i -= 1) { t += 1; }", c);
}

#[test]
fn offsets_and_values_in_small_variables() {
    // The loop variable of pe.hexpat.
    warns("fn f() { for (u8 cursor = $, cursor < $ + 16, cursor += 1) { } }; f();", "narrow-offset", "`cursor` is u8 but holds an offset");
    warns("fn f() { u16 end = std::mem::size() - 4; return end; }; f();", "narrow-offset", "above 65535 is cut off");
    // Data read at an offset, a remainder and a length are not offsets.
    quiet("fn f() { u8 b = $[3]; u8 pad = $ % 4; s16 v = std::mem::read_signed($, 2); return b + pad + v; }; f();", "narrow-offset");
    quiet("struct S { u32 x; u8 used = $ - addressof(this); }; S s @ 0;", "narrow-offset");
    quiet("fn f() { u64 at = $; return at; }; f();", "narrow-offset");

    warns("const u8 max_path = 1000;", "narrowing", "`max_path` is u8 and cannot hold 1000");
    notes("struct H { u32 page; }; H h @ 0; u16 size = h.page;", "narrowing", "can be as large as 4294967295");
    quiet("struct H { u32 flags; }; H h @ 0; u8 low = h.flags & 0xFF; u8 top = h.flags >> 24;", "narrowing");
}

#[test]
fn enum_dispatch_without_a_branch_for_every_value() {
    let m = "missing-case";
    let kinds = "enum Kind : u8 { A, B, C };";
    // The type reads nothing when no branch matches: a warning.
    warns(
        &format!("{kinds} struct E {{ Kind k = std::mem::read_unsigned($, 1); if (k == Kind::A) u8 a; else if (k == Kind::B) u16 b; }}; E es[while(true)] @ 0;"),
        m,
        "no branch for `C` of `Kind` and no `else`; `E` then reads nothing",
    );
    // The type reads its tag in any case: a note.
    notes(
        &format!("{kinds} struct E {{ Kind k; if (k == Kind::A) u8 a; else if (k == Kind::B) u16 b; }}; E e @ 0;"),
        m,
        "no branch for `C` of `Kind` and no `else`",
    );
    notes(&format!("{kinds} struct E {{ Kind k; match (k) {{ (Kind::A): u8 a; (Kind::B | Kind::C): u16 b; }} }}; enum Other : u8 {{ X }}; E e @ 0;"), "unused", "`Other`");
    notes(&format!("{kinds} struct E {{ Kind k; match (k) {{ (Kind::A): u8 a; (Kind::B): u16 b; }} }}; E e @ 0;"), m, "and no `(_)` arm");
    // Every value has a branch, or there is a fallback.
    quiet(&format!("{kinds} struct E {{ Kind k; if (k == Kind::A) u8 a; else if (k == Kind::B || k == Kind::C) u16 b; }}; E e @ 0;"), m);
    quiet(&format!("{kinds} struct E {{ Kind k; if (k == Kind::A) u8 a; else if (k == Kind::B) u16 b; else u32 c; }}; E e @ 0;"), m);
    quiet(&format!("{kinds} struct E {{ Kind k; match (k) {{ (Kind::A): u8 a; (_): u16 b; }} }}; E e @ 0;"), m);
    // A single test is a condition, not a dispatch.
    quiet(&format!("{kinds} struct E {{ Kind k; if (k == Kind::A) u8 a; }}; E e @ 0;"), m);
    // Two names for one value count as one.
    quiet("enum K : u8 { A = 1, B = 2, Same = 2 }; struct E { K k; if (k == K::A) u8 a; else if (k == K::B) u16 b; }; E e @ 0;", m);
}

#[test]
fn termination_and_unused_and_unresolved() {
    warns("u32 x = 5; while (x != 0) { x = x >> 1; }", "termination", "not proven to end: no counter with a constant step");
    notes("struct E { }; E es[while(true)] @ 0;", "termination", "an entry can be empty (decided at run time)");
    quiet("struct E { u8 a; }; E es[while(true)] @ 0;", "termination");

    notes("struct Used { u8 a; }; struct Spare { u8 b; }; fn helper() { return 1; }; Used u @ 0;", "unused", "type `Spare` is never used");
    notes("struct Used { u8 a; }; fn helper() { return 1; }; Used u @ 0;", "unused", "function `helper` is never called");
    // An enum used only through its constants, and a function named by an attribute.
    quiet("enum K : u8 { A }; fn show(auto v) { return \"x\"; }; u8 a @ 0 [[format(\"show\")]]; if (a == K::A) u8 b @ 1;", "unused");
    // A file that places nothing is a library.
    quiet("struct A { u8 a; }; fn f() { return 1; };", "unused");

    warns("Missing m @ 0;", "unresolved", "unknown type `Missing`");
}

fn data_findings(src: &str, data: &[u8]) -> Vec<String> {
    let mut rt = Runtime::new(data.to_vec());
    rt.run_source(src, None).unwrap_or_else(|e| panic!("{}\n{}", e, src));
    lint_data(&rt, "data.bin").iter().map(|f| f.to_string()).collect()
}

#[test]
fn coverage_and_overlap_of_a_file() {
    let l = data_findings("struct H { u16 a; u8 b; }; H h @ 0;", &[0; 3]);
    assert_eq!(l, vec!["data.bin: note[coverage]: 3 of 3 bytes are described (100.0%)"]);

    // Bytes 3-5 and the last byte are not described.
    let l = data_findings("u16 a @ 0; u8 b @ 2; u16 c @ 6;", &[0; 9]);
    assert_eq!(l, vec!["data.bin: note[coverage]: 5 of 9 bytes are described (55.6%); not described: 0x3-0x5 (3 bytes), 0x8 (1 bytes)"]);

    // Two patterns on the same bytes.
    let l = data_findings("u32 word @ 0; u8 low @ 0; u8 tail @ 4;", &[0; 5]);
    assert_eq!(l.len(), 2, "{:#?}", l);
    assert!(l[1].contains("note[overlap]: 1 byte(s) are described more than once: `word` and `low` share 0x0"), "{}", l[1]);

    // A union and a bitfield are one pattern each; a pointer's target counts.
    let l = data_findings(
        "union U { u8 a; u16 b; }; bitfield B { x : 4; y : 4; }; struct S { U u; B b; u8 *p : u8; }; S s @ 0;",
        &[0, 0, 0, 5, 0, 9],
    );
    assert_eq!(l, vec!["data.bin: note[coverage]: 5 of 6 bytes are described (83.3%); not described: 0x4 (1 bytes)"]);
}

#[test]
fn enum_values_that_are_not_entries() {
    let l = data_findings("enum K : u8 { A = 1, B = 2 }; struct S { K first; K second; }; S s @ 0;", &[1, 7]);
    assert_eq!(l.len(), 2, "{:#?}", l);
    assert_eq!(l[1], "data.bin: warning[enum-value]: `s.second` at 0x1 is 0x7, which is not an entry of `K`");
    // A range entry covers its values.
    let l = data_findings("enum K : u8 { A = 1, Rest = 2 ... 9 }; K k @ 0;", &[7]);
    assert_eq!(l.len(), 1, "{:#?}", l);
}

#[test]
fn the_bundled_formats_have_no_warnings() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../formats");
    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().map(|e| e != "hexpat").unwrap_or(true) {
            continue;
        }
        let src = std::fs::read_to_string(&path).unwrap();
        let f = lint(&src, &path, &[], false).unwrap();
        let warnings: Vec<_> = f.iter().filter(|f| f.level == Level::Warning).collect();
        assert!(warnings.is_empty(), "{}: {:#?}", path.display(), warnings);
    }
}
