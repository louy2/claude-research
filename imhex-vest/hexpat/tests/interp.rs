//! Self-contained interpreter tests: small patterns evaluated against
//! synthetic byte strings, checking offsets, sizes and values.

use hexpat::interp::dump::DumpOptions;
use hexpat::{PatternKind, Runtime, Value};

fn run(src: &str, data: &[u8]) -> Runtime {
    let mut rt = Runtime::new(data.to_vec());
    rt.run_source(src, None).unwrap_or_else(|e| panic!("pattern failed: {}\n{}", e, src));
    rt
}

fn member(rt: &Runtime, path: &str) -> hexpat::PatternRef {
    let mut parts = path.split('.');
    let root_name = parts.next().unwrap();
    let mut cur = rt
        .patterns
        .iter()
        .find(|p| p.borrow().name == root_name)
        .unwrap_or_else(|| panic!("no top-level pattern {}", root_name))
        .clone();
    for part in parts {
        let next = if let Some(idx) = part.strip_prefix('[') {
            let i: u64 = idx.trim_end_matches(']').parse().unwrap();
            let entries = match &cur.borrow().kind {
                PatternKind::Array { entries } => entries.clone(),
                _ => panic!("{} is not a materialised array", part),
            };
            entries[i as usize].clone()
        } else {
            cur.borrow().member(part).unwrap_or_else(|| panic!("no member {} in {}", part, path))
        };
        cur = next;
    }
    cur
}

fn value(rt: &Runtime, path: &str) -> Value {
    rt.pattern_value(&member(rt, path)).unwrap()
}

fn uint(rt: &Runtime, path: &str) -> u128 {
    value(rt, path).as_u128().unwrap()
}

#[test]
fn struct_with_scalars_and_endianness() {
    let src = "struct H { u8 a; le u16 b; be u32 c; s8 d; float f; }; H h @ 0;";
    let rt = run(src, &[0x01, 0x34, 0x12, 0xDE, 0xAD, 0xBE, 0xEF, 0xFF, 0x00, 0x00, 0xC0, 0x3F]);
    assert_eq!(uint(&rt, "h.a"), 1);
    assert_eq!(uint(&rt, "h.b"), 0x1234);
    assert_eq!(uint(&rt, "h.c"), 0xDEADBEEF);
    assert_eq!(value(&rt, "h.d").as_i128().unwrap(), -1);
    assert_eq!(value(&rt, "h.f").as_f64().unwrap(), 1.5);
    let h = member(&rt, "h");
    assert_eq!(h.borrow().size, 12);
    assert_eq!(member(&rt, "h.f").borrow().offset, 8);
}

#[test]
fn pragma_endian_and_nested_structs() {
    let src = "#pragma endian big\nstruct In { u16 x; }; struct Out { In a; In b; u32 y; }; Out o @ 2;";
    let rt = run(src, &[0, 0, 0x00, 0x01, 0x00, 0x02, 0, 0, 0, 3]);
    assert_eq!(uint(&rt, "o.a.x"), 1);
    assert_eq!(uint(&rt, "o.b.x"), 2);
    assert_eq!(uint(&rt, "o.y"), 3);
    assert_eq!(member(&rt, "o").borrow().offset, 2);
    assert_eq!(member(&rt, "o").borrow().size, 8);
}

#[test]
fn arrays_fixed_unsized_and_while() {
    let src = r#"
        struct S { u8 n; u16 vals[n]; char name[]; u8 rest[while($ < 12)]; };
        S s @ 0;
    "#;
    let data = [2u8, 0x10, 0x00, 0x20, 0x00, b'h', b'i', 0, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE];
    let rt = run(src, &data);
    assert_eq!(member(&rt, "s.vals").borrow().entry_count(), Some(2));
    assert_eq!(member(&rt, "s.vals").borrow().size, 4);
    assert_eq!(value(&rt, "s.name").as_str().unwrap(), "hi");
    assert_eq!(member(&rt, "s.name").borrow().size, 3);
    assert_eq!(member(&rt, "s.rest").borrow().entry_count(), Some(4));
    assert_eq!(member(&rt, "s").borrow().size, 12);
    let mut rt = rt;
    let arr = member(&rt, "s.vals");
    let e1 = rt.array_element(&arr, 1).unwrap();
    assert_eq!(rt.pattern_value(&e1).unwrap().as_u128().unwrap(), 0x20);
}

