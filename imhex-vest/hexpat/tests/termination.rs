//! Every evaluation stops. The first half runs with the measure checks
//! off: patterns that would run forever end with an error from one of the
//! limits in `hexpat::interp::Limits`. The second half shows the measure
//! checks, which are on by default and stop the same patterns where the
//! problem happens.

use hexpat::Runtime;

/// Evaluation recurses once per nesting level, so run it the way the CLI
/// does, on a thread with a large stack.
fn on_big_stack<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    std::thread::Builder::new().stack_size(256 << 20).spawn(f).unwrap().join().unwrap()
}

/// Runs `src` over four zero bytes and returns the error message.
fn error_of(src: &'static str, tune: fn(&mut Runtime)) -> String {
    on_big_stack(move || {
        let mut rt = Runtime::new(vec![0; 4]);
        tune(&mut rt);
        match rt.run_source(src, None) {
            Ok(()) => panic!("pattern finished without an error:\n{}", src),
            Err(e) => e.to_string(),
        }
    })
}

/// Default limits, without the measure checks.
fn defaults(rt: &mut Runtime) {
    rt.limits.check_measures = false;
}

fn checked(_: &mut Runtime) {}

fn small(rt: &mut Runtime) {
    rt.limits.check_measures = false;
    rt.limits.max_steps = 10_000;
    rt.limits.max_loop_iterations = 1_000;
    rt.limits.max_array_entries = 1_000;
    rt.limits.max_patterns = 5_000;
}

#[track_caller]
fn assert_stops(src: &'static str, tune: fn(&mut Runtime), expected: &str) {
    let e = error_of(src, tune);
    assert!(e.contains(expected), "expected `{}` in `{}` for:\n{}", expected, e, src);
}

#[test]
fn types_that_reach_themselves_hit_the_depth_limit() {
    let creating = "recursion limit reached creating";
    // A pointer cycle: the zero byte points back at offset 0.
    assert_stops("struct N { u8 v; N *next : u8; }; N n @ 0;", defaults, creating);
    // A struct that places itself.
    assert_stops("struct S { u8 a; S s @ 0; }; S s @ 0;", defaults, creating);
    // A struct or union that contains itself without reading anything first.
    assert_stops("struct S { S s; }; S s @ 0;", defaults, creating);
    assert_stops("union U { u8 a; U u; }; U u @ 0;", defaults, creating);
    assert_stops("struct S { S a[2] @ 0; }; S s @ 0;", defaults, creating);
    // Two types that reach each other.
    assert_stops(
        "using B; struct A { u8 x; B b @ 0; }; struct B { u8 y; A *a : u8; }; A a @ 0;",
        defaults,
        creating,
    );
    // A template that instantiates itself with a new argument each time.
    assert_stops("struct T<auto n> { T<n + 1> x; }; T<0> t @ 0;", defaults, creating);
    // sizeof evaluates the type in a scratch section.
    assert_stops("struct S { S s; }; u32 n = sizeof(S);", defaults, creating);
}

#[test]
fn alias_and_inheritance_cycles_hit_the_depth_limit() {
    assert_stops("using B; using A = B; using B = A; A x @ 0;", defaults, "recursion limit reached resolving alias");
    assert_stops("struct A : A { u8 x; }; A a @ 0;", defaults, "recursion limit reached inheriting from");
}

#[test]
fn function_recursion_hits_the_depth_limit() {
    assert_stops("fn f(u32 x) { return f(x); }; u32 y = f(1);", defaults, "recursion limit reached calling `f`");
    // In a transform, which runs when the value is used.
    assert_stops(
        "fn t(auto v) { return t(v); }; u8 a @ 0 [[transform(\"t\")]]; u8 b @ a;",
        defaults,
        "recursion limit reached calling `t`",
    );
}

#[test]
fn nesting_below_the_limit_is_allowed() {
    // 100 nodes, each pointing at the next byte; the last one points at itself
    // and is not followed.
    on_big_stack(|| {
        let mut data: Vec<u8> = (1..=100).collect();
        data[99] = 99;
        let mut rt = Runtime::new(data);
        rt.run_source("struct N { if ($ < 99) N *next : u8; else u8 last; }; N n @ 0;", None).unwrap();
    });
    // The depth is the current nesting, not a total: 1000 calls in a row.
    on_big_stack(|| {
        let mut rt = Runtime::new(vec![0; 4]);
        rt.run_source("fn f(u32 x) { return x + 1; }; u32 t = 0; for (u32 i = 0, i < 1000, i = i + 1) { t = f(t); }", None)
            .unwrap();
    });
}

#[test]
fn loops_hit_the_iteration_limit() {
    assert_stops("u32 i = 0; while (true) { i = i + 1; }", small, "while loop exceeded the iteration limit");
    assert_stops("for (u32 i = 0, true, i = i + 1) { }", small, "for loop exceeded the iteration limit");
}

