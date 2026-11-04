//! Runs the verified strict Parquet parser on the fixtures written by
//! `scripts/make-parquet-fixtures.py`: `ok-*.parquet` must be accepted,
//! `bad-*.parquet` rejected, each at the stage expected. Also checks the
//! canonical Thrift rules on hand-written encodings.
use interval_vest::parquet::{parse_parquet, PKind, PNode, Parquet};
use interval_vest::ranked::{check_tiled, resolve, RankedFormatExec};
use interval_vest::thrift::{parse_value, EVal};
use std::fs;
use std::path::PathBuf;

fn data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data")
}

fn fixture(name: &str) -> Vec<u8> {
    fs::read(data_dir().join(name)).unwrap_or_else(|e| panic!("{name}: {e}"))
}

#[derive(Debug, PartialEq)]
enum Stage {
    /// A record does not parse, or a node, rank or link check fails.
    Resolve,
    /// The regions do not tile the file.
    Tiling,
    Accepted,
}

fn stage(b: &[u8]) -> Stage {
    let j = Parquet.root(b).expect("fixtures are longer than 8 bytes");
    let Some(rs) = resolve(&Parquet, b, j) else { return Stage::Resolve };
    if check_tiled(&rs, &Parquet.layout(&rs), b.len()) {
        Stage::Accepted
    } else {
        Stage::Tiling
    }
}

#[test]
fn every_fixture_is_classified() {
    let mut names: Vec<String> = fs::read_dir(data_dir())
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .filter(|n| n.ends_with(".parquet"))
        .collect();
    names.sort();
    assert!(names.len() >= 20, "fixtures missing; run scripts/make-parquet-fixtures.py");
    for n in &names {
        let accepted = parse_parquet(&fixture(n)).is_some();
        assert_eq!(accepted, n.starts_with("ok-"), "{n}");
    }
}

#[test]
fn rejections_happen_where_expected() {
    use Stage::*;
    let cases = [
        // The tail must be "PAR1" and the metadata must end right before it.
        ("bad-trailing-byte.parquet", Resolve),
        ("bad-no-head.parquet", Resolve),
        // FileMetaData and page checks.
        ("bad-num-rows.parquet", Resolve),
        ("bad-num-values.parquet", Resolve),
        ("bad-data-page-offset.parquet", Resolve),
        ("bad-uncompressed-size.parquet", Resolve),
        ("bad-inline-copy-mismatch.parquet", Resolve),
        // Encodings Thrift readers accept but the canonical profile does not.
        ("bad-nonminimal-varint.parquet", Resolve),
        ("bad-long-field-header.parquet", Resolve),
        ("bad-field-order.parquet", Resolve),
        // Every region is fine on its own; the layout is not.
        ("bad-gap-between-chunks.parquet", Tiling),
        ("bad-overlapping-chunks.parquet", Tiling),
        ("bad-polars-unreferenced-copy.parquet", Tiling),
        ("ok-pyarrow.parquet", Accepted),
    ];
    for (n, want) in cases {
        assert_eq!(stage(&fixture(n)), want, "{n}");
    }
}

#[test]
fn regions_tile_the_file() {
    for n in ["ok-pyarrow.parquet", "ok-pyarrow-pageindex.parquet", "ok-legacy-inline-copy.parquet"] {
        let b = fixture(n);
        let rs = parse_parquet(&b).unwrap();
        let mut spans: Vec<(usize, usize)> = rs.iter().map(|r| (r.start, r.start + r.len)).collect();
        spans.sort();
        assert_eq!(spans[0], (0, 4), "{n}");
        for w in spans.windows(2) {
            assert_eq!(w[0].1, w[1].0, "{n}: regions must be contiguous");
        }
        assert_eq!(spans.last().unwrap().1, b.len(), "{n}");
    }
}