#[test]
fn enums_and_scoped_values() {
    let src = r#"
        enum Kind : u8 { A = 1, B, C = 10 ... 20 };
        struct S { Kind k; if (k == Kind::B) u8 extra; };
        S s @ 0;
    "#;
    let mut rt = run(src, &[2, 7]);
    assert_eq!(uint(&rt, "s.k"), 2);
    assert_eq!(uint(&rt, "s.extra"), 7);
    let k = member(&rt, "s.k");
    assert_eq!(rt.formatted_value(&k).unwrap(), "Kind::B");
    let rt2 = run(src, &[15]);
    let k = member(&rt2, "s.k");
    let mut rt2 = rt2;
    assert_eq!(rt2.formatted_value(&k).unwrap(), "Kind::C");
    assert!(member(&rt2, "s").borrow().member("extra").is_none());
}

#[test]
fn bitfields_least_and_most_significant_first() {
    let src = r#"
        bitfield Flags { a : 3; b : 1; padding : 2; c : 2; };
        bitfield Big { hi : 4; lo : 12; } [[bitfield_order(std::core::BitfieldOrder::MostToLeastSignificant, 16)]];
        Flags f @ 0;
        Big g @ 1;
    "#;
    // 0b10_00_1_101 = 0x8D: a=5, b=1, c=2
    let rt = run(src, &[0x8D, 0xCD, 0xAB]);
    assert_eq!(uint(&rt, "f.a"), 5);
    assert_eq!(uint(&rt, "f.b"), 1);
    assert_eq!(uint(&rt, "f.c"), 2);
    assert_eq!(member(&rt, "f").borrow().size, 1);
    // Little-endian 16-bit integer 0xABCD: top nibble 0xA, low 12 bits 0xBCD.
    assert_eq!(uint(&rt, "g.hi"), 0xA);
    assert_eq!(uint(&rt, "g.lo"), 0xBCD);
    assert_eq!(member(&rt, "g").borrow().size, 2);
}

#[test]
fn unions_and_padding() {
    let src = "union U { u8 b; u32 w; }; struct S { U u; padding[2]; u8 t; }; S s @ 0;";
    let rt = run(src, &[1, 2, 3, 4, 0, 0, 9]);
    assert_eq!(uint(&rt, "s.u.b"), 1);
    assert_eq!(uint(&rt, "s.u.w"), 0x04030201);
    assert_eq!(member(&rt, "s.u").borrow().size, 4);
    assert_eq!(uint(&rt, "s.t"), 9);
    assert_eq!(member(&rt, "s").borrow().size, 7);
}

#[test]
fn pointers_follow_addresses() {
    let src = "struct T { u16 v; }; struct S { T *p : u8; }; S s @ 0;";
    let rt = run(src, &[3, 0, 0, 0x34, 0x12]);
    let p = member(&rt, "s.p");
    let pointee = match &p.borrow().kind {
        PatternKind::Pointer { pointee, address } => {
            assert_eq!(*address, 3);
            pointee.clone()
        }
        _ => panic!("not a pointer"),
    };
    assert_eq!(rt.pattern_value(&pointee.borrow().member("v").unwrap()).unwrap().as_u128().unwrap(), 0x1234);
    assert_eq!(member(&rt, "s").borrow().size, 1);
}