#[test]
fn arrays_that_do_not_advance_hit_a_limit() {
    // Elements of size zero.
    assert_stops("struct E { }; E es[while(true)] @ 0;", small, "array entry limit exceeded");
    // Elements that move the cursor back.
    assert_stops("struct E { u8 x; $ = 0; }; E es[while(true)] @ 0;", small, "limit exceeded");
}

#[test]
fn nested_loops_hit_the_step_limit() {
    // Each loop stays under the per-loop limit; together they do not finish.
    assert_stops(
        "u32 i = 0; while (i < 900) { u32 j = 0; while (j < 900) { j = j + 1; } i = i + 1; }",
        small,
        "step limit exceeded",
    );
    // Recursion that stays shallow but branches: 2^60 calls.
    assert_stops(
        "fn f(u32 d) { if (d == 0) return 1; return f(d - 1) + f(d - 1); }; u32 y = f(60);",
        small,
        "step limit exceeded",
    );
    // sizeof does not count towards the pattern limit, but it does count steps.
    assert_stops(
        "struct E { }; struct A { E e[100]; }; u64 t = 0; while (true) { t = t + sizeof(A); }",
        |rt| {
            small(rt);
            rt.limits.max_loop_iterations = u64::MAX;
        },
        "step limit exceeded",
    );
}

#[test]
fn try_catch_cannot_swallow_the_step_limit() {
    // Each level catches the failure of the one below and tries again.
    let src = "struct S { try { S a @ 0; } catch { S b @ 0; } try { S c @ 0; } catch { S d @ 0; } }; S s @ 0;";
    let e = error_of(src, |rt| {
        rt.limits.check_measures = false;
        rt.limits.max_steps = 10_000;
        rt.limits.max_patterns = u64::MAX;
        rt.limits.max_recursion = 64;
    });
    assert!(e.contains("step limit exceeded"), "{}", e);
    // A loop that catches every error from its body.
    assert_stops(
        "fn boom() { std::error(\"x\"); }; while (true) { try { boom(); } catch { } }",
        |rt| {
            small(rt);
            rt.limits.max_loop_iterations = u64::MAX;
        },
        "step limit exceeded",
    );
}

#[test]
fn built_strings_are_bounded() {
    // Doubling a string 60 times would need 2^60 bytes.
    assert_stops(
        "str s = \"a\"; for (u32 i = 0, i < 60, i = i + 1) { s = s + s; }",
        defaults,
        "exceeds the string size limit",
    );
    assert_stops("str s = \"ab\" * 0xFFFFFFFFFF;", defaults, "exceeds the string size limit");
    // A huge field width is clamped.
    on_big_stack(|| {
        let mut rt = Runtime::new(vec![0; 4]);
        rt.run_source("std::print(\"{}\", std::string::length(std::format(\"{:99999999999}\", 1)));", None).unwrap();
        assert_eq!(rt.console, vec!["4096".to_string()]);
    });
}

// ---------------------------------------------------------------------------
// The measure checks (`Limits::check_measures`, on by default).

/// Runs `src` over `data` with the default settings; it must succeed.
fn run_ok(src: &'static str, data: Vec<u8>) {
    on_big_stack(move || {
        let mut rt = Runtime::new(data);
        rt.run_source(src, None).unwrap_or_else(|e| panic!("{}\n{}", e, src));
    });
}

#[test]
fn a_type_is_not_instantiated_inside_itself_at_the_same_offset() {
    let repeat = "inside an instance of itself at the same offset";
    // The second instance is refused; nothing runs up to the depth limit.
    assert_stops("struct N { u8 v; N *next : u8; }; N n @ 0;", checked, repeat);
    assert_stops("struct S { u8 a; S s @ 0; }; S s @ 0;", checked, repeat);
    assert_stops("struct S { S s; }; S s @ 0;", checked, repeat);
    assert_stops("union U { u8 a; U u; }; U u @ 0;", checked, repeat);
    assert_stops("using B; struct A { u8 x; B b @ 0; }; struct B { u8 y; A *a : u8; }; A a @ 0;", checked, repeat);
    // A list whose last node points back at the first.
    on_big_stack(|| {
        let mut rt = Runtime::new(vec![1, 2, 3, 0]);
        let e = rt.run_source("struct N { N *next : u8; }; N n @ 0;", None).unwrap_err().to_string();
        assert!(e.contains("`N` is instantiated at 0x0 inside an instance of itself"), "{}", e);
    });
    // Template arguments are part of the identity: the same template inside
    // itself with another argument is fine ...
    run_ok("struct Opt<T> { T v; }; Opt<Opt<u8>> o @ 0;", vec![0; 4]);
    // ... and one that takes a new argument at every level never repeats;
    // the depth limit ends it.
    assert_stops("struct T<auto n> { T<n + 1> x; }; T<0> t @ 0;", checked, "recursion limit reached creating");
    // Inheritance and alias cycles create no instance; the depth limit ends them.
    assert_stops("struct A : A { u8 x; }; A a @ 0;", checked, "recursion limit reached inheriting from");
}