#[test]
fn region_kinds() {
    let count = |n: &str, f: fn(&PKind) -> bool| {
        parse_parquet(&fixture(n)).unwrap().iter().filter(|r| f(&r.kind)).count()
    };
    let chunk = |k: &PKind| matches!(k, PKind::Chunk { .. });
    let copy = |k: &PKind| matches!(k, PKind::InlineCopy { .. });
    let index = |k: &PKind| matches!(k, PKind::ColumnIndex | PKind::OffsetIndex);
    let bloom = |k: &PKind| matches!(k, PKind::Bloom);
    // Five columns; four row groups of 50 rows in the second file.
    assert_eq!(count("ok-pyarrow.parquet", chunk), 5);
    assert_eq!(count("ok-pyarrow-rowgroups.parquet", chunk), 20);
    assert_eq!(count("ok-pyarrow-pageindex.parquet", index), 10);
    assert_eq!(count("ok-legacy-inline-copy.parquet", copy), 5);
    assert!(count("ok-duckdb.parquet", bloom) > 0, "duckdb writes bloom filters");
    // An empty table still has one row group. Four of its chunks hold just a
    // dictionary page; the boolean column's chunk is empty, at offset 0.
    let b = fixture("ok-pyarrow-empty.parquet");
    let rs = parse_parquet(&b).unwrap();
    let pages: Vec<(usize, usize)> = rs
        .iter()
        .filter_map(|r| match &r.val {
            PNode::Pages(ps) => Some((r.start, ps.len())),
            _ => None,
        })
        .collect();
    assert_eq!(pages.iter().filter(|p| p.1 == 1).count(), 4);
    assert_eq!(pages.iter().filter(|p| **p == (0, 0)).count(), 1);
}

#[test]
fn single_byte_corruptions_are_rejected_or_change_the_parse() {
    let good = fixture("ok-pyarrow.parquet");
    let reference = summary(&good).unwrap();
    for i in 0..good.len() {
        let mut b = good.clone();
        b[i] ^= 0x01;
        if let Some(s) = summary(&b) {
            assert_ne!(s, reference, "byte {i}");
        }
    }
}

/// The regions and the bytes each one covers: two accepted inputs with the
/// same parse have the same summary.
fn summary(b: &[u8]) -> Option<Vec<(String, usize, usize, Vec<u8>)>> {
    Some(
        parse_parquet(b)?
            .iter()
            .map(|r| (format!("{:?}", r.kind), r.start, r.len, b[r.start..r.start + r.len].to_vec()))
            .collect(),
    )
}

fn thrift_struct(b: &[u8]) -> Option<usize> {
    parse_value(b, 0, 12, 16).map(|(n, _)| n)
}

#[test]
fn canonical_thrift() {
    // {1: i32 21}: short header (delta 1, type 5), zigzag varint 42.
    assert_eq!(thrift_struct(&[0x15, 0x2a, 0x00]), Some(3));
    // The same value with a two-byte varint.
    assert_eq!(thrift_struct(&[0x15, 0xaa, 0x00, 0x00]), None);
    // The same field with a long header (type, then zigzag field id 1 = 2).
    assert_eq!(thrift_struct(&[0x05, 0x02, 0x2a, 0x00]), None);
    // Field 20 needs the long form: delta 20 > 15.
    assert_eq!(thrift_struct(&[0x05, 0x28, 0x2a, 0x00]), Some(4));
    // Field ids must increase: 2 then 1 (long form for the negative delta).
    assert_eq!(thrift_struct(&[0x25, 0x2a, 0x05, 0x02, 0x2a, 0x00]), None);
    // Booleans: true in the header (type 1); a list of two booleans.
    assert_eq!(thrift_struct(&[0x11, 0x00]), Some(2));
    assert_eq!(thrift_struct(&[0x19, 0x21, 0x01, 0x02, 0x00]), Some(5));
    assert_eq!(thrift_struct(&[0x19, 0x21, 0x01, 0x00, 0x00]), None);
    // A list of 3 i32: short header; the long header for a size below 15 is rejected.
    assert_eq!(thrift_struct(&[0x19, 0x35, 0x02, 0x04, 0x06, 0x00]), Some(6));
    assert_eq!(thrift_struct(&[0x19, 0xf5, 0x03, 0x02, 0x04, 0x06, 0x00]), None);
    // Maps (type 11) are not part of the profile.
    assert_eq!(thrift_struct(&[0x1b, 0x00, 0x00]), None);
    // Nesting is bounded: 20 nested structs exceed the depth limit of 16.
    let mut deep = vec![0x1c; 20];
    deep.extend(std::iter::repeat(0).take(21));
    assert_eq!(thrift_struct(&deep), None);
    match parse_value(&[0x15, 0x2a, 0x00], 0, 12, 16) {
        Some((_, EVal::Struct(fs))) => {
            assert_eq!(fs.len(), 1);
            assert_eq!(fs[0].id, 1);
            assert!(matches!(fs[0].val, EVal::I32(21)));
        },
        _ => panic!(),
    }
}
