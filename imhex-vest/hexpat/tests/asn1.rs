//! The ASN.1/ACN translation of `formats/*.hexpat`.
//!
//! 1. The committed specs under `asn1-formats/asn1/` are what `emit-asn1`
//!    produces today.
//! 2. When `scripts/regen-asn1.sh` has built the ASN1SCC Scala decoders
//!    (`asn1-formats/build/<name>/classpath` exists), each sample file is
//!    decoded by them and by the hexpat interpreter, and every decoded field
//!    must agree. The Scala harness also re-encodes every decoded value:
//!    decoding the re-encoded bytes must give the same value, and the bytes
//!    must be identical except where the emitter reported a lossy length.
//!
//! Samples are synthetic files built here plus, when the ImHex-Patterns
//! corpus is available (`IMHEX_PATTERNS` or `vendor/ImHex-Patterns`), the
//! corpus fixture for each format.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use hexpat::emit_asn1::{emit_asn1, Asn1Options, Asn1Output, Role, Shape, TypeMap};
use hexpat::{PatternRef, Runtime};
use serde_json::Value as Json;

const FORMATS: &[&str] = &["bmp", "elf", "gif", "gzip", "ico", "png", "qoi", "tar", "wav", "zip"];

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn pattern_source(name: &str) -> String {
    let path = root().join(format!("formats/{}.hexpat", name));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {}", path.display(), e))
}

fn emit(name: &str) -> Asn1Output {
    let mut rt = Runtime::new(Vec::new());
    let program = rt.load(&pattern_source(name), None).unwrap();
    emit_asn1(&program, rt.default_endian, &Asn1Options { module: name.into(), ..Default::default() }).unwrap()
}

#[test]
fn committed_specs_are_up_to_date() {
    for name in FORMATS {
        let out = emit(name);
        let dir = root().join("asn1-formats/asn1").join(name);
        for (file, text) in [
            (format!("{}.asn", name), out.asn.clone()),
            (format!("{}.acn", name), out.acn.clone()),
            (format!("{}.map.json", name), out.map_json()),
            ("HexpatHarness.scala".to_string(), out.scala_harness()),
        ] {
            let path = dir.join(&file);
            let committed = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {}", path.display(), e));
            assert_eq!(committed, text, "{} is stale; run scripts/regen-asn1.sh", path.display());
        }
        assert!(out.root.is_some() || out.trailing.is_some(), "{}: nothing to decode", name);
        // The map file is valid JSON.
        let _: Json = serde_json::from_str(&out.map_json()).unwrap();
    }
}

#[test]
fn every_format_translates() {
    // What each format is expected to need from the translation.
    let png = emit("png");
    assert_eq!(png.trailing.as_ref().map(|t| t.field.as_str()), Some("chunks"));
    assert_eq!(png.lossy.len(), 1, "{:?}", png.lossy);
    let zip = emit("zip");
    assert!(zip.root.is_none() && zip.trailing.as_ref().map(|t| t.element.as_str()) == Some("LocalFileHeader"));
    let elf = emit("elf");
    assert!(elf.acn.contains("header [present-when (ident-class == 1)]"), "{}", elf.acn);
    let ico = emit("ico");
    assert!(ico.acn.contains("entries [size header-count]"), "{}", ico.acn);
    let qoi = emit("qoi");
    assert!(qoi.asn.contains("magic INTEGER (1903126886)"), "{}", qoi.asn);
    for name in FORMATS {
        let out = emit(name);
        assert!(out.notes.iter().all(|n| !n.contains("not emitted")), "{}: {:?}", name, out.notes);
    }
}

// ----------------------------------------------------------------------
// Differential test against the ASN1SCC Scala decoders
// ----------------------------------------------------------------------

fn le16(v: u16) -> [u8; 2] {
    v.to_le_bytes()
}
fn le32(v: u32) -> [u8; 4] {
    v.to_le_bytes()
}
fn be32(v: u32) -> [u8; 4] {
    v.to_be_bytes()
}