#[test]
fn match_statement_with_ranges_and_alternatives() {
    let src = r#"
        struct S {
            u8 tag;
            match (tag) {
                (1 | 2): u8 small;
                (10 ... 20): u16 mid;
                (_): u32 big;
            }
        };
        S a @ 0; S b @ 2; S c @ 5;
    "#;
    let rt = run(src, &[2, 0xAA, 15, 0xCD, 0xAB, 99, 1, 0, 0, 0]);
    assert_eq!(uint(&rt, "a.small"), 0xAA);
    assert_eq!(uint(&rt, "b.mid"), 0xABCD);
    assert_eq!(uint(&rt, "c.big"), 1);
}

#[test]
fn functions_locals_and_control_flow() {
    let src = r#"
        fn fib(u32 n) {
            if (n < 2) return n;
            return fib(n - 1) + fib(n - 2);
        };
        fn sum_to(u32 n) {
            u32 acc = 0;
            for (u32 i = 1, i <= n, i += 1) acc += i;
            return acc;
        };
        u32 result = fib(10) + sum_to(4);
        struct S { u8 x[result]; };
        S s @ 0;
    "#;
    let rt = run(src, &[0u8; 80]);
    assert_eq!(member(&rt, "s.x").borrow().entry_count(), Some(65));
}

#[test]
fn strings_format_and_std_functions() {
    let src = r#"
        struct S { char magic[4]; u8 n; };
        S s @ 0;
        std::assert(s.magic == "RIFF", "bad magic");
        std::print("n={} hex={:02X} m={}", s.n, s.n, s.magic);
        str t = std::format("{}-{}", std::string::to_upper("ab"), std::string::length(s.magic));
        std::print(t);
        std::assert(std::mem::size() == 6, "size");
        std::assert(std::mem::read_unsigned(4, 1) == 7, "read");
        std::assert(std::mem::read_string(0, 2) == "RI", "read_string");
    "#;
    let rt = run(src, b"RIFF\x07\x00");
    assert_eq!(rt.console, vec!["n=7 hex=07 m=RIFF".to_string(), "AB-4".to_string()]);
}

#[test]
fn templates_using_and_namespaces() {
    let src = r#"
        namespace ns {
            struct Pair<T> { T a; T b; };
            using Words = Pair<u16>;
            fn double(auto x) { return x * 2; };
        }
        struct Magic<auto Expected> {
            char value[std::string::length(Expected)];
            std::assert(value == Expected, "magic mismatch");
        };
        struct S { Magic<"AB"> m; ns::Words w; u8 c[ns::double(2)]; };
        S s @ 0;
    "#;
    let rt = run(src, &[b'A', b'B', 1, 0, 2, 0, 9, 9, 9, 9]);
    assert_eq!(uint(&rt, "s.w.a"), 1);
    assert_eq!(uint(&rt, "s.w.b"), 2);
    assert_eq!(member(&rt, "s.c").borrow().entry_count(), Some(4));
    assert_eq!(member(&rt, "s").borrow().size, 10);
}

