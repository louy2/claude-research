//! Runs the verified strict ZIP parser on the fixtures written by
//! `scripts/make-zip-fixtures.py`: `ok-*.zip` must be accepted, `bad-*.zip`
//! rejected.
use interval_vest::zip::{parse_zip, ZipKind, ZipNode};
use std::fs;
use std::path::PathBuf;

fn data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data")
}

fn fixture(name: &str) -> Vec<u8> {
    fs::read(data_dir().join(name)).unwrap_or_else(|e| panic!("{name}: {e}"))
}

/// The names and contents of the local files, in central directory order.
fn files(b: &[u8]) -> Option<Vec<(Vec<u8>, Vec<u8>)>> {
    let rs = parse_zip(b)?;
    Some(
        rs.iter()
            .filter_map(|r| match &r.val {
                ZipNode::LocalFile(l) => Some((l.name.to_vec(), l.data.to_vec())),
                _ => None,
            })
            .collect(),
    )
}

#[test]
fn every_fixture_is_classified() {
    let mut names: Vec<String> = fs::read_dir(data_dir())
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .filter(|n| n.ends_with(".zip"))
        .collect();
    names.sort();
    assert!(names.len() >= 12, "fixtures missing; run scripts/make-zip-fixtures.py");
    for n in &names {
        let accepted = parse_zip(&fixture(n)).is_some();
        if n.starts_with("ok-") {
            assert!(accepted, "{n} should be accepted");
        } else {
            assert!(!accepted, "{n} should be rejected");
        }
    }
}

#[test]
fn regions_tile_the_file() {
    for n in ["ok-canonical.zip", "ok-python-stored.zip", "ok-python-deflated.zip", "ok-infozip.zip"] {
        let b = fixture(n);
        let rs = parse_zip(&b).unwrap();
        assert_eq!(rs[0].kind, ZipKind::Eocd, "{n}");
        assert_eq!(rs[1].kind, ZipKind::CentralDir, "{n}");
        assert!(rs[2..].iter().all(|r| r.kind == ZipKind::LocalFile), "{n}");
        let mut spans: Vec<(usize, usize)> = rs.iter().map(|r| (r.start, r.start + r.len)).collect();
        spans.sort();
        assert_eq!(spans[0].0, 0, "{n}");
        for w in spans.windows(2) {
            assert_eq!(w[0].1, w[1].0, "{n}: regions must be contiguous");
        }
        assert_eq!(spans.last().unwrap().1, b.len(), "{n}");
    }
}

#[test]
fn python_archives_have_the_expected_files() {
    let stored = files(&fixture("ok-python-stored.zip")).unwrap();
    let names: Vec<&[u8]> = stored.iter().map(|(n, _)| n.as_slice()).collect();
    assert_eq!(names, [&b"hello.txt"[..], b"dir/numbers.bin", b"empty"]);
    assert_eq!(stored[0].1, b"Hello, interval parsing!\n");
    assert_eq!(stored[1].1.len(), 1024);
    assert!(stored[2].1.is_empty());

    let b = fixture("ok-python-empty.zip");
    assert_eq!(parse_zip(&b).unwrap().len(), 2, "end record and an empty central directory");
}

#[test]
fn comment_and_trailing_bytes() {
    let good = fixture("ok-canonical.zip");
    match &parse_zip(&good).unwrap()[0].val {
        ZipNode::Eocd(e) => assert_eq!(e.comment, b"archive comment"),
        _ => unreachable!(),
    }
    // The end record must end the file; a shorter or longer comment length
    // does not line up.
    for cut in [1, 2, 7] {
        assert!(parse_zip(&good[..good.len() - cut]).is_none(), "truncated by {cut}");
    }
}

#[test]
fn single_byte_corruptions_are_rejected_or_change_the_parse() {
    // Non-malleability, checked on one input: no other byte string parses to
    // the same regions and values. Flip each byte in turn; the parser must
    // either reject or return something different.
    let good = fixture("ok-canonical.zip");
    let reference = format!("{:?}", summary(&good).unwrap());
    for i in 0..good.len() {
        let mut b = good.clone();
        b[i] ^= 0x01;
        if let Some(s) = summary(&b) {
            assert_ne!(format!("{s:?}"), reference, "byte {i}");
        }
    }
}

/// Everything the parse returns, as owned data.
fn summary(b: &[u8]) -> Option<Vec<(ZipKind, usize, usize, String)>> {
    Some(
        parse_zip(b)?
            .iter()
            .map(|r| (r.kind, r.start, r.len, node_debug(&r.val)))
            .collect(),
    )
}

fn node_debug(v: &ZipNode) -> String {
    match v {
        ZipNode::Eocd(e) => format!("{e:?}"),
        ZipNode::CentralDir(c) => format!("{c:?}"),
        ZipNode::LocalFile(l) => format!("{l:?}"),
    }
}

#[derive(Debug, PartialEq)]
enum Stage {
    /// No end of central directory record ends the file.
    Root,
    /// A record does not parse, a rank or link check fails.
    Resolve,
    /// The regions do not tile the file.
    Tiling,
    Accepted,
}

fn stage(b: &[u8]) -> Stage {
    use interval_vest::ranked::{check_tiled, resolve, RankedFormatExec};
    use interval_vest::zip::Zip;
    let Some(j) = Zip.root(b) else { return Stage::Root };
    let Some(rs) = resolve(&Zip, b, j) else { return Stage::Resolve };
    if check_tiled(&rs, &Zip.layout(&rs), b.len()) {
        Stage::Accepted
    } else {
        Stage::Tiling
    }
}

#[test]
fn rejections_happen_where_expected() {
    use Stage::*;
    let cases = [
        ("bad-trailing-byte.zip", Root),
        ("bad-prefixed.zip", Resolve),
        ("bad-name-mismatch.zip", Resolve),
        ("bad-count-mismatch.zip", Resolve),
        ("bad-data-descriptor-flag.zip", Resolve),
        ("bad-gap-between-files.zip", Tiling),
        ("bad-gap-before-cd.zip", Tiling),
        ("bad-local-order.zip", Tiling),
        // Both central entries reach the same local file and agree with it;
        // only the tiling check sees that the regions overlap.
        ("bad-overlap.zip", Tiling),
        // The fake record is found first and resolves (an empty central
        // directory at offset 0), but cannot tile the file.
        ("bad-fake-eocd-in-comment.zip", Tiling),
        ("ok-canonical.zip", Accepted),
    ];
    for (n, want) in cases {
        assert_eq!(stage(&fixture(n)), want, "{n}");
    }
    if data_dir().join("bad-infozip-stdin-data-descriptor.zip").exists() {
        assert_eq!(stage(&fixture("bad-infozip-stdin-data-descriptor.zip")), Resolve);
    }
}