fn synthetic(name: &str) -> Vec<Vec<u8>> {
    let mut d = Vec::new();
    match name {
        "bmp" => {
            d.extend(b"BM");
            d.extend(le32(66));
            d.extend(le16(0));
            d.extend(le16(0));
            d.extend(le32(54));
            d.extend(le32(40));
            d.extend(le32(2));
            d.extend((-2i32).to_le_bytes());
            d.extend(le16(1));
            d.extend(le16(24));
            d.extend(le32(0));
            d.extend(le32(12));
            d.extend(le32(2835));
            d.extend(le32(2835));
            d.extend(le32(0));
            d.extend(le32(0));
        }
        "gif" => {
            d.extend(b"GIF89a");
            d.extend(le16(320));
            d.extend(le16(200));
            d.extend([0b1111_0111, 0x10, 0]);
        }
        "png" => {
            d.extend([0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1A, b'\n']);
            d.extend(be32(13));
            d.extend(b"IHDR");
            d.extend(be32(640));
            d.extend(be32(480));
            d.extend([8, 6, 0, 0, 0]);
            d.extend(be32(0xDEADBEEF));
            d.extend(be32(3));
            d.extend(b"IDAT");
            d.extend([1, 2, 3]);
            d.extend(be32(0x11223344));
            d.extend(be32(0));
            d.extend(b"IEND");
            d.extend(be32(0xAE426082));
        }
        "wav" => {
            d.extend(b"RIFF");
            d.extend(le32(36));
            d.extend(b"WAVEfmt ");
            d.extend(le32(16));
            d.extend(le16(1));
            d.extend(le16(2));
            d.extend(le32(44100));
            d.extend(le32(176400));
            d.extend(le16(4));
            d.extend(le16(16));
        }
        "elf" => {
            let mut all = Vec::new();
            for class in [1u8, 2, 7] {
                let mut d = Vec::new();
                d.extend([0x7F, b'E', b'L', b'F', class, 1, 1, 0, 0]);
                d.extend([0u8; 7]);
                if class == 7 {
                    // Neither 32- nor 64-bit: the header is absent.
                    all.push(d);
                    continue;
                }
                d.extend(le16(2));
                d.extend(le16(0x3E));
                d.extend(le32(1));
                if class == 2 {
                    d.extend(0x401000u64.to_le_bytes());
                    d.extend(64u64.to_le_bytes());
                    d.extend(0xFFFF_FFFF_FFFF_FFF0u64.to_le_bytes());
                } else {
                    d.extend(le32(0x8048000));
                    d.extend(le32(52));
                    d.extend(le32(0));
                }
                d.extend(le32(0));
                d.extend(le16(if class == 2 { 64 } else { 52 }));
                d.extend(le16(if class == 2 { 56 } else { 32 }));
                d.extend(le16(3));
                d.extend(le16(if class == 2 { 64 } else { 40 }));
                d.extend(le16(0));
                d.extend(le16(0));
                all.push(d);
            }
            return all;
        }
        "tar" => {
            d = vec![0u8; 512];
            d[..8].copy_from_slice(b"test.txt");
            d[100..108].copy_from_slice(b"0000644\0");
            d[124..136].copy_from_slice(b"00000000012\0");
            d[156] = b'0';
            d[257..263].copy_from_slice(b"ustar\0");
            d[263..265].copy_from_slice(b"00");
        }
        "zip" => {
            for (name, body) in [("a.txt", b"hello".as_slice()), ("dir/b.bin", b"\x00\x01\x02".as_slice())] {
                d.extend(b"PK\x03\x04");
                d.extend(le16(20));
                d.extend(le16(0));
                d.extend(le16(0));
                d.extend(le16(0x1234));
                d.extend(le16(0x5678));
                d.extend(le32(0xCAFEBABE));
                d.extend(le32(body.len() as u32));
                d.extend(le32(body.len() as u32));
                d.extend(le16(name.len() as u16));
                d.extend(le16(2));
                d.extend(name.as_bytes());
                d.extend([0xAA, 0xBB]);
                d.extend(body);
            }
        }
        "ico" => {
            d.extend(le16(0));
            d.extend(le16(1));
            d.extend(le16(2));
            for (w, size, off) in [(16u8, 1128u32, 38u32), (32u8, 4264u32, 1166u32)] {
                d.extend([w, w, 0, 0]);
                d.extend(le16(1));
                d.extend(le16(32));
                d.extend(le32(size));
                d.extend(le32(off));
            }
        }
        "qoi" => {
            d.extend(b"qoif");
            d.extend(be32(1920));
            d.extend(be32(1080));
            d.extend([4, 0]);
        }
        "gzip" => {
            d.extend([0x1F, 0x8B, 8, 0b0000_1001]);
            d.extend(le32(0x5F5E_1000));
            d.extend([2, 3]);
        }
        _ => unreachable!(),
    }
    vec![d]
}