#[test]
fn nested_instances_at_different_offsets_are_allowed() {
    // A tree: each node reads its child count first.
    run_ok("struct Node { u8 n; Node kids[n]; }; Node root @ 0;", vec![2, 1, 0, 0]);
    // The same type twice at one offset, side by side, is not nesting.
    run_ok("struct S { u8 a; }; union U { S x; S y; }; U u @ 0;", vec![0; 4]);
    run_ok("struct S { u8 a; }; S a @ 0; S b @ 0;", vec![0; 4]);
}

#[test]
fn array_entries_must_advance() {
    let stuck = "did not advance the cursor";
    assert_stops("struct E { }; E es[while(true)] @ 0;", checked, stuck);
    assert_stops("struct E { continue; }; E es[while(true)] @ 0;", checked, stuck);
    // An entry that reads and then goes back to where it started.
    assert_stops("struct E { u8 x; $ = 0; }; E es[while(true)] @ 0;", checked, stuck);
    assert_stops("struct E { u8 x; $ -= 1; }; E es[while(true)] @ 0;", checked, stuck);
    // The condition cannot move the cursor back either: each entry starts
    // where the previous one ended, so this array ends at the end of the data.
    assert_stops(
        "fn back() { $ = 0; return true; }; struct E { u8 x; }; E es[while(back())] @ 0;",
        checked,
        "cannot read 1 byte(s) at address 0x4",
    );
    // Peeking and stepping back is fine as long as the entry gains ground.
    run_ok(
        "struct E { u8 a; u8 b; $ -= 1; }; E es[while($ < 3)] @ 0;",
        vec![0; 4],
    );
    // The last entry may be empty if it ends the array.
    run_ok("struct E { if (std::mem::eof()) break; u8 a; }; E es[while(true)] @ 0;", vec![0; 4]);
    // An array with a count is bounded by the count.
    run_ok("struct E { }; E es[3] @ 0;", vec![0; 4]);
}

#[test]
fn an_array_that_ends_by_a_counter_needs_the_checks_off() {
    // Entries that read nothing; the condition counts them. It terminates,
    // but not by the measure for arrays, so the check refuses it.
    let src = "u32 i = 0; struct E { i += 1; }; E es[while(i < 3)] @ 0;";
    assert_stops(src, checked, "did not advance the cursor");
    on_big_stack(move || {
        let mut rt = Runtime::new(vec![0; 4]);
        rt.limits.check_measures = false;
        rt.run_source(src, None).unwrap();
    });
}

#[test]
fn the_cursor_stays_inside_the_data() {
    let past = "past the end of the data";
    assert_stops("u8 a @ 0; $ = 100;", checked, past);
    run_ok("u8 a @ 0; $ = 4;", vec![0; 4]);
    // Skipping forward without reading runs into the end instead of going on.
    assert_stops("struct E { $ += 4; }; E es[while(true)] @ 0;", checked, past);
    assert_stops("struct E { padding[3]; }; E es[while(true)] @ 0;", checked, past);
    assert_stops("struct S { u8 a; while (true) { $ += 1; } }; S s @ 0;", checked, past);
    // Reading past the end fails with or without the checks.
    assert_stops("struct E { u8 a; }; E es[while(true)] @ 0;", checked, "cannot read 1 byte(s) at address 0x4");
    assert_stops("struct E { u8 a; }; E es[while(true)] @ 0;", defaults, "cannot read 1 byte(s) at address 0x4");
    assert_stops("struct S { u8 a; S next; }; S s @ 0;", checked, "cannot read 1 byte(s) at address 0x4");
}

#[test]
fn loop_counters_do_not_wrap() {
    assert_stops("u32 t = 0; for (u8 i = 0, i < 300, i += 1) { t += 1; }", checked, "loop counter `i` went from 255 to 0 instead of up");
    assert_stops("u32 t = 0; for (u8 i = 3, i >= 0, i -= 1) { t += 1; }", checked, "loop counter `i` went from 0 to 255 instead of down");
    assert_stops("u8 i = 0; while (i < 300) { i = i + 1; }", checked, "instead of up");
    // A float that is too large for the step to change it.
    assert_stops(
        "double x = 100000000000000000.0; while (x < 200000000000000000.0) { x = x + 1; }",
        checked,
        "instead of up",
    );
    // Counters that stay in range are untouched.
    run_ok("u32 t = 0; for (u8 i = 0, i < 255, i += 1) { t += 1; } for (s8 j = 3, j >= 0, j -= 1) { t += 1; }", vec![0; 4]);
    // A counter the condition does not bound is not watched.
    run_ok("u32 t = 0; for (u8 i = 250, t < 10, i += 1) { t += 1; }", vec![0; 4]);
}
