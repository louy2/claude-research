//! The JSON export must match the reference implementation byte for byte.
//! The first case is the reference's own formatter test
//! (`tests/files/export/json.json` in WerWolv/PatternLanguage), with the
//! input bytes reconstructed from the expected values.

use hexpat::interp::dump_json_imhex;
use hexpat::Runtime;

fn json(src: &str, data: &[u8], meta: bool) -> String {
    let mut rt = Runtime::new(data.to_vec());
    rt.run_source(src, None).unwrap_or_else(|e| panic!("{}", e));
    dump_json_imhex(&mut rt, meta)
}

#[test]
fn matches_the_reference_export_test() {
    let src = r#"
        struct MyStruct {
            char s[];
            u8 ua;
            u16 ub;
            u32 uc;
            u48 ud;
            u64 ue;
            u128 uf;
            s8 sa;
            s16 sb;
            s32 sc;
            s48 sd;
            s64 se;
        };
        MyStruct data @ 0x0;
    "#;
    let mut data: Vec<u8> = Vec::new();
    data.extend(b"\x89PNG\r\n\x1a\n\0");
    data.push(0);
    data.extend(3328u16.to_le_bytes());
    data.extend(1380206665u32.to_le_bytes());
    data.extend(&873070592u64.to_le_bytes()[..6]);
    data.extend(16573246628824624646u64.to_le_bytes());
    data.extend(159330415869275869250811929192955402058u128.to_le_bytes());
    data.extend((-68i8).to_le_bytes());
    data.extend((-11044i16).to_le_bytes());
    data.extend((-25165923i32).to_le_bytes());
    data.extend(&29773251444219i64.to_le_bytes()[..6]);
    data.extend((-1463797564129820304i64).to_le_bytes());

    let expected = r#"{
    "data": {
        "s": "\u0089PNG\r\n\u001a\n\u0000",
        "ua": 0,
        "ub": 3328,
        "uc": 1380206665,
        "ud": 873070592,
        "ue": 16573246628824624646,
        "uf": 159330415869275869250811929192955402058,
        "sa": -68,
        "sb": -11044,
        "sc": -25165923,
        "sd": 29773251444219,
        "se": -1463797564129820304
    }
}"#;
    assert_eq!(json(src, &data, false).trim(), expected.trim());
}

#[test]
fn objects_arrays_strings_enums_and_pointers() {
    let src = r#"
        enum Kind : u8 { A = 1, B = 2 };
        bitfield Flags { x : 3; bool y : 1; padding : 4; };
        struct Inner { u16 v; } [[sealed]];
        fn hexfmt(u8 v) { return std::format("0x{:02X}", v); };
        struct Root {
            Kind kind;
            Kind unknown;
            char tag[3];
            u8 raw[2];
            Inner sealed;
            Flags flags;
            u8 formatted [[format("hexfmt")]];
            u8 secret [[hidden]];
            padding[1];
            float f;
            char c;
            bool b;
            u16 *ptr : u8;
        };
        Root root @ 0;
    "#;
    let mut data = vec![2u8, 9, b'a', b'b', 0, 7, 8, 0x34, 0x12, 0b0000_1101, 0xAB, 0xFF, 0x00];
    data.extend(1.5f32.to_le_bytes());
    data.extend([0x89, 1, 20, 0xCD, 0xAB]);
    let expected = r#"{
    "root": {
        "kind": "Kind::B",
        "unknown": "Kind::???",
        "tag": "ab\u0000",
        "raw": [
            7,
            8
        ],
        "sealed": "Inner sealed @ 0x7",
        "flags": {
            "x": 5,
            "y": true
        },
        "formatted": "0xAB",
        "f": 1.5,
        "c": "\\x89",
        "b": true,
        "ptr": {
            "*(ptr)": 43981
        }
    }
}"#;
    assert_eq!(json(src, &data, false).trim(), expected.trim());
}

#[test]
fn metadata_lines() {
    let src = "struct S { u8 a [[comment(\"first\")]]; } [[comment(\"outer\")]]; S s @ 0;";
    let out = json(src, &[5], true);
    let expected = r##"{
    "s": {
        "__type": "S",
        "__address": "0",
        "__size": "1",
        "__color": "#70B4771F",
        "__endian": "little",
        "__comment": "outer",
        "a": 5
    }
}"##;
    assert_eq!(out.trim(), expected.trim());
}