fn corpus_fixture(name: &str) -> Option<PathBuf> {
    let base = match std::env::var("IMHEX_PATTERNS") {
        Ok(p) => PathBuf::from(p),
        Err(_) => root().join("vendor/ImHex-Patterns"),
    };
    let dir = base.join("tests/patterns/test_data");
    let prefix = format!("{}.hexpat", name);
    let mut found: Vec<PathBuf> = std::fs::read_dir(&dir)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .flat_map(|p| {
            let f = p.file_name().unwrap().to_string_lossy().to_string();
            if p.is_dir() && f == prefix {
                std::fs::read_dir(&p).map(|r| r.flatten().map(|e| e.path()).collect::<Vec<_>>()).unwrap_or_default()
            } else if f.starts_with(&format!("{}.", prefix)) {
                vec![p]
            } else {
                Vec::new()
            }
        })
        .filter(|p| p.is_file())
        .collect();
    found.sort();
    found.into_iter().next()
}

struct Ctx<'a> {
    types: HashMap<String, &'a TypeMap>,
    data: &'a [u8],
    errors: Vec<String>,
    fields: usize,
}

fn resolve(pat: &PatternRef, path: &str) -> Option<PatternRef> {
    let mut cur = pat.clone();
    for part in path.split('.') {
        let next = cur.borrow().member(part)?;
        cur = next;
    }
    Some(cur)
}

fn bytes_of(ctx: &Ctx, p: &PatternRef) -> Vec<u8> {
    let (off, size) = (p.borrow().offset as usize, p.borrow().size as usize);
    ctx.data.get(off..off + size).map(|s| s.to_vec()).unwrap_or_default()
}

fn json_bytes(j: &Json) -> Option<Vec<u8>> {
    j.as_array()?.iter().map(|b| b.as_u64().map(|v| v as u8)).collect()
}

fn compare_shape(ctx: &mut Ctx, rt: &mut Runtime, shape: &Shape, j: &Json, p: &PatternRef, at: &str) {
    ctx.fields += 1;
    let ok = match shape {
        Shape::Int | Shape::Unsigned(_) => {
            let v = rt.pattern_value(p).and_then(|v| v.as_i128()).unwrap_or_else(|e| panic!("{}: {}", at, e));
            let v = match shape {
                Shape::Unsigned(bits) => v.rem_euclid(1i128 << bits),
                _ => v,
            };
            // The Scala harness prints 64-bit values as signed longs.
            j.as_i64() == Some(v as i64)
        }
        Shape::CharsInt(n) => {
            let b = bytes_of(ctx, p);
            let v = b.iter().take(*n as usize).fold(0u64, |acc, x| (acc << 8) | *x as u64);
            j.as_i64() == Some(v as i64)
        }
        Shape::Bytes => json_bytes(j).as_deref() == Some(bytes_of(ctx, p).as_slice()),
        Shape::Real => {
            let v = rt.pattern_value(p).and_then(|v| v.as_f64()).unwrap();
            j.as_f64() == Some(v)
        }
        Shape::Ref(t) => {
            compare_type(ctx, rt, t, j, p, at);
            true
        }
        Shape::SeqOf(elem) => {
            let n = p.borrow().entry_count().unwrap_or(0);
            match j.as_array() {
                Some(items) if items.len() as u64 == n => {
                    for (i, item) in items.iter().enumerate() {
                        let e = rt.array_element(p, i as u64).unwrap();
                        compare_shape(ctx, rt, elem, item, &e, &format!("{}[{}]", at, i));
                    }
                    true
                }
                _ => false,
            }
        }
        Shape::Null => true,
    };
    if !ok {
        ctx.errors.push(format!("{}: ASN1SCC decoded {} ({:?})", at, j, shape));
    }
}

