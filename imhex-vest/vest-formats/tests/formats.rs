//! End-to-end tests: each format is parsed by the verified `vest_lib`-based
//! parser generated from `formats/<name>.hexpat`, the same bytes are
//! evaluated by the hexpat interpreter, and both views must agree. The
//! verified serializer must also reproduce the input bytes.

use hexpat::{PatternKind, Runtime, Value};
use vest_lib::core::exec::parser::Parser;
use vest_lib::core::exec::serializer::{Prepare, SerializerExt};

fn pattern_source(name: &str) -> String {
    let path = format!("{}/../formats/{}.hexpat", env!("CARGO_MANIFEST_DIR"), name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {}", path, e))
}

/// Runs the hexpat interpreter on `data` and returns it for inspection.
fn interpret(name: &str, data: &[u8]) -> Runtime {
    let mut rt = Runtime::new(data.to_vec());
    rt.run_source(&pattern_source(name), None).unwrap_or_else(|e| panic!("interpreter failed on {}: {}", name, e));
    rt
}

/// Looks up `root.member.member[index].member` in the interpreter output.
fn field(rt: &mut Runtime, path: &str) -> Value {
    let mut parts = path.split('.');
    let root = parts.next().unwrap();
    let mut cur = rt.patterns.iter().find(|p| p.borrow().name == root).unwrap_or_else(|| panic!("no pattern {}", root)).clone();
    for part in parts {
        let (name, index) = match part.find('[') {
            Some(i) => (&part[..i], Some(part[i + 1..part.len() - 1].parse::<u64>().unwrap())),
            None => (part, None),
        };
        let next = cur.borrow().member(name).unwrap_or_else(|| panic!("no member {} in {}", name, path));
        cur = match index {
            Some(i) => rt.array_element(&next, i).unwrap(),
            None => next,
        };
    }
    rt.pattern_value(&cur).unwrap()
}

fn u(rt: &mut Runtime, path: &str) -> u128 {
    field(rt, path).as_u128().unwrap()
}

fn s(rt: &mut Runtime, path: &str) -> String {
    field(rt, path).as_str().unwrap()
}

fn count(rt: &mut Runtime, path: &str) -> u64 {
    let v = field(rt, path);
    let p = v.as_pattern().unwrap();
    let n = p.borrow().entry_count();
    n.unwrap()
}

/// Serializes `value` with `fmt` and checks it reproduces `data`.
macro_rules! round_trip {
    ($fmt:expr, $value:expr, $data:expr) => {{
        let len = $fmt.prepare(&$value).expect("prepare");
        assert_eq!(len, $data.len(), "serialized length");
        let mut out = vec![0u8; len];
        $fmt.serialize(&$value, out.as_mut_slice());
        assert_eq!(out, $data, "serializer output");
    }};
}

fn le16(v: u16) -> [u8; 2] {
    v.to_le_bytes()
}
fn le32(v: u32) -> [u8; 4] {
    v.to_le_bytes()
}
fn be32(v: u32) -> [u8; 4] {
    v.to_be_bytes()
}

#[test]
fn bmp() {
    use vest_formats::bmp::*;
    let mut data = Vec::new();
    data.extend(b"BM");
    data.extend(le32(54 + 12));
    data.extend(le16(0));
    data.extend(le16(0));
    data.extend(le32(54));
    data.extend(le32(40));
    data.extend(le32(2)); // width
    data.extend((-2i32).to_le_bytes()); // height (top-down)
    data.extend(le16(1));
    data.extend(le16(24));
    data.extend(le32(0)); // RGB
    data.extend(le32(12));
    data.extend(le32(2835));
    data.extend(le32(2835));
    data.extend(le32(0));
    data.extend(le32(0));
    assert_eq!(data.len(), 54);

    let (n, bmp) = BmpFmt.parse(&data.as_slice()).expect("verified parse");
    assert_eq!(n, 54);
    assert_eq!(bmp.file.signature, b"BM");
    assert_eq!(bmp.file.fileSize, 66);
    assert_eq!(bmp.info.width, 2);
    assert_eq!(bmp.info.height as i32, -2);
    assert_eq!(bmp.info.compression, Compression::RGB);
    assert_eq!(bmp.info.bitsPerPixel, 24);

    let mut rt = interpret("bmp", &data);
    assert_eq!(s(&mut rt, "bmp.file.signature"), "BM");
    assert_eq!(u(&mut rt, "bmp.file.fileSize") as u32, bmp.file.fileSize);
    assert_eq!(field(&mut rt, "bmp.info.height").as_i128().unwrap(), -2);
    assert_eq!(u(&mut rt, "bmp.info.bitsPerPixel") as u16, bmp.info.bitsPerPixel);
    assert_eq!(u(&mut rt, "bmp.info.compression"), 0);

    round_trip!(BmpFmt, bmp, data);
}

#[test]
fn gif() {
    use vest_formats::gif::*;
    let mut data = Vec::new();
    data.extend(b"GIF89a");
    data.extend(le16(320));
    data.extend(le16(200));
    // flags: global table (1) | color resolution 7 (111) | sort 0 | size 7 (111)
    data.push(0b1111_0111);
    data.push(0x10);
    data.push(0);

    let (n, gif) = GifFmt.parse(&data.as_slice()).expect("verified parse");
    assert_eq!(n, 13);
    assert_eq!(gif.header.signature, b"GIF");
    assert_eq!(gif.header.version, b"89a");
    assert_eq!(gif.screen.width, 320);
    assert_eq!(gif.screen.flags.globalColorTable, 1);
    assert_eq!(gif.screen.flags.colorResolution, 7);
    assert_eq!(gif.screen.flags.sortFlag, 0);
    assert_eq!(gif.screen.flags.globalColorTableSize, 7);
    assert_eq!(gif.screen.backgroundColorIndex, 0x10);

    let mut rt = interpret("gif", &data);
    assert_eq!(s(&mut rt, "gif.header.version"), "89a");
    assert_eq!(u(&mut rt, "gif.screen.width"), 320);
    assert_eq!(u(&mut rt, "gif.screen.flags.globalColorTable"), 1);
    assert_eq!(u(&mut rt, "gif.screen.flags.colorResolution"), 7);
    assert_eq!(u(&mut rt, "gif.screen.flags.globalColorTableSize"), 7);

    round_trip!(GifFmt, gif, data);
}

#[test]
fn png() {
    use vest_formats::png::*;
    let mut data = Vec::new();
    data.extend([0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1A, b'\n']);
    // IHDR
    data.extend(be32(13));
    data.extend(b"IHDR");
    data.extend(be32(640));
    data.extend(be32(480));
    data.extend([8, 6, 0, 0, 0]);
    data.extend(be32(0xDEADBEEF));
    // IDAT with 3 bytes
    data.extend(be32(3));
    data.extend(b"IDAT");
    data.extend([1, 2, 3]);
    data.extend(be32(0x11223344));
    // IEND
    data.extend(be32(0));
    data.extend(b"IEND");
    data.extend(be32(0xAE426082));

    let (n, png) = PngFmt.parse(&data.as_slice()).expect("verified parse");
    assert_eq!(n, data.len());
    assert_eq!(png.signature.png, b"PNG");
    assert_eq!(png.chunks.len(), 3);
    assert_eq!(png.chunks[0].type_, b"IHDR");
    match &png.chunks[0].ihdr {
        ChunkIhdr::Variant1(ihdr) => {
            assert_eq!(ihdr.width, 640);
            assert_eq!(ihdr.height, 480);
            assert_eq!(ihdr.colorType, ColorType::RGBA);
        }
        other => panic!("IHDR chunk not dispatched: {:?}", other),
    }
    match &png.chunks[1].ihdr {
        ChunkIhdr::Default(bytes) => assert_eq!(*bytes, &[1, 2, 3][..]),
        other => panic!("IDAT chunk not raw: {:?}", other),
    }
    assert_eq!(png.chunks[2].crc, 0xAE426082);

    let mut rt = interpret("png", &data);
    assert_eq!(count(&mut rt, "png.chunks"), 3);
    assert_eq!(s(&mut rt, "png.chunks[0].type"), "IHDR");
    assert_eq!(u(&mut rt, "png.chunks[0].ihdr.width"), 640);
    assert_eq!(u(&mut rt, "png.chunks[0].ihdr.colorType"), 6);
    assert_eq!(count(&mut rt, "png.chunks[1].data"), 3);
    assert_eq!(u(&mut rt, "png.chunks[2].crc"), 0xAE426082);

    round_trip!(PngFmt, png, data);
}

#[test]
fn wav() {
    use vest_formats::wav::*;
    let mut data = Vec::new();
    data.extend(b"RIFF");
    data.extend(le32(36));
    data.extend(b"WAVE");
    data.extend(b"fmt ");
    data.extend(le32(16));
    data.extend(le16(1));
    data.extend(le16(2));
    data.extend(le32(44100));
    data.extend(le32(176400));
    data.extend(le16(4));
    data.extend(le16(16));

    let (n, wav) = WavFmt.parse(&data.as_slice()).expect("verified parse");
    assert_eq!(n, 36);
    assert_eq!(wav.riff.wave, b"WAVE");
    assert_eq!(wav.fmt.format, Format::PCM);
    assert_eq!(wav.fmt.channels, 2);
    assert_eq!(wav.fmt.sampleRate, 44100);
    assert_eq!(wav.fmt.bitsPerSample, 16);

    let mut rt = interpret("wav", &data);
    assert_eq!(s(&mut rt, "wav.fmt.id"), "fmt ");
    assert_eq!(u(&mut rt, "wav.fmt.format"), 1);
    assert_eq!(u(&mut rt, "wav.fmt.sampleRate"), 44100);
    assert_eq!(u(&mut rt, "wav.fmt.byteRate"), 176400);

    round_trip!(WavFmt, wav, data);
}

fn elf_fixture(class: u8) -> Vec<u8> {
    let mut data = Vec::new();
    data.extend([0x7F, b'E', b'L', b'F', class, 1, 1, 0, 0]);
    data.extend([0u8; 7]);
    data.extend(le16(2)); // EXEC
    data.extend(le16(0x3E)); // x86-64
    data.extend(le32(1));
    if class == 2 {
        data.extend(0x401000u64.to_le_bytes());
        data.extend(64u64.to_le_bytes());
        data.extend(0u64.to_le_bytes());
    } else {
        data.extend(le32(0x8048000));
        data.extend(le32(52));
        data.extend(le32(0));
    }
    data.extend(le32(0));
    data.extend(le16(if class == 2 { 64 } else { 52 }));
    data.extend(le16(if class == 2 { 56 } else { 32 }));
    data.extend(le16(3));
    data.extend(le16(if class == 2 { 64 } else { 40 }));
    data.extend(le16(0));
    data.extend(le16(0));
    data
}

#[test]
fn elf() {
    use vest_formats::elf::*;
    for class in [1u8, 2u8] {
        let data = elf_fixture(class);
        let (n, elf) = ElfFmt.parse(&data.as_slice()).expect("verified parse");
        assert_eq!(n, data.len());
        assert_eq!(elf.ident.magic, &[0x7F, b'E', b'L', b'F'][..]);
        let mut rt = interpret("elf", &data);
        assert_eq!(u(&mut rt, "elf.ident.class"), class as u128);
        match (&elf.header, class) {
            (ElfHeader::Elf64(h), 2) => {
                assert_eq!(elf.ident.class, Class::Elf64);
                assert_eq!(h.type_, ObjectType::Exec);
                assert_eq!(h.entry, 0x401000);
                assert_eq!(h.phnum, 3);
                assert_eq!(u(&mut rt, "elf.header.entry"), 0x401000);
                assert_eq!(u(&mut rt, "elf.header.type"), 2);
            }
            (ElfHeader::Elf32(h), 1) => {
                assert_eq!(elf.ident.class, Class::Elf32);
                assert_eq!(h.entry, 0x8048000);
                assert_eq!(h.ehsize, 52);
                assert_eq!(u(&mut rt, "elf.header.entry"), 0x8048000);
                assert_eq!(u(&mut rt, "elf.header.ehsize"), 52);
            }
            other => panic!("unexpected header dispatch for class {}: {:?}", class, other.0),
        }
        assert_eq!(u(&mut rt, "elf.header.phnum"), 3);
        round_trip!(ElfFmt, elf, data);
    }
}

#[test]
fn tar() {
    use vest_formats::tar::*;
    let mut data = vec![0u8; 512];
    data[..8].copy_from_slice(b"test.txt");
    data[100..108].copy_from_slice(b"0000644\0");
    data[124..136].copy_from_slice(b"00000000012\0");
    data[156] = b'0';
    data[257..263].copy_from_slice(b"ustar\0");
    data[263..265].copy_from_slice(b"00");

    let (n, hdr) = PosixHeaderFmt.parse(&data.as_slice()).expect("verified parse");
    assert_eq!(n, 512);
    assert_eq!(&hdr.name[..8], b"test.txt");
    assert_eq!(hdr.magic, b"ustar\0");
    assert_eq!(hdr.typeflag, b'0');

    let mut rt = interpret("tar", &data);
    assert!(s(&mut rt, "header.name").starts_with("test.txt"));
    assert_eq!(s(&mut rt, "header.magic"), "ustar\0");
    assert_eq!(s(&mut rt, "header.size"), "00000000012\0");

    round_trip!(PosixHeaderFmt, hdr, data);
}

#[test]
fn zip() {
    use vest_formats::zip::*;
    let mut data = Vec::new();
    for (name, body) in [("a.txt", b"hello".as_slice()), ("dir/b.bin", b"\x00\x01\x02".as_slice())] {
        data.extend(b"PK\x03\x04");
        data.extend(le16(20));
        data.extend(le16(0));
        data.extend(le16(0)); // stored
        data.extend(le16(0x1234));
        data.extend(le16(0x5678));
        data.extend(le32(0xCAFEBABE));
        data.extend(le32(body.len() as u32));
        data.extend(le32(body.len() as u32));
        data.extend(le16(name.len() as u16));
        data.extend(le16(0));
        data.extend(name.as_bytes());
        data.extend(body);
    }

    let (n, zip) = ZipFmt.parse(&data.as_slice()).expect("verified parse");
    assert_eq!(n, data.len());
    assert_eq!(zip.entries.len(), 2);
    assert_eq!(zip.entries[0].name, b"a.txt");
    assert_eq!(zip.entries[0].data, b"hello");
    assert_eq!(zip.entries[1].name, b"dir/b.bin");
    assert_eq!(zip.entries[1].method, Method::Stored);
    assert_eq!(zip.entries[1].crc32, 0xCAFEBABE);

    let mut rt = interpret("zip", &data);
    assert_eq!(count(&mut rt, "zip.entries"), 2);
    assert_eq!(s(&mut rt, "zip.entries[0].name"), "a.txt");
    assert_eq!(s(&mut rt, "zip.entries[1].name"), "dir/b.bin");
    assert_eq!(count(&mut rt, "zip.entries[1].data"), 3);
    assert_eq!(u(&mut rt, "zip.entries[1].crc32"), 0xCAFEBABE);

    round_trip!(ZipFmt, zip, data);
}

#[test]
fn ico() {
    use vest_formats::ico::*;
    let mut data = Vec::new();
    data.extend(le16(0));
    data.extend(le16(1));
    data.extend(le16(2));
    for (w, size, off) in [(16u8, 1128u32, 38u32), (32u8, 4264u32, 1166u32)] {
        data.extend([w, w, 0, 0]);
        data.extend(le16(1));
        data.extend(le16(32));
        data.extend(le32(size));
        data.extend(le32(off));
    }

    let (n, ico) = IcoFmt.parse(&data.as_slice()).expect("verified parse");
    assert_eq!(n, data.len());
    assert_eq!(ico.header.type_, ImageType::Icon);
    assert_eq!(ico.entries.len(), 2);
    assert_eq!(ico.entries[1].width, 32);
    assert_eq!(ico.entries[1].offset, 1166);

    let mut rt = interpret("ico", &data);
    assert_eq!(u(&mut rt, "ico.header.type"), 1);
    assert_eq!(count(&mut rt, "ico.entries"), 2);
    assert_eq!(u(&mut rt, "ico.entries[1].width"), 32);
    assert_eq!(u(&mut rt, "ico.entries[1].offset"), 1166);

    round_trip!(IcoFmt, ico, data);
}

#[test]
fn qoi() {
    use vest_formats::qoi::*;
    let mut data = Vec::new();
    data.extend(b"qoif");
    data.extend(be32(1920));
    data.extend(be32(1080));
    data.extend([4, 0]);

    let (n, qoi) = QoiFmt.parse(&data.as_slice()).expect("verified parse");
    assert_eq!(n, 14);
    assert_eq!(&qoi.magic, b"qoif");
    assert_eq!(qoi.width, 1920);
    assert_eq!(qoi.channels, Channels::RGBA);
    assert_eq!(qoi.colorspace, Colorspace::SRGB);

    let mut rt = interpret("qoi", &data);
    assert_eq!(s(&mut rt, "qoi.magic.value"), "qoif");
    assert_eq!(u(&mut rt, "qoi.width"), 1920);
    assert_eq!(u(&mut rt, "qoi.channels"), 4);

    // A bad magic is rejected by both.
    let mut bad = data.clone();
    bad[0] = b'x';
    assert!(QoiFmt.parse(&bad.as_slice()).is_err());
    let mut rt = Runtime::new(bad);
    assert!(rt.run_source(&pattern_source("qoi"), None).is_err());

    round_trip!(QoiFmt, qoi, data);
}

#[test]
fn gzip() {
    use vest_formats::gzip::*;
    let data = vec![0x1F, 0x8B, 8, 0b0000_1000, 0xDC, 0x1F, 0xAA, 0x63, 0, 3];

    let (n, gz) = GzipFmt.parse(&data.as_slice()).expect("verified parse");
    assert_eq!(n, 10);
    assert_eq!(gz.id1, 0x1F);
    assert_eq!(gz.compressionMethod, 8);
    assert_eq!(gz.flags.name, 1);
    assert_eq!(gz.flags.text, 0);
    assert_eq!(gz.flags.comment, 0);
    assert_eq!(gz.mtime, 0x63AA1FDC);
    assert_eq!(gz.os, Os::Unix);

    let mut rt = interpret("gzip", &data);
    assert_eq!(u(&mut rt, "gzip.flags.name"), 1);
    assert_eq!(u(&mut rt, "gzip.flags.text"), 0);
    assert_eq!(u(&mut rt, "gzip.mtime"), 0x63AA1FDC);
    assert_eq!(u(&mut rt, "gzip.os"), 3);

    round_trip!(GzipFmt, gz, data);
}

#[test]
fn interpreter_and_verified_parser_agree_on_layout() {
    use vest_formats::elf::*;
    // Every root pattern's byte length equals what the verified parser consumed.
    let cases: Vec<(&str, Vec<u8>, usize)> = vec![
        ("elf", elf_fixture(2), 64),
        ("elf", elf_fixture(1), 52),
    ];
    for (name, data, expected) in cases {
        let (n, _) = ElfFmt.parse(&data.as_slice()).unwrap();
        assert_eq!(n, expected);
        let rt = interpret(name, &data);
        let root = rt.patterns[0].clone();
        assert_eq!(root.borrow().size as usize, expected);
        assert!(matches!(root.borrow().kind, PatternKind::Struct { .. }));
    }
}
