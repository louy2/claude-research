//! Every evaluation stops: patterns that would run forever end with an
//! error from one of the limits in `hexpat::interp::Limits`.

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

fn defaults(_: &mut Runtime) {}

fn small(rt: &mut Runtime) {
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