fn compare_type(ctx: &mut Ctx, rt: &mut Runtime, ty: &str, j: &Json, p: &PatternRef, at: &str) {
    let t = *ctx.types.get(ty).unwrap_or_else(|| panic!("no type {}", ty));
    if let Some(shape) = &t.alias {
        compare_shape(ctx, rt, shape, j, p, at);
        return;
    }
    // Paths decoded by some present component (two OPTIONAL components can
    // stand for the branches of one hexpat member).
    let mut present: Vec<&str> = Vec::new();
    for c in &t.components {
        if matches!(c.role, Role::Inserted | Role::Padding) {
            continue;
        }
        let key = c.name.replace('-', "_");
        let Some(jv) = j.get(&key) else {
            ctx.errors.push(format!("{}.{}: missing in the ASN1SCC value", at, key));
            continue;
        };
        if jv.is_null() && c.optional {
            continue;
        }
        present.push(&c.path);
        let here = format!("{}.{}", at, c.path);
        match resolve(p, &c.path) {
            Some(target) => compare_shape(ctx, rt, &c.shape, jv, &target, &here),
            None => ctx.errors.push(format!("{}: decoded by ASN1SCC but absent in the interpreter", here)),
        }
    }
    // Members that hexpat decoded must not be missing on the ASN.1 side.
    for c in t.components.iter().filter(|c| c.optional) {
        if !present.contains(&c.path.as_str()) && resolve(p, &c.path).is_some() {
            ctx.errors.push(format!("{}.{}: decoded by the interpreter but absent in the ASN1SCC value", at, c.path));
        }
    }
}

fn run_harness(classpath: &str, file: &Path) -> Json {
    let java = std::env::var("JAVA").unwrap_or_else(|_| "java".into());
    let out = Command::new(java)
        .args(["-Xss256m", "-cp", classpath, "asn1scala.HexpatHarness"])
        .arg(file)
        .output()
        .expect("run java");
    assert!(out.status.success(), "harness failed on {}: {}", file.display(), String::from_utf8_lossy(&out.stderr));
    serde_json::from_slice(&out.stdout).unwrap_or_else(|e| panic!("harness output for {}: {}", file.display(), e))
}

/// Checks one decoded PDU report from the harness.
fn check_unit(ctx: &mut Ctx, rt: &mut Runtime, out: &Asn1Output, unit: &Json, p: &PatternRef, at: &str) {
    let ty = unit["type"].as_str().unwrap().to_string();
    let asn_ty = out.types.iter().find(|t| t.name.replace('-', "_") == ty).map(|t| t.name.clone()).unwrap();
    if let Some(e) = unit.get("error") {
        ctx.errors.push(format!("{}: ASN1SCC decoding failed: {}", at, e));
        return;
    }
    let expected_bytes = (unit["bits"].as_u64().unwrap() / 8) as u64;
    if expected_bytes != p.borrow().size {
        ctx.errors.push(format!("{}: ASN1SCC consumed {} bytes, the interpreter {}", at, expected_bytes, p.borrow().size));
    }
    compare_type(ctx, rt, &asn_ty, &unit["value"], p, at);
    if unit["valueRoundTrip"] != Json::Bool(true) {
        ctx.errors.push(format!("{}: decode(encode(v)) != v", at));
    }
    if unit["bytesRoundTrip"] != Json::Bool(true) {
        let lossy = out.lossy.iter().any(|l| l.starts_with(&format!("{}.", asn_ty)));
        if !lossy {
            ctx.errors.push(format!("{}: re-encoding changed the bytes", at));
        }
    }
}

