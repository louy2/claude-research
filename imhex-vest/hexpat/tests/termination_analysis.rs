//! The static termination check (`hexpat::termination`): which loops,
//! unsized arrays and cycles it proves, and which it does not.

use hexpat::termination::{analyze, Report, Verdict};
use std::path::Path;

fn report(src: &str) -> Report {
    analyze(src, Path::new("test.hexpat"), &[]).unwrap_or_else(|e| panic!("{}\n{}", e, src))
}

/// The verdicts as "what: tag: reason", in report order.
fn lines(src: &str) -> Vec<String> {
    report(src)
        .obligations
        .iter()
        .map(|o| match &o.verdict {
            Verdict::Proven(w) => format!("{}: proven: {}", o.what, w),
            Verdict::RuntimeCheck(w) => format!("{}: run-time check: {}", o.what, w),
            Verdict::NotProven(w) => format!("{}: not proven: {}", o.what, w),
        })
        .collect()
}

#[track_caller]
fn proven(src: &str, argument: &str) {
    let r = report(src);
    let l = lines(src);
    assert!(r.all_proven(), "not all proven: {:#?}\n{}", l, src);
    assert!(l.iter().any(|x| x.contains(argument)), "no `{}` in {:#?}\n{}", argument, l, src);
}

/// An obligation that the run-time measure checks decide.
#[track_caller]
fn decided(src: &str, reason: &str) {
    let l = lines(src);
    assert!(
        l.iter().any(|x| x.contains("run-time check") && x.contains(reason)),
        "no run-time check `{}` in {:#?}\n{}",
        reason,
        l,
        src
    );
    assert!(!report(src).all_proven());
}

#[track_caller]
fn open(src: &str, reason: &str) {
    let l = lines(src);
    assert!(
        l.iter().any(|x| x.contains("not proven") && x.contains(reason)),
        "no unproven `{}` in {:#?}\n{}",
        reason,
        l,
        src
    );
}

#[test]
fn a_pattern_without_loops_or_cycles_has_no_obligations() {
    let r = report("struct H { u32 magic; u16 n; }; struct F { H h; u8 body[h.n]; }; F f @ 0;");
    assert!(r.obligations.is_empty());
    assert!(r.all_proven());
    assert_eq!(r.reachable_types, 2);
}

#[test]
fn counters() {
    proven("u32 t = 0; for (u32 i = 0, i < 10, i += 1) { t = t + i; }", "`i` moves up by 1");
    proven("u32 n = 8; u32 i = 0; while (i < n) { i = i + 2; }", "`i` moves up by 2");
    proven("fn f(str s) { s32 i = std::string::length(s); while (i > 0) { i = i - 1; } }; f(\"ab\");", "`i` moves down by 1");
    // The bound may call a function of its arguments only.
    proven(
        "fn len(str s) { return std::string::length(s); }; fn f(str s) { for (u32 i = 0, i < len(s), i += 1) { } }; f(\"ab\");",
        "`i` moves up by 1",
    );
    // One conjunct that bounds the counter is enough.
    proven("u32 i = 0; while (std::mem::read_unsigned(i, 1) != 0 && i < 4) { i += 1; }", "`i` moves up by 1");
}

#[test]
fn counters_that_do_not_convince() {
    open("u32 i = 0; while (true) { i = i + 1; }", "the condition does not bound the counter");
    open("u32 i = 0; u32 x = 5; while (x != 0) { x = x >> 1; }", "no counter with a constant step");
    // The bound moves with the counter.
    open("u32 n = 4; for (u32 i = 0, i < n, i += 1) { n = n + 1; }", "the loop assigns `n`");
    // The counter is also assigned somewhere else in the body.
    open("u32 i = 0; while (i < 4) { i = i + 1; if (i == 3) i = 0; }", "no counter");
    // A `continue` can skip the step.
    open("u32 i = 0; while (i < 4) { if (i == 2) continue; i = i + 1; }", "no counter");
    // The step goes away from the bound.
    open("for (u32 i = 0, i < 4, i -= 1) { }", "the condition does not bound the counter");
    // A function called in the loop may assign the counter.
    open("u32 i = 0; fn reset() { i = 0; }; while (i < 4) { reset(); i = i + 1; }", "may assign variables it does not own");
    // The bound reads the cursor.
    open("fn f() { for (u32 i = 0, i < $, i += 1) { } }; f();", "the bound depends on the cursor");
    // The bound calls a function that reads a global.
    open(
        "u32 g = 4; fn lim() { return g; }; u32 i = 0; while (i < lim()) { i = i + 1; }",
        "the bound calls `lim`",
    );
}