#[test]
fn parent_this_dollar_sizeof_and_addressof() {
    let src = r#"
        struct Body { u8 len; u8 data[parent.hdr_len - 1]; };
        struct Pkt {
            u8 hdr_len;
            Body body;
            u8 tail[addressof(body) + sizeof(body) - $ + 2];
            u32 total = sizeof(this);
        };
        Pkt p @ 0;
    "#;
    let rt = run(src, &[4, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    assert_eq!(member(&rt, "p.body.data").borrow().entry_count(), Some(3));
    assert_eq!(member(&rt, "p.tail").borrow().entry_count(), Some(2));
    assert_eq!(member(&rt, "p").borrow().size, 7);
    assert_eq!(uint(&rt, "p.total"), 7);
}

#[test]
fn break_inside_struct_stops_array() {
    let src = r#"
        struct Rec { u8 tag; if (tag == 0xFF) break; u8 v; };
        Rec recs[while(!std::mem::eof())] @ 0;
    "#;
    let rt = run(src, &[1, 10, 2, 20, 0xFF, 3, 30]);
    assert_eq!(member(&rt, "recs").borrow().entry_count(), Some(3));
    assert_eq!(member(&rt, "recs").borrow().size, 5);
}

#[test]
fn attributes_name_comment_hidden_and_format() {
    let src = r#"
        fn hexfmt(u16 v) { return std::format("0x{:04X}", v); };
        struct S {
            u16 a [[name("first"), comment("c"), format("hexfmt")]];
            u8 b [[hidden]];
        } [[name("Renamed")]];
        S s @ 0;
    "#;
    let mut rt = run(src, &[0x34, 0x12, 0]);
    let s = member(&rt, "s");
    assert_eq!(s.borrow().display_name.as_deref(), Some("Renamed"));
    let a = s.borrow().member("a").unwrap();
    assert_eq!(a.borrow().display_name.as_deref(), Some("first"));
    assert_eq!(a.borrow().comment.as_deref(), Some("c"));
    assert_eq!(rt.formatted_value(&a).unwrap(), "0x1234");
    assert!(s.borrow().member("b").unwrap().borrow().hidden);
    let text = hexpat::interp::dump_text(&mut rt, &DumpOptions { formatted: true, show_hidden: false, max_entries: 0 });
    assert!(text.contains("first: u16 @ 0x0 [2] = 0x1234"));
    assert!(!text.contains("b: u8"));
}

#[test]
fn transform_attribute_makes_struct_usable_as_integer() {
    let src = r#"
        fn as_int(ref auto x) { return x.lo | (x.hi << 8); };
        struct Split { u8 lo; u8 hi; } [[transform("as_int")]];
        struct S { Split n; u8 body[n]; };
        S s @ 0;
    "#;
    let rt = run(src, &[3, 0, 1, 2, 3, 4]);
    assert_eq!(member(&rt, "s.body").borrow().entry_count(), Some(3));
}

#[test]
fn placement_in_struct_does_not_move_cursor() {
    let src = "struct S { u8 off; u16 far @ off; u8 next; }; S s @ 0;";
    let rt = run(src, &[4, 9, 0, 0, 0x22, 0x11]);
    assert_eq!(uint(&rt, "s.far"), 0x1122);
    assert_eq!(uint(&rt, "s.next"), 9);
    assert_eq!(member(&rt, "s").borrow().size, 2);
}

#[test]
fn try_catch_recovers_and_errors_propagate() {
    let src = r#"
        struct S { try { u8 a; std::error("boom"); } catch { u8 b; } };
        S s @ 0;
    "#;
    let rt = run(src, &[1, 2]);
    assert_eq!(uint(&rt, "s.b"), 1);
    let mut rt = Runtime::new(vec![0; 2]);
    let err = rt.run_source("struct S { u32 x; }; S s @ 0;", None).unwrap_err();
    assert!(err.message.contains("out of bounds"), "{}", err);
}

#[test]
fn out_of_bounds_reads_are_errors_but_std_reads_are_lenient() {
    let src = "u8 last = std::mem::read_unsigned(100, 1); std::assert(last == 0, \"lenient\"); u8 x @ 0;";
    let rt = run(src, &[5]);
    assert_eq!(uint(&rt, "x"), 5);
}

#[test]
fn every_integer_read_uses_vest_primitives() {
    // Exercises u24/u48/u96/u128 and signed widths through the reader.
    let src = "struct S { u24 a; be u48 b; u96 c; s16 d; s64 e; }; S s @ 0;";
    let mut data = vec![0x01, 0x02, 0x03, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];
    data.extend([0u8; 12]);
    data.extend([0xFE, 0xFF]);
    data.extend([0xFF; 8]);
    let rt = run(src, &data);
    assert_eq!(uint(&rt, "s.a"), 0x030201);
    assert_eq!(uint(&rt, "s.b"), 0xAABBCCDDEEFF);
    assert_eq!(uint(&rt, "s.c"), 0);
    assert_eq!(value(&rt, "s.d").as_i128().unwrap(), -2);
    assert_eq!(value(&rt, "s.e").as_i128().unwrap(), -1);
    assert_eq!(member(&rt, "s").borrow().size, 3 + 6 + 12 + 2 + 8);
}