#[test]
fn scala_decoders_agree_with_the_interpreter() {
    let mut checked = 0;
    for name in FORMATS {
        let build = root().join("asn1-formats/build").join(name);
        let Ok(classpath) = std::fs::read_to_string(build.join("classpath")) else {
            eprintln!("{}: no ASN1SCC build (run scripts/regen-asn1.sh); skipping", name);
            continue;
        };
        let out = emit(name);
        let mut files: Vec<(String, Vec<u8>)> =
            synthetic(name).into_iter().enumerate().map(|(i, d)| (format!("synthetic #{}", i + 1), d)).collect();
        if let Some(f) = corpus_fixture(name) {
            files.push((f.file_name().unwrap().to_string_lossy().to_string(), std::fs::read(&f).unwrap()));
        }
        for (label, data) in files {
            let tmp = std::env::temp_dir().join(format!("hexpat-asn1-{}-{}.bin", name, std::process::id()));
            std::fs::write(&tmp, &data).unwrap();
            let report = run_harness(classpath.trim(), &tmp);
            let _ = std::fs::remove_file(&tmp);

            let mut rt = Runtime::new(data.clone());
            if let Err(e) = rt.run_source(&pattern_source(name), None) {
                // Inputs the pattern does not describe must be rejected by both.
                assert_ne!(report["ok"], Json::Bool(true), "{} {}: the interpreter fails ({}) but ASN1SCC accepts: {}", name, label, e, report);
                eprintln!("{} {}: rejected by both ({})", name, label, e);
                checked += 1;
                continue;
            }
            let root_pat = rt.patterns.first().unwrap().clone();
            let mut ctx = Ctx { types: out.types.iter().map(|t| (t.name.replace('-', "_"), t)).collect(), data: &data, errors: Vec::new(), fields: 0 };
            // Keys are Scala names; compare_type looks up ASN.1 names too.
            for t in &out.types {
                ctx.types.insert(t.name.clone(), t);
            }
            let at = format!("{} {}", name, label);
            if report["ok"] != Json::Bool(true) {
                ctx.errors.push(format!("{}: harness stopped early: {}", at, report));
            }
            if let Some(unit) = report.get("root") {
                let mut unit = unit.clone();
                // The root's own size excludes the trailing array.
                if out.trailing.is_some() {
                    let bits = unit["bits"].as_u64().unwrap_or(0);
                    let mut root_ctx_size = root_pat.borrow().size;
                    let trailing = out.trailing.as_ref().unwrap();
                    if let Some(arr) = root_pat.borrow().member(&trailing.field) {
                        root_ctx_size -= arr.borrow().size;
                    }
                    if bits / 8 != root_ctx_size {
                        ctx.errors.push(format!("{}: root consumed {} bytes, expected {}", at, bits / 8, root_ctx_size));
                    }
                    unit["bits"] = Json::from(root_pat.borrow().size * 8);
                }
                check_unit(&mut ctx, &mut rt, &out, &unit, &root_pat, &at);
            }
            if let Some(trailing) = &out.trailing {
                let arr = root_pat.borrow().member(&trailing.field).expect("interpreter trailing array");
                let n = arr.borrow().entry_count().unwrap_or(0);
                let units = report["trailing"].as_array().cloned().unwrap_or_default();
                if units.len() as u64 != n {
                    ctx.errors.push(format!("{}: {} trailing elements decoded, the interpreter has {}", at, units.len(), n));
                }
                for (i, unit) in units.iter().enumerate().take(n as usize) {
                    let e = rt.array_element(&arr, i as u64).unwrap();
                    check_unit(&mut ctx, &mut rt, &out, unit, &e, &format!("{}.{}[{}]", at, trailing.field, i));
                }
            }
            assert!(ctx.errors.is_empty(), "{} {}:\n  {}", name, label, ctx.errors.join("\n  "));
            eprintln!("{} {}: {} fields agree", name, label, ctx.fields);
            checked += 1;
        }
    }
    eprintln!("{} files checked", checked);
}