#[test]
fn arrays_progress_through_the_data() {
    proven("struct E { u8 tag; u16 len; }; E es[while(!std::mem::eof())] @ 0;", "each entry reads at least 3 byte(s)");
    proven("char s[] @ 0;", "each entry reads at least 1 byte(s)");
    // The shortest path through an entry counts.
    proven("struct E { u8 t; if (t == 1) u32 x; }; E es[while(true)] @ 0;", "at least 1 byte(s)");
    // An entry that always breaks ends the array.
    proven("struct E { break; }; E es[while(true)] @ 0;", "every entry ends the array");
    // Members of a union overlap: the union is as long as its longest
    // member, not their sum.
    proven("union U { u8 a; u16 b; }; U us[while(true)] @ 0;", "at least 2 byte(s)");
    proven("union U { u8 a; if (a == 1) { u8 b; u8 c; } }; U us[while(true)] @ 0;", "at least 1 byte(s)");
    proven("bitfield B { a : 3; b : 5; }; B bs[while(true)] @ 0;", "at least 1 byte(s)");
    // A forward skip by a constant counts as progress.
    proven("struct E { $ += 4; }; E es[while(true)] @ 0;", "at least 4 byte(s)");
}

#[test]
fn arrays_that_may_stand_still() {
    decided("struct E { }; E es[while(true)] @ 0;", "an entry can be empty");
    // No arm may match.
    decided("struct E { u8 t = 1; if (t == 0) u8 a; else if (t == 2) u16 b; }; E es[while(true)] @ 0;", "an entry can be empty");
    decided("struct E { u8 t = 1; match (t) { (0): u8 a; } }; E es[while(true)] @ 0;", "an entry can be empty");
    // `continue` drops the entry but the array goes on.
    decided("struct E { continue; u8 a; }; E es[while(true)] @ 0;", "an entry can be empty");
    // A local, a placed member and a no_unique_address member read nothing.
    decided("struct E { u8 a = 1; u8 b @ 0; u8 c [[no_unique_address]]; }; E es[while(true)] @ 0;", "an entry can be empty");
    // Peek and step back.
    decided("struct E { u8 a; $ -= 1; }; E es[while(true)] @ 0;", "may move the cursor back");
    // Every such array is still guarded: the interpreter checks each entry.
    assert!(report("struct E { }; E es[while(true)] @ 0;").guarded());
    decided("struct E { u8 a; $ = a; }; E es[while(true)] @ 0;", "may move the cursor back");
    // ... also when a function or a nested type does it.
    decided("fn back() { $ = 0; }; struct E { u8 a; back(); }; E es[while(true)] @ 0;", "may move the cursor back");
    decided("struct R { $ = 0; }; struct E { u8 a; R r; }; E es[while(true)] @ 0;", "may move the cursor back");
    // try/catch restores the cursor when the body fails.
    decided("struct E { try { u8 a; } catch { } }; E es[while(true)] @ 0;", "an entry can be empty");
}

#[test]
fn arrays_of_a_template_parameter() {
    proven(
        "struct L<T> { T data[while(std::mem::read_unsigned($, 1) != 0)]; }; struct E { u16 x; }; L<char> a @ 0; L<E> b @ 0;",
        "for every type the template is used with",
    );
    decided("struct L<T> { T data[while(true)]; }; struct Z { }; L<u8> a @ 0; L<Z> b @ 0;", "with Z an entry can be empty");
    // The parameter is passed on through another template.
    decided(
        "struct L<T> { T data[while(true)]; }; struct W<T> { L<T> l; }; struct Z { }; W<Z> w @ 0;",
        "with Z an entry can be empty",
    );
}

#[test]
fn loops_in_a_struct_body() {
    proven("struct S { u8 n; while (std::mem::read_unsigned($, 1) != 0) { u8 b; } }; S s @ 0;", "each round reads at least 1 byte(s)");
    open("struct S { u8 n; while (n != 0) { u8 b @ 0; } }; S s @ 0;", "a round can read nothing");
    // In a function nothing is read, so only a counter helps.
    open("fn f() { while (std::mem::read_unsigned($, 1) != 0) { } }; f();", "no counter");
    proven("fn f() { while ($ < 16) { $ += 1; } }; f();", "the cursor moves up by 1");
}

