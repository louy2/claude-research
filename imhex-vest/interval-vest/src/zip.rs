//! A strict ZIP profile as a ranked format.
//!
//! | kind              | rank | region                                   | jumps to |
//! |-------------------|------|------------------------------------------|----------|
//! | end of central dir| 2    | found by a backward scan, ends the file  | central directory at `[cd_offset, cd_offset + cd_size)`, exact |
//! | central directory | 1    | `Vec<cd_entry>` filling its interval     | one local file per entry, at `local_header_offset` |
//! | local file        | 0    | header, name, extra, compressed data     | nothing |
//!
//! Links: the central directory has `entries_total` entries, and each local
//! header agrees with its central directory entry on the name and on the
//! shared `file_info` (version needed, flags, method, times, CRC-32, sizes). Layout: the local files in central directory
//! order, then the central directory, then the end record, with no gaps.
//!
//! The record formats are in `vest/zip_regions.vest` (single disk, no
//! encryption, no data descriptors, stored or deflated, no ZIP64).
use crate::gen::zip_regions::*;
use crate::ranked::*;
use vest_lib::core::exec::bytes_eq;
use vest_lib::core::exec::parser::Parser;
use vest_lib::core::proof::*;
use vest_lib::core::spec::*;
use vstd::prelude::*;