#[test]
fn inline_recursion_needs_a_byte_between_instances() {
    // A tree: every node reads its tag before its children.
    proven(
        "struct Node { u8 kind; if (kind == 1) { u8 n; Node kids[n]; } }; Node root @ 0;",
        "at least one byte is read between a type and its next instance",
    );
    // Two types; one of the two edges has progress.
    proven(
        "using Value; struct Entry { u8 tag; if (tag == 0) break; Value v; }; struct Value { Entry es[while(true)]; }; Value v @ 0;",
        "at least one byte is read",
    );
    // A forward declaration and inheritance, as in the mp4 pattern.
    proven(
        "struct Base { u32 size; char type[4]; }; using Box; struct Container : Base { Box kids[while($ < size)]; }; \
         struct Leaf : Base { u8 x; }; struct Box { u32 t = std::mem::read_unsigned($ + 4, 4); \
         match (t) { (1): Leaf box; (_): Container box; } }; Box all[while(!std::mem::eof())] @ 0;",
        "each entry reads at least 8 byte(s)",
    );
    // Without a byte in between, the run-time check decides: an instance
    // inside itself at the same offset is refused.
    decided("struct S { S s; u8 a; }; S s @ 0;", "can reach itself without reading a byte first");
    decided("union U { u8 a; U u; }; U u @ 0;", "can reach itself without reading a byte first");
    decided("struct S { u8 a; $ = 0; S s; }; S s @ 0;", "may move the cursor back");
    // The recursion goes through a template argument.
    decided("struct W<T> { T inner; }; struct N { W<N> w; }; N n @ 0;", "can reach itself without reading a byte first");
    // Cycles that put no instance at the cursor are left to the depth limit.
    open("struct A : A { u8 x; }; A a @ 0;", "inherits from itself or is an alias of itself");
    open("using B; using A = B; using B = A; A x @ 0;", "inherits from itself or is an alias of itself");
    open("struct T<auto n> { T<n + 1> x; }; T<0> t @ 0;", "a template argument is computed anew at each level");
    open("struct S { u32 n = sizeof(S); }; S s @ 0;", "evaluates a type away from the cursor");
    // A parent inside the cycle is fine when a member closes it.
    decided("using Node; struct Base { Node child; }; struct Node : Base { u8 x; }; Node n @ 0;", "can reach itself without reading");
    proven("struct W<T> { u8 tag; if (tag == 1) T inner; }; struct N { W<N> w; }; N n @ 0;", "at least one byte is read");
}

#[test]
fn cycles_through_placements_need_a_run_time_check() {
    for src in [
        "struct N { u8 v; N *next : u8; }; N n @ 0;",
        "struct S { u8 a; S s @ a; }; S s @ 0;",
        "using B; struct A { u8 x; B b @ x; }; struct B { u8 y; A *a : u8; }; A a @ 0;",
    ] {
        let l = lines(src);
        assert!(l.iter().any(|x| x.contains("run-time check")), "{:#?}\n{}", l, src);
        let r = report(src);
        assert!(!r.all_proven() && r.guarded());
    }
    // Without a cycle, placements and pointers need nothing: every jump goes
    // to a type further down.
    let r = report("struct Leaf { u8 x; }; struct Dir { u8 off; Leaf l @ off; Leaf *p : u8; }; Dir d @ 0;");
    assert!(r.obligations.is_empty() && r.all_proven());
}

#[test]
fn recursive_functions_are_not_proven() {
    open("fn f(u32 x) { if (x == 0) return 0; return f(x - 1); }; u32 y = f(3);", "recursive function");
    // Through a format attribute.
    open("fn fmt(auto v) { return fmt(v); }; u8 a @ 0 [[format(\"fmt\")]];", "recursive function");
    // A function and a type that reach each other.
    open("fn g() { S s @ 0; }; struct S { u8 a; g(); }; S s @ 0;", "types and functions that reach each other");
    assert!(!report("fn f(u32 x) { return f(x); }; u32 y = f(1);").guarded());
}

#[test]
fn only_reachable_code_is_judged() {
    let r = report("fn unused() { while (true) { } }; struct Unused { Unused u; }; u8 a @ 0;");
    assert!(r.obligations.is_empty());
    assert_eq!((r.reachable_types, r.reachable_functions), (0, 0));
}

#[test]
fn unresolved_names_are_reported() {
    let r = report("Missing m @ 0;");
    assert_eq!(r.notes, vec!["unknown type `Missing`".to_string()]);
    assert!(!r.all_proven());
}

#[test]
fn the_bundled_formats_are_fully_proven() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../formats");
    let mut seen = 0;
    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().map(|e| e != "hexpat").unwrap_or(true) {
            continue;
        }
        let src = std::fs::read_to_string(&path).unwrap();
        let r = analyze(&src, &path, &[]).unwrap_or_else(|e| panic!("{}: {}", path.display(), e));
        assert!(r.all_proven(), "{}: {:#?} {:?}", path.display(), r.obligations, r.notes);
        seen += 1;
    }
    assert_eq!(seen, 10);
}