verus! {

/// Size of an end of central directory record without its comment.
pub const EOCD_MIN: usize = 22;

/// Longest archive comment.
pub const COMMENT_MAX: usize = 65535;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ZipKind {
    Eocd,
    CentralDir,
    LocalFile,
}

impl View for ZipKind {
    type V = ZipKind;

    open spec fn view(&self) -> ZipKind {
        *self
    }
}

pub enum ZipVal {
    Eocd(EocdSpec),
    CentralDir(CentralDirectorySpec),
    LocalFile(LocalFileSpec),
}

pub enum ZipNode<'a> {
    Eocd(Eocd<'a>),
    CentralDir(CentralDirectory<'a>),
    LocalFile(LocalFile<'a>),
}

impl<'a> DeepView for ZipNode<'a> {
    type V = ZipVal;

    open spec fn deep_view(&self) -> ZipVal {
        match self {
            ZipNode::Eocd(e) => ZipVal::Eocd(e.deep_view()),
            ZipNode::CentralDir(c) => ZipVal::CentralDir(c.deep_view()),
            ZipNode::LocalFile(l) => ZipVal::LocalFile(l.deep_view()),
        }
    }
}

/// A local header agrees with its central directory entry: the same
/// `file_info` (version needed, flags, method, times, CRC-32, sizes) and the
/// same name.
pub open spec fn entry_agrees(e: CdEntrySpec, l: LocalFileSpec) -> bool {
    &&& e.info == l.info
    &&& e.name == l.name
}

pub open spec fn eocd_scan_lo(len: int) -> int {
    if len - EOCD_MIN > COMMENT_MAX {
        len - EOCD_MIN - COMMENT_MAX
    } else {
        0
    }
}

pub struct Zip;

impl RankedFormat for Zip {
    type Kind = ZipKind;

    type Val = ZipVal;

    open spec fn rank(&self, k: ZipKind) -> nat {
        match k {
            ZipKind::Eocd => 2,
            ZipKind::CentralDir => 1,
            ZipKind::LocalFile => 0,
        }
    }

    open spec fn spec_parse_node(&self, k: ZipKind, s: Seq<u8>) -> Option<(int, ZipVal)> {
        match k {
            ZipKind::Eocd => match EocdFmt.spec_parse(s) {
                Some((n, v)) => Some((n, ZipVal::Eocd(v))),
                None => None,
            },
            ZipKind::CentralDir => match CentralDirectoryFmt.spec_parse(s) {
                Some((n, v)) => Some((n, ZipVal::CentralDir(v))),
                None => None,
            },
            ZipKind::LocalFile => match LocalFileFmt.spec_parse(s) {
                Some((n, v)) => Some((n, ZipVal::LocalFile(v))),
                None => None,
            },
        }
    }

    open spec fn spec_serialize_node(&self, k: ZipKind, v: ZipVal) -> Seq<u8> {
        match v {
            ZipVal::Eocd(e) => EocdFmt.spec_serialize(e),
            ZipVal::CentralDir(c) => CentralDirectoryFmt.spec_serialize(c),
            ZipVal::LocalFile(l) => LocalFileFmt.spec_serialize(l),
        }
    }

    open spec fn spec_node_ok(&self, k: ZipKind, v: ZipVal) -> bool {
        match v {
            ZipVal::Eocd(e) => e.entries_disk == e.entries_total,
            _ => true,
        }
    }

    open spec fn spec_jumps(&self, k: ZipKind, v: ZipVal, root_len: int) -> Seq<SJump<ZipKind>> {
        match v {
            ZipVal::Eocd(e) => seq![
                SJump {
                    kind: ZipKind::CentralDir,
                    l: e.cd_offset as int,
                    r: e.cd_offset as int + e.cd_size as int,
                    exact: true,
                },
            ],
            ZipVal::CentralDir(c) => Seq::new(
                c.entries.len(),
                |i: int|
                    SJump {
                        kind: ZipKind::LocalFile,
                        l: c.entries[i].attrs.local_header_offset as int,
                        r: root_len,
                        exact: false,
                    },
            ),
            ZipVal::LocalFile(_) => seq![],
        }
    }

    open spec fn spec_link_ok(
        &self,
        k: ZipKind,
        v: ZipVal,
        i: int,
        child_kind: ZipKind,
        child_val: ZipVal,
    ) -> bool {
        match v {
            ZipVal::Eocd(e) => match child_val {
                ZipVal::CentralDir(c) => c.entries.len() == e.entries_total as int,
                _ => false,
            },
            ZipVal::CentralDir(c) => match child_val {
                ZipVal::LocalFile(l) => 0 <= i < c.entries.len() && entry_agrees(c.entries[i], l),
                _ => false,
            },
            ZipVal::LocalFile(_) => false,
        }
    }

    open spec fn spec_root(&self, root: Seq<u8>) -> Option<SJump<ZipKind>> {
        if root.len() < EOCD_MIN {
            None
        } else {
            match spec_scan_back(
                EocdFmt,
                root,
                eocd_scan_lo(root.len() as int),
                root.len() - EOCD_MIN,
            ) {
                Some((p, _)) => Some(
                    SJump { kind: ZipKind::Eocd, l: p, r: root.len() as int, exact: true },
                ),
                None => None,
            }
        }
    }

    open spec fn spec_layout(&self, rs: Seq<SRegion<ZipKind, ZipVal>>) -> Seq<int> {
        if rs.len() < 2 {
            Seq::new(rs.len(), |i: int| i)
        } else {
            Seq::new((rs.len() - 2) as nat, |i: int| i + 2).push(1).push(0)
        }
    }

    proof fn lemma_parse_node_serialize(&self, k: ZipKind, s: Seq<u8>) {
        match k {
            ZipKind::Eocd => {
                EocdFmt.lemma_parse_safe(s);
                EocdFmt.theorem_parse_serialize_roundtrip(s);
            },
            ZipKind::CentralDir => {
                CentralDirectoryFmt.lemma_parse_safe(s);
                CentralDirectoryFmt.theorem_parse_serialize_roundtrip(s);
            },
            ZipKind::LocalFile => {
                LocalFileFmt.lemma_parse_safe(s);
                LocalFileFmt.theorem_parse_serialize_roundtrip(s);
            },
        }
    }
}

/// The ranks are a static property of this format: every jump out of a
/// parsed node goes to a lower rank, so the driver's rank check never
/// rejects a ZIP file.
pub proof fn lemma_zip_jumps_descend(k: ZipKind, s: Seq<u8>, root_len: int)
    ensures
        Zip.spec_parse_node(k, s) matches Some((_, v)) ==> forall|i: int|
            0 <= i < Zip.spec_jumps(k, v, root_len).len() ==> Zip.rank(
                #[trigger] Zip.spec_jumps(k, v, root_len)[i].kind,
            ) < Zip.rank(k),
{
}

impl<'a> RankedFormatExec<'a> for Zip {
    type EKind = ZipKind;

    type EVal = ZipNode<'a>;

    fn rank_of(&self, k: ZipKind) -> (r: u64) {
        match k {
            ZipKind::Eocd => 2,
            ZipKind::CentralDir => 1,
            ZipKind::LocalFile => 0,
        }
    }

    fn parse_node(&self, k: ZipKind, s: &'a [u8]) -> (res: Option<(usize, ZipNode<'a>)>) {
        match k {
            ZipKind::Eocd => match EocdFmt.parse(&s) {
                Ok((n, v)) => Some((n, ZipNode::Eocd(v))),
                Err(_) => None,
            },
            ZipKind::CentralDir => match CentralDirectoryFmt.parse(&s) {
                Ok((n, v)) => Some((n, ZipNode::CentralDir(v))),
                Err(_) => None,
            },
            ZipKind::LocalFile => match LocalFileFmt.parse(&s) {
                Ok((n, v)) => Some((n, ZipNode::LocalFile(v))),
                Err(_) => None,
            },
        }
    }

    fn node_ok(&self, k: ZipKind, v: &ZipNode<'a>) -> (b: bool) {
        match v {
            ZipNode::Eocd(e) => {
                proof {
                    e.lemma_deep_view_fields();
                }
                e.entries_disk == e.entries_total
            },
            _ => true,
        }
    }

    fn jumps(&self, k: ZipKind, v: &ZipNode<'a>, root_len: usize) -> (js: Vec<EJump<ZipKind>>) {
        match v {
            ZipNode::Eocd(e) => {
                proof {
                    e.lemma_deep_view_fields();
                }
                let l = e.cd_offset as usize;
                let r = l + e.cd_size as usize;
                let mut js = Vec::new();
                js.push(EJump { kind: ZipKind::CentralDir, l, r, exact: true });
                js
            },
            ZipNode::CentralDir(c) => {
                proof {
                    c.lemma_deep_view_fields();
                }
                let ghost sjs = self.spec_jumps(k@, v.deep_view(), root_len as int);
                let mut js: Vec<EJump<ZipKind>> = Vec::new();
                let mut i: usize = 0;
                while i < c.entries.len()
                    invariant
                        0 <= i <= c.entries@.len(),
                        c.deep_view().entries == c.entries.deep_view(),
                        sjs == self.spec_jumps(k@, v.deep_view(), root_len as int),
                        *v == ZipNode::CentralDir(*c),
                        sjs.len() == c.entries@.len(),
                        js@.len() == i,
                        forall|q: int| 0 <= q < i ==> #[trigger] js@[q]@ == sjs[q],
                    decreases c.entries@.len() - i,
                {
                    let e = &c.entries[i];
                    proof {
                        e.lemma_deep_view_fields();
                        e.attrs.lemma_deep_view_fields();
                        assert(c.entries.deep_view()[i as int] == e.deep_view());
                    }
                    js.push(
                        EJump {
                            kind: ZipKind::LocalFile,
                            l: e.attrs.local_header_offset as usize,
                            r: root_len,
                            exact: false,
                        },
                    );
                    i += 1;
                }
                js
            },
            ZipNode::LocalFile(_) => Vec::new(),
        }
    }

    fn link_ok(
        &self,
        k: ZipKind,
        v: &ZipNode<'a>,
        i: usize,
        child_kind: ZipKind,
        child_val: &ZipNode<'a>,
    ) -> (b: bool) {
        match v {
            ZipNode::Eocd(e) => match child_val {
                ZipNode::CentralDir(c) => {
                    proof {
                        e.lemma_deep_view_fields();
                        c.lemma_deep_view_fields();
                    }
                    c.entries.len() == e.entries_total as usize
                },
                _ => false,
            },
            ZipNode::CentralDir(c) => match child_val {
                ZipNode::LocalFile(l) => {
                    proof {
                        c.lemma_deep_view_fields();
                        l.lemma_deep_view_fields();
                    }
                    if i >= c.entries.len() {
                        return false;
                    }
                    let e = &c.entries[i];
                    proof {
                        e.lemma_deep_view_fields();
                        e.info.lemma_deep_view_fields();
                        l.info.lemma_deep_view_fields();
                        assert(c.entries.deep_view()[i as int] == e.deep_view());
                    }
                    let a = &e.info;
                    let b = &l.info;
                    let same_info = a.version_needed == b.version_needed && a.flags == b.flags
                        && a.method == b.method && a.mod_time == b.mod_time && a.mod_date
                        == b.mod_date && a.crc32 == b.crc32 && a.compressed_size
                        == b.compressed_size && a.uncompressed_size == b.uncompressed_size;
                    proof {
                        if same_info {
                            assert(e.info.deep_view() =~= l.info.deep_view());
                        }
                    }
                    same_info && bytes_eq(e.name, l.name)
                },
                _ => false,
            },
            ZipNode::LocalFile(_) => false,
        }
    }

    fn root(&self, root: &'a [u8]) -> (j: Option<EJump<ZipKind>>) {
        if root.len() < EOCD_MIN {
            return None;
        }
        let hi = root.len() - EOCD_MIN;
        let lo = if hi > COMMENT_MAX {
            hi - COMMENT_MAX
        } else {
            0
        };
        match scan_back(&EocdFmt, root, lo, hi) {
            Some((p, _)) => Some(EJump { kind: ZipKind::Eocd, l: p, r: root.len(), exact: true }),
            None => None,
        }
    }

    fn layout(&self, rs: &Vec<ERegion<ZipKind, ZipNode<'a>>>) -> (o: Vec<usize>) {
        let n = rs.len();
        let ghost spec = self.spec_layout(dv_regions(rs@));
        let mut o: Vec<usize> = Vec::new();
        if n < 2 {
            let mut i: usize = 0;
            while i < n
                invariant
                    0 <= i <= n,
                    n == rs@.len(),
                    n < 2,
                    o@.len() == i,
                    forall|q: int| 0 <= q < i ==> #[trigger] o@[q] as int == q,
                decreases n - i,
            {
                o.push(i);
                i += 1;
            }
        } else {
            let mut i: usize = 2;
            while i < n
                invariant
                    2 <= i <= n,
                    n == rs@.len(),
                    o@.len() == i - 2,
                    forall|q: int| 0 <= q < i - 2 ==> #[trigger] o@[q] as int == q + 2,
                decreases n - i,
            {
                o.push(i);
                i += 1;
            }
            o.push(1);
            o.push(0);
        }
        assert(dv_order(o@) =~= spec);
        o
    }
}

/// Parses `b` under the strict ZIP profile; see [`spec_parse_file`].
pub fn parse_zip<'a>(b: &'a [u8]) -> (res: Option<Vec<ERegion<ZipKind, ZipNode<'a>>>>)
    ensures
        match res {
            Some(rs) => spec_parse_file(Zip, b@) == Some(
                SFile { len: b@.len() as int, regions: dv_regions(rs@) },
            ),
            None => spec_parse_file(Zip, b@) is None,
        },
{
    parse_file(&Zip, b)
}

/// The strict profile is non-malleable: two accepted archives with the same
/// regions and values are the same bytes.
pub proof fn theorem_zip_non_malleable(b1: Seq<u8>, b2: Seq<u8>)
    requires
        spec_parse_file(Zip, b1) is Some,
        spec_parse_file(Zip, b1) == spec_parse_file(Zip, b2),
    ensures
        b1 == b2,
{
    theorem_non_malleable(Zip, b1, b2);
}

} // verus!
