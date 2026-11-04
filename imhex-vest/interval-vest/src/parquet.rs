//! A strict Parquet profile as a ranked format.
//!
//! | kind          | rank | region                                                | jumps to |
//! |---------------|------|-------------------------------------------------------|----------|
//! | tail          | 2    | the last 8 bytes: metadata length, `PAR1`             | the FileMetaData before it, exact |
//! | FileMetaData  | 1    | one Thrift struct filling its interval                | `PAR1` at 0; per column chunk: its pages, its inline copy, its column index, offset index, bloom filter |
//! | head          | 0    | `PAR1`                                                | — |
//! | column chunk  | 0    | page header + page data, repeated to the end          | — |
//! | inline copy   | 0    | a Thrift struct at `file_offset`                      | — |
//! | column / offset index | 0 | a Thrift struct filling its interval            | — |
//! | bloom filter  | 0    | a Thrift header, then `numBytes` of bitset            | — |
//!
//! Checks on the FileMetaData: no encryption; every column chunk is in this
//! file (no `file_path`); its offsets, sizes and counts are non-negative i64s
//! of the right Thrift type; an index offset comes with its length;
//! `num_rows` is the sum of the row groups' `num_rows`.
//!
//! Checks on a column chunk, against the values the FileMetaData passes
//! down in the jump (the kind carries them):
//!
//! - a dictionary page first exactly when `dictionary_page_offset` is set,
//!   data pages (v1 or v2) otherwise;
//! - `data_page_offset` is the first data page (or 0 if there is none);
//! - the data pages' `num_values` add up to the chunk's `num_values`;
//! - headers plus uncompressed page sizes add up to
//!   `total_uncompressed_size`.
//!
//! Older writers put a copy of the ColumnChunk, or of its ColumnMetaData,
//! after the chunk data and point `file_offset` at it. When `file_offset`
//! is the end of the chunk, that copy must be there and must equal the
//! footer's.
//!
//! Layout: all regions sorted by offset ([`spec_sorted_layout`]), which must
//! tile the file.
use crate::gen::parquet_regions::*;
use crate::ranked::*;
use crate::thrift::*;
use vest_lib::core::exec::parser::Parser;
use vest_lib::core::proof::*;
use vest_lib::core::spec::*;
use vstd::prelude::*;
use vstd::slice::slice_subrange;

verus! {

/// Nesting depth allowed in a Thrift struct of the file.
pub const MAX_DEPTH: u64 = 16;

pub open spec fn maxd() -> nat {
    MAX_DEPTH as nat
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PKind {
    Tail,
    Meta,
    Head,
    /// The values the FileMetaData gives for this chunk.
    Chunk { start: u64, dict: bool, num_values: u64, uncompressed: u64, dpo: u64 },
    /// The copy of the ColumnChunk `col` of row group `rg`.
    InlineCopy { rg: u64, col: u64 },
    ColumnIndex,
    OffsetIndex,
    Bloom,
}

impl View for PKind {
    type V = PKind;

    open spec fn view(&self) -> PKind {
        *self
    }
}

pub struct Page {
    pub header: TVal,
    pub header_len: int,
    pub body: Seq<u8>,
}

pub enum PVal {
    Head(HeadSpec),
    Tail(TailSpec),
    Struct(TVal),
    Pages(Seq<Page>),
    Bloom(TVal, Seq<u8>),
}

// ============================================================
// Typed access to Thrift values
// ============================================================

pub open spec fn fields_of(v: TVal) -> Seq<TField> {
    match v {
        TVal::Struct(fs) => fs,
        _ => seq![],
    }
}

/// A list of structs (an empty list may declare any element type).
pub open spec fn list_of(v: Option<TVal>) -> Option<Seq<TVal>> {
    match v {
        Some(TVal::List(et, xs)) => if et == 12 || xs.len() == 0 {
            Some(xs)
        } else {
            None
        },
        _ => None,
    }
}

pub open spec fn nn64(v: Option<TVal>) -> Option<int> {
    match v {
        Some(TVal::I64(x)) => if x >= 0 {
            Some(x)
        } else {
            None
        },
        _ => None,
    }
}

pub open spec fn nn32(v: Option<TVal>) -> Option<int> {
    match v {
        Some(TVal::I32(x)) => if x >= 0 {
            Some(x)
        } else {
            None
        },
        _ => None,
    }
}

/// Absent, or a non-negative i64.
pub open spec fn opt64(v: Option<TVal>) -> Option<Option<int>> {
    match v {
        None => Some(None),
        Some(TVal::I64(x)) => if x >= 0 {
            Some(Some(x))
        } else {
            None
        },
        _ => None,
    }
}

/// Absent, or a non-negative i32.
pub open spec fn opt32(v: Option<TVal>) -> Option<Option<int>> {
    match v {
        None => Some(None),
        Some(TVal::I32(x)) => if x >= 0 {
            Some(Some(x))
        } else {
            None
        },
        _ => None,
    }
}

// ============================================================
// FileMetaData
// ============================================================

/// What the FileMetaData says about one column chunk.
pub struct Desc {
    pub start: int,
    pub size: int,
    pub num_values: int,
    pub uncompressed: int,
    pub dpo: int,
    pub dict: bool,
    pub copy: bool,
    pub fo: int,
    pub col_index: Option<(int, int)>,
    pub off_index: Option<(int, int)>,
    pub bloom: Option<(int, Option<int>)>,
}

/// An offset and a length, both present or both absent.
pub open spec fn index_pair(o: Option<Option<int>>, l: Option<Option<int>>) -> Option<
    Option<(int, int)>,
> {
    match (o, l) {
        (Some(None), Some(None)) => Some(None),
        (Some(Some(a)), Some(Some(b))) => Some(Some((a, b))),
        _ => None,
    }
}

/// A bloom filter offset with an optional length; no length without offset.
pub open spec fn bloom_pair(o: Option<Option<int>>, l: Option<Option<int>>) -> Option<
    Option<(int, Option<int>)>,
> {
    match (o, l) {
        (Some(None), Some(None)) => Some(None),
        (Some(Some(a)), Some(b)) => Some(Some((a, b))),
        _ => None,
    }
}

pub open spec fn desc_of(cc: TVal) -> Option<Desc> {
    let fs = fields_of(cc);
    if !(cc is Struct) || get(fs, 1) is Some || get(fs, 8) is Some || get(fs, 9) is Some {
        None
    } else {
        match (get(fs, 2), get(fs, 3)) {
            (Some(TVal::I64(fo)), Some(TVal::Struct(m))) => {
                let nv = nn64(get(m, 5));
                let unc = nn64(get(m, 6));
                let tcs = nn64(get(m, 7));
                let dpo = nn64(get(m, 9));
                let dict = opt64(get(m, 11));
                let ci = index_pair(opt64(get(fs, 6)), opt32(get(fs, 7)));
                let oi = index_pair(opt64(get(fs, 4)), opt32(get(fs, 5)));
                let bl = bloom_pair(opt64(get(m, 14)), opt32(get(m, 15)));
                if nv is None || unc is None || tcs is None || dpo is None || dict is None
                    || ci is None || oi is None || bl is None {
                    None
                } else {
                    let start = match dict->0 {
                        Some(x) => x,
                        None => dpo->0,
                    };
                    let size = tcs->0;
                    Some(
                        Desc {
                            start,
                            size,
                            num_values: nv->0,
                            uncompressed: unc->0,
                            dpo: dpo->0,
                            dict: dict->0 is Some,
                            copy: fo == start + size && size > 0,
                            fo,
                            col_index: ci->0,
                            off_index: oi->0,
                            bloom: bl->0,
                        },
                    )
                }
            },
            _ => None,
        }
    }
}

pub open spec fn meta_rgs(v: TVal) -> Seq<TVal> {
    match list_of(get(fields_of(v), 4)) {
        Some(xs) => xs,
        None => seq![],
    }
}

pub open spec fn rg_ccs(rg: TVal) -> Seq<TVal> {
    match list_of(get(fields_of(rg), 1)) {
        Some(xs) => xs,
        None => seq![],
    }
}

pub open spec fn ccs_ok(ccs: Seq<TVal>) -> bool
    decreases ccs.len(),
{
    ccs.len() == 0 || (ccs_ok(ccs.drop_last()) && desc_of(ccs.last()) is Some)
}

pub open spec fn rg_ok(rg: TVal) -> bool {
    &&& rg is Struct
    &&& nn64(get(fields_of(rg), 3)) is Some
    &&& list_of(get(fields_of(rg), 1)) is Some
    &&& ccs_ok(rg_ccs(rg))
}

pub open spec fn rgs_ok(rgs: Seq<TVal>) -> bool
    decreases rgs.len(),
{
    rgs.len() == 0 || (rgs_ok(rgs.drop_last()) && rg_ok(rgs.last()))
}

pub open spec fn rg_rows(rg: TVal) -> int {
    match nn64(get(fields_of(rg), 3)) {
        Some(x) => x,
        None => 0,
    }
}

pub open spec fn sum_rows(rgs: Seq<TVal>) -> int
    decreases rgs.len(),
{
    if rgs.len() == 0 {
        0
    } else {
        sum_rows(rgs.drop_last()) + rg_rows(rgs.last())
    }
}

pub open spec fn meta_ok(v: TVal) -> bool {
    &&& v is Struct
    &&& get(fields_of(v), 8) is None
    &&& list_of(get(fields_of(v), 4)) is Some
    &&& rgs_ok(meta_rgs(v))
    &&& nn64(get(fields_of(v), 3)) == Some(sum_rows(meta_rgs(v)))
}

pub open spec fn desc_jumps(d: Desc, rg: int, col: int, len: int) -> Seq<SJump<PKind>> {
    seq![
        SJump {
            kind: PKind::Chunk {
                start: d.start as u64,
                dict: d.dict,
                num_values: d.num_values as u64,
                uncompressed: d.uncompressed as u64,
                dpo: d.dpo as u64,
            },
            l: d.start,
            r: d.start + d.size,
            exact: true,
        },
    ] + (if d.copy {
        seq![
            SJump {
                kind: PKind::InlineCopy { rg: rg as u64, col: col as u64 },
                l: d.fo,
                r: len,
                exact: false,
            },
        ]
    } else {
        seq![]
    }) + (match d.col_index {
        Some((o, l)) => seq![SJump { kind: PKind::ColumnIndex, l: o, r: o + l, exact: true }],
        None => seq![],
    }) + (match d.off_index {
        Some((o, l)) => seq![SJump { kind: PKind::OffsetIndex, l: o, r: o + l, exact: true }],
        None => seq![],
    }) + (match d.bloom {
        Some((o, Some(l))) => seq![SJump { kind: PKind::Bloom, l: o, r: o + l, exact: true }],
        Some((o, None)) => seq![SJump { kind: PKind::Bloom, l: o, r: len, exact: false }],
        None => seq![],
    })
}

pub open spec fn cc_jumps(cc: TVal, rg: int, col: int, len: int) -> Seq<SJump<PKind>> {
    match desc_of(cc) {
        Some(d) => desc_jumps(d, rg, col, len),
        None => seq![],
    }
}

pub open spec fn ccs_jumps(ccs: Seq<TVal>, rg: int, len: int) -> Seq<SJump<PKind>>
    decreases ccs.len(),
{
    if ccs.len() == 0 {
        seq![]
    } else {
        ccs_jumps(ccs.drop_last(), rg, len) + cc_jumps(ccs.last(), rg, ccs.len() - 1, len)
    }
}

pub open spec fn rgs_jumps(rgs: Seq<TVal>, len: int) -> Seq<SJump<PKind>>
    decreases rgs.len(),
{
    if rgs.len() == 0 {
        seq![]
    } else {
        rgs_jumps(rgs.drop_last(), len) + ccs_jumps(rg_ccs(rgs.last()), rgs.len() - 1, len)
    }
}

pub open spec fn head_jump() -> SJump<PKind> {
    SJump { kind: PKind::Head, l: 0, r: 4, exact: true }
}

pub open spec fn meta_jumps(v: TVal, len: int) -> Seq<SJump<PKind>> {
    seq![head_jump()] + rgs_jumps(meta_rgs(v), len)
}

/// The inline copy is the ColumnChunk's `file_offset` and `meta_data`, or
/// the `meta_data` alone.
pub open spec fn copy_ok(v: TVal, rg: int, col: int, c: TVal) -> bool {
    let rgs = meta_rgs(v);
    if !(0 <= rg < rgs.len()) {
        false
    } else {
        let ccs = rg_ccs(rgs[rg]);
        if !(0 <= col < ccs.len()) {
            false
        } else {
            let fs = fields_of(ccs[col]);
            match (get(fs, 2), get(fs, 3)) {
                (Some(fo), Some(md)) => c == TVal::Struct(
                    seq![TField { id: 2, val: fo }, TField { id: 3, val: md }],
                ) || c == md,
                _ => false,
            }
        }
    }
}

// ============================================================
// Column chunks
// ============================================================

pub open spec fn page_csize(h: TVal) -> Option<int> {
    nn32(get(fields_of(h), 3))
}

/// Pages from position `i` to the end of the chunk.
pub open spec fn spec_pages(s: Seq<u8>, i: int) -> Option<Seq<Page>>
    decreases s.len() - i,
{
    if i < 0 || i > s.len() {
        None
    } else if i == s.len() {
        Some(seq![])
    } else {
        match spec_struct(s, i, maxd()) {
            None => None,
            Some((j, h)) => match page_csize(h) {
                None => None,
                Some(c) => if j <= i || j + c > s.len() {
                    None
                } else {
                    match spec_pages(s, j + c) {
                        None => None,
                        Some(ps) => Some(
                            seq![Page { header: h, header_len: j - i, body: s.subrange(j, j + c) }]
                                + ps,
                        ),
                    }
                },
            },
        }
    }
}

pub open spec fn ser_pages(ps: Seq<Page>) -> Seq<u8>
    decreases ps.len(),
{
    if ps.len() == 0 {
        seq![]
    } else {
        ser_value(ps[0].header, maxd()) + ps[0].body + ser_pages(ps.drop_first())
    }
}

pub open spec fn page_type(p: Page) -> Option<int> {
    match get(fields_of(p.header), 1) {
        Some(TVal::I32(x)) => Some(x),
        _ => None,
    }
}

/// `num_values` of a data page (v1: field 5, v2: field 8); 0 for a
/// dictionary page.
pub open spec fn page_nvals(p: Page) -> Option<int> {
    let fs = fields_of(p.header);
    let t = page_type(p);
    if t == Some(0int) {
        match get(fs, 5) {
            Some(TVal::Struct(h)) => nn32(get(h, 1)),
            _ => None,
        }
    } else if t == Some(3int) {
        match get(fs, 8) {
            Some(TVal::Struct(h)) => nn32(get(h, 1)),
            _ => None,
        }
    } else if t == Some(2int) {
        Some(0)
    } else {
        None
    }
}

pub open spec fn page_usize(p: Page) -> Option<int> {
    nn32(get(fields_of(p.header), 2))
}

pub open spec fn page_len(p: Page) -> int {
    p.header_len + p.body.len()
}

/// The page at position `k`: a dictionary page first when the chunk has a
/// dictionary, data pages everywhere else.
pub open spec fn page_ok(p: Page, k: int, dict: bool) -> bool {
    &&& page_nvals(p) is Some
    &&& page_usize(p) is Some
    &&& if k == 0 && dict {
        page_type(p) == Some(2)
    } else {
        page_type(p) == Some(0) || page_type(p) == Some(3)
    }
}

pub open spec fn sum_nvals(ps: Seq<Page>) -> int
    decreases ps.len(),
{
    if ps.len() == 0 {
        0
    } else {
        sum_nvals(ps.drop_last()) + match page_nvals(ps.last()) {
            Some(x) => x,
            None => 0,
        }
    }
}

pub open spec fn sum_unc(ps: Seq<Page>) -> int
    decreases ps.len(),
{
    if ps.len() == 0 {
        0
    } else {
        sum_unc(ps.drop_last()) + ps.last().header_len + match page_usize(ps.last()) {
            Some(x) => x,
            None => 0,
        }
    }
}

pub open spec fn chunk_ok(
    start: int,
    dict: bool,
    nv: int,
    unc: int,
    dpo: int,
    ps: Seq<Page>,
) -> bool {
    &&& forall|k: int| 0 <= k < ps.len() ==> #[trigger] page_ok(ps[k], k, dict)
    &&& dict ==> ps.len() >= 1
    &&& sum_nvals(ps) == nv
    &&& sum_unc(ps) == unc
    &&& if (dict && ps.len() >= 2) {
        dpo == start + page_len(ps[0])
    } else if (!dict && ps.len() >= 1) {
        dpo == start
    } else {
        dpo == 0 || dpo == start
    }
}

pub open spec fn spec_bloom(s: Seq<u8>) -> Option<(int, PVal)> {
    match spec_struct(s, 0, maxd()) {
        Some((j, h)) => match nn32(get(fields_of(h), 1)) {
            Some(nb) => if j + nb <= s.len() {
                Some((j + nb, PVal::Bloom(h, s.subrange(j, j + nb))))
            } else {
                None
            },
            None => None,
        },
        None => None,
    }
}

pub proof fn lemma_pages_ps(s: Seq<u8>, i: int)
    ensures
        spec_pages(s, i) matches Some(ps) ==> ser_pages(ps) == s.subrange(i, s.len() as int),
    decreases s.len() - i,
{
    if 0 <= i < s.len() {
        lemma_value_ps(s, i, 12, maxd());
        if let Some((j, h)) = spec_struct(s, i, maxd()) {
            if let Some(c) = page_csize(h) {
                if j > i && j + c <= s.len() {
                    lemma_pages_ps(s, j + c);
                    if let Some(rest) = spec_pages(s, j + c) {
                        let ps = seq![Page { header: h, header_len: j - i, body: s.subrange(j, j + c) }]
                            + rest;
                        assert(ps.drop_first() =~= rest);
                        assert(s.subrange(i, s.len() as int) =~= s.subrange(i, j) + s.subrange(j, j + c)
                            + s.subrange(j + c, s.len() as int));
                    }
                }
            }
        }
    } else if i == s.len() {
        assert(s.subrange(i, i) =~= Seq::<u8>::empty());
    }
}

// ============================================================
// The ranked format
// ============================================================

pub struct Parquet;

impl RankedFormat for Parquet {
    type Kind = PKind;

    type Val = PVal;

    open spec fn rank(&self, k: PKind) -> nat {
        match k {
            PKind::Tail => 2,
            PKind::Meta => 1,
            _ => 0,
        }
    }

    open spec fn spec_parse_node(&self, k: PKind, s: Seq<u8>) -> Option<(int, PVal)> {
        match k {
            PKind::Tail => match TailFmt.spec_parse(s) {
                Some((n, v)) => Some((n, PVal::Tail(v))),
                None => None,
            },
            PKind::Head => match HeadFmt.spec_parse(s) {
                Some((n, v)) => Some((n, PVal::Head(v))),
                None => None,
            },
            PKind::Chunk { .. } => match spec_pages(s, 0) {
                Some(ps) => Some((s.len() as int, PVal::Pages(ps))),
                None => None,
            },
            PKind::Bloom => spec_bloom(s),
            _ => match spec_struct(s, 0, maxd()) {
                Some((n, v)) => Some((n, PVal::Struct(v))),
                None => None,
            },
        }
    }

    open spec fn spec_serialize_node(&self, k: PKind, v: PVal) -> Seq<u8> {
        match v {
            PVal::Head(h) => HeadFmt.spec_serialize(h),
            PVal::Tail(t) => TailFmt.spec_serialize(t),
            PVal::Struct(t) => ser_value(t, maxd()),
            PVal::Pages(ps) => ser_pages(ps),
            PVal::Bloom(h, b) => ser_value(h, maxd()) + b,
        }
    }

    open spec fn spec_node_ok(&self, k: PKind, v: PVal) -> bool {
        match k {
            PKind::Meta => match v {
                PVal::Struct(t) => meta_ok(t),
                _ => false,
            },
            PKind::Chunk { start, dict, num_values, uncompressed, dpo } => match v {
                PVal::Pages(ps) => chunk_ok(
                    start as int,
                    dict,
                    num_values as int,
                    uncompressed as int,
                    dpo as int,
                    ps,
                ),
                _ => false,
            },
            _ => true,
        }
    }

    open spec fn spec_jumps(&self, k: PKind, v: PVal, root_len: int) -> Seq<SJump<PKind>> {
        match v {
            PVal::Tail(t) => {
                let m = t.metadata_len as int;
                if m <= root_len - 8 {
                    seq![SJump { kind: PKind::Meta, l: root_len - 8 - m, r: root_len - 8, exact: true }]
                } else {
                    seq![SJump { kind: PKind::Meta, l: 0, r: 0, exact: true }]
                }
            },
            PVal::Struct(t) => match k {
                PKind::Meta => meta_jumps(t, root_len),
                _ => seq![],
            },
            _ => seq![],
        }
    }

    open spec fn spec_link_ok(
        &self,
        k: PKind,
        v: PVal,
        i: int,
        child_kind: PKind,
        child_val: PVal,
    ) -> bool {
        match child_kind {
            PKind::InlineCopy { rg, col } => match (v, child_val) {
                (PVal::Struct(t), PVal::Struct(c)) => copy_ok(t, rg as int, col as int, c),
                _ => false,
            },
            _ => true,
        }
    }

    open spec fn spec_root(&self, root: Seq<u8>) -> Option<SJump<PKind>> {
        if root.len() < 8 {
            None
        } else {
            Some(SJump { kind: PKind::Tail, l: root.len() - 8, r: root.len() as int, exact: true })
        }
    }

    open spec fn spec_layout(&self, rs: Seq<SRegion<PKind, PVal>>) -> Seq<int> {
        spec_sorted_layout(rs)
    }

    proof fn lemma_parse_node_serialize(&self, k: PKind, s: Seq<u8>) {
        match k {
            PKind::Tail => {
                TailFmt.lemma_parse_safe(s);
                TailFmt.theorem_parse_serialize_roundtrip(s);
            },
            PKind::Head => {
                HeadFmt.lemma_parse_safe(s);
                HeadFmt.theorem_parse_serialize_roundtrip(s);
            },
            PKind::Chunk { .. } => {
                lemma_pages_ps(s, 0);
                assert(s.take(s.len() as int) =~= s.subrange(0, s.len() as int));
            },
            PKind::Bloom => {
                lemma_value_ps(s, 0, 12, maxd());
                if let Some((j, h)) = spec_struct(s, 0, maxd()) {
                    if let Some(nb) = nn32(get(fields_of(h), 1)) {
                        if j + nb <= s.len() {
                            assert(s.take(j + nb) =~= s.subrange(0, j) + s.subrange(j, j + nb));
                        }
                    }
                }
            },
            _ => {
                lemma_value_ps(s, 0, 12, maxd());
                if let Some((j, h)) = spec_struct(s, 0, maxd()) {
                    assert(s.take(j) =~= s.subrange(0, j));
                }
            },
        }
    }
}

/// The ranks are a static property of this format.
pub proof fn lemma_parquet_jumps_descend(k: PKind, s: Seq<u8>, root_len: int)
    ensures
        Parquet.spec_parse_node(k, s) matches Some((_, v)) ==> forall|i: int|
            0 <= i < Parquet.spec_jumps(k, v, root_len).len() ==> Parquet.rank(
                #[trigger] Parquet.spec_jumps(k, v, root_len)[i].kind,
            ) < Parquet.rank(k),
{
    if let Some((_, v)) = Parquet.spec_parse_node(k, s) {
        if k == PKind::Meta {
            if let PVal::Struct(t) = v {
                lemma_rgs_jumps_rank(meta_rgs(t), root_len);
                lemma_leaf_concat(seq![head_jump()], rgs_jumps(meta_rgs(t), root_len));
            }
        }
    }
}

/// Every jump goes to a kind of rank 0.
pub open spec fn leaf_jumps(js: Seq<SJump<PKind>>) -> bool {
    forall|i: int| 0 <= i < js.len() ==> Parquet.rank(#[trigger] js[i].kind) == 0
}

pub proof fn lemma_leaf_concat(a: Seq<SJump<PKind>>, b: Seq<SJump<PKind>>)
    requires
        leaf_jumps(a),
        leaf_jumps(b),
    ensures
        leaf_jumps(a + b),
{
    assert forall|i: int| 0 <= i < (a + b).len() implies Parquet.rank(#[trigger] (a + b)[i].kind)
        == 0 by {
        if i < a.len() {
            assert((a + b)[i] == a[i]);
        } else {
            assert((a + b)[i] == b[i - a.len()]);
        }
    }
}

pub proof fn lemma_desc_jumps_rank(d: Desc, rg: int, col: int, len: int)
    ensures
        leaf_jumps(desc_jumps(d, rg, col, len)),
{
    let a = seq![
        SJump {
            kind: PKind::Chunk {
                start: d.start as u64,
                dict: d.dict,
                num_values: d.num_values as u64,
                uncompressed: d.uncompressed as u64,
                dpo: d.dpo as u64,
            },
            l: d.start,
            r: d.start + d.size,
            exact: true,
        },
    ];
    let b = if d.copy {
        seq![
            SJump {
                kind: PKind::InlineCopy { rg: rg as u64, col: col as u64 },
                l: d.fo,
                r: len,
                exact: false,
            },
        ]
    } else {
        seq![]
    };
    let c = match d.col_index {
        Some((o, l)) => seq![SJump { kind: PKind::ColumnIndex, l: o, r: o + l, exact: true }],
        None => seq![],
    };
    let e = match d.off_index {
        Some((o, l)) => seq![SJump { kind: PKind::OffsetIndex, l: o, r: o + l, exact: true }],
        None => seq![],
    };
    let f = match d.bloom {
        Some((o, Some(l))) => seq![SJump { kind: PKind::Bloom, l: o, r: o + l, exact: true }],
        Some((o, None)) => seq![SJump { kind: PKind::Bloom, l: o, r: len, exact: false }],
        None => seq![],
    };
    lemma_leaf_concat(a, b);
    lemma_leaf_concat(a + b, c);
    lemma_leaf_concat(a + b + c, e);
    lemma_leaf_concat(a + b + c + e, f);
}

pub proof fn lemma_ccs_jumps_rank(ccs: Seq<TVal>, rg: int, len: int)
    ensures
        leaf_jumps(ccs_jumps(ccs, rg, len)),
    decreases ccs.len(),
{
    if ccs.len() > 0 {
        lemma_ccs_jumps_rank(ccs.drop_last(), rg, len);
        if let Some(d) = desc_of(ccs.last()) {
            lemma_desc_jumps_rank(d, rg, ccs.len() - 1, len);
        }
        lemma_leaf_concat(ccs_jumps(ccs.drop_last(), rg, len), cc_jumps(ccs.last(), rg, ccs.len() - 1, len));
    }
}

pub proof fn lemma_rgs_jumps_rank(rgs: Seq<TVal>, len: int)
    ensures
        leaf_jumps(rgs_jumps(rgs, len)),
    decreases rgs.len(),
{
    if rgs.len() > 0 {
        lemma_rgs_jumps_rank(rgs.drop_last(), len);
        lemma_ccs_jumps_rank(rg_ccs(rgs.last()), rgs.len() - 1, len);
        lemma_leaf_concat(rgs_jumps(rgs.drop_last(), len), ccs_jumps(rg_ccs(rgs.last()), rgs.len() - 1, len));
    }
}

/// The profile is non-malleable: two accepted files with the same regions
/// and values are the same bytes.
pub proof fn theorem_parquet_non_malleable(b1: Seq<u8>, b2: Seq<u8>)
    requires
        spec_parse_file(Parquet, b1) is Some,
        spec_parse_file(Parquet, b1) == spec_parse_file(Parquet, b2),
    ensures
        b1 == b2,
{
    theorem_non_malleable(Parquet, b1, b2);
}

// ============================================================
// Executable values and typed access
// ============================================================

pub struct EPage<'a> {
    pub header: EVal<'a>,
    pub header_len: usize,
    pub body: &'a [u8],
}

pub enum PNode<'a> {
    Head(Head),
    Tail(Tail),
    Struct(EVal<'a>),
    Pages(Vec<EPage<'a>>),
    Bloom(EVal<'a>, &'a [u8]),
}

pub open spec fn epages(ps: Seq<EPage>) -> Seq<Page> {
    Seq::new(
        ps.len(),
        |k: int| Page { header: ev(ps[k].header, maxd()), header_len: ps[k].header_len as int, body: ps[k].body@ },
    )
}

impl<'a> DeepView for PNode<'a> {
    type V = PVal;

    open spec fn deep_view(&self) -> PVal {
        match self {
            PNode::Head(h) => PVal::Head(h.deep_view()),
            PNode::Tail(t) => PVal::Tail(t.deep_view()),
            PNode::Struct(v) => PVal::Struct(ev(*v, maxd())),
            PNode::Pages(ps) => PVal::Pages(epages(ps@)),
            PNode::Bloom(h, b) => PVal::Bloom(ev(*h, maxd()), b@),
        }
    }
}

pub open spec fn opt_ev(o: Option<&EVal>, d: nat) -> Option<TVal> {
    match o {
        Some(v) => Some(ev(*v, d)),
        None => None,
    }
}

proof fn lemma_get_field(fs: Seq<TField>, r: Option<&EVal>, d: nat, id: int)
    requires
        match r {
            Some(v) => get(fs, id) == Some(ev(*v, d)),
            None => get(fs, id) is None,
        },
    ensures
        get(fs, id) == opt_ev(r, d),
{
}

fn x_nn64(o: Option<&EVal>, Ghost(d): Ghost<nat>) -> (r: Option<u64>)
    ensures
        match r {
            Some(x) => nn64(opt_ev(o, d)) == Some(x as int) && x < 0x8000_0000_0000_0000,
            None => nn64(opt_ev(o, d)) is None,
        },
{
    match o {
        Some(EVal::I64(x)) => if *x >= 0 {
            Some(*x as u64)
        } else {
            None
        },
        _ => None,
    }
}

fn x_nn32(o: Option<&EVal>, Ghost(d): Ghost<nat>) -> (r: Option<u64>)
    ensures
        match r {
            Some(x) => nn32(opt_ev(o, d)) == Some(x as int) && x < 0x8000_0000,
            None => nn32(opt_ev(o, d)) is None,
        },
{
    match o {
        Some(EVal::I32(x)) => if *x >= 0 {
            Some(*x as u64)
        } else {
            None
        },
        _ => None,
    }
}

pub open spec fn oo(r: Option<Option<u64>>) -> Option<Option<int>> {
    match r {
        Some(Some(x)) => Some(Some(x as int)),
        Some(None) => Some(None),
        None => None,
    }
}

fn x_opt64(o: Option<&EVal>, Ghost(d): Ghost<nat>) -> (r: Option<Option<u64>>)
    ensures
        oo(r) == opt64(opt_ev(o, d)),
        r matches Some(Some(x)) ==> x < 0x8000_0000_0000_0000,
{
    match o {
        None => Some(None),
        Some(EVal::I64(x)) => if *x >= 0 {
            Some(Some(*x as u64))
        } else {
            None
        },
        _ => None,
    }
}

fn x_opt32(o: Option<&EVal>, Ghost(d): Ghost<nat>) -> (r: Option<Option<u64>>)
    ensures
        oo(r) == opt32(opt_ev(o, d)),
        r matches Some(Some(x)) ==> x < 0x8000_0000,
{
    match o {
        None => Some(None),
        Some(EVal::I32(x)) => if *x >= 0 {
            Some(Some(*x as u64))
        } else {
            None
        },
        _ => None,
    }
}

/// The list of structs at field `id`, if it is one.
fn x_list<'b, 'a>(o: Option<&'b EVal<'a>>, Ghost(d): Ghost<nat>) -> (r: Option<&'b Vec<EVal<'a>>>)
    requires
        d >= 1,
    ensures
        match r {
            Some(xs) => list_of(opt_ev(o, d)) == Some(evs(xs@, (d - 1) as nat)) && exists|et: u8|
                o == Some(&EVal::List(et, *xs)),
            None => list_of(opt_ev(o, d)) is None,
        },
{
    match o {
        Some(EVal::List(et, xs)) => if *et == 12 || xs.len() == 0 {
            Some(xs)
        } else {
            None
        },
        _ => None,
    }
}

// ============================================================
// Executable FileMetaData checks
// ============================================================

pub struct EDesc {
    pub start: u64,
    pub size: u64,
    pub num_values: u64,
    pub uncompressed: u64,
    pub dpo: u64,
    pub dict: bool,
    pub copy: bool,
    pub fo: i64,
    pub col_index: Option<(u64, u64)>,
    pub off_index: Option<(u64, u64)>,
    pub bloom: Option<(u64, Option<u64>)>,
}

pub open spec fn pair_view(p: Option<(u64, u64)>) -> Option<(int, int)> {
    match p {
        Some((a, b)) => Some((a as int, b as int)),
        None => None,
    }
}

pub open spec fn dview(e: EDesc) -> Desc {
    Desc {
        start: e.start as int,
        size: e.size as int,
        num_values: e.num_values as int,
        uncompressed: e.uncompressed as int,
        dpo: e.dpo as int,
        dict: e.dict,
        copy: e.copy,
        fo: e.fo as int,
        col_index: pair_view(e.col_index),
        off_index: pair_view(e.off_index),
        bloom: match e.bloom {
            Some((a, Some(b))) => Some((a as int, Some(b as int))),
            Some((a, None)) => Some((a as int, None)),
            None => None,
        },
    }
}

pub open spec fn desc_bounds(e: EDesc) -> bool {
    &&& e.start < 0x8000_0000_0000_0000
    &&& e.size < 0x8000_0000_0000_0000
    &&& e.copy ==> 0 <= e.fo
    &&& e.col_index matches Some((a, b)) ==> a < 0x8000_0000_0000_0000 && b < 0x8000_0000
    &&& e.off_index matches Some((a, b)) ==> a < 0x8000_0000_0000_0000 && b < 0x8000_0000
    &&& e.bloom matches Some((a, b)) ==> a < 0x8000_0000_0000_0000 && (b matches Some(l) ==> l
        < 0x8000_0000)
}

fn x_pair(o: Option<Option<u64>>, l: Option<Option<u64>>) -> (r: Option<Option<(u64, u64)>>)
    ensures
        match r {
            Some(p) => index_pair(oo(o), oo(l)) == Some(pair_view(p)),
            None => index_pair(oo(o), oo(l)) is None,
        },
        r matches Some(Some((a, b))) ==> o == Some(Some(a)) && l == Some(Some(b)),
{
    match (o, l) {
        (Some(None), Some(None)) => Some(None),
        (Some(Some(a)), Some(Some(b))) => Some(Some((a, b))),
        _ => None,
    }
}

fn x_desc_of(cc: &EVal, Ghost(d): Ghost<nat>) -> (r: Option<EDesc>)
    requires
        d >= 2,
    ensures
        match r {
            Some(e) => desc_of(ev(*cc, d)) == Some(dview(e)) && desc_bounds(e),
            None => desc_of(ev(*cc, d)) is None,
        },
{
    let fs = match cc {
        EVal::Struct(fs) => fs,
        _ => {
            return None;
        },
    };
    let ghost d1 = (d - 1) as nat;
    let ghost sfs = efs(fs@, d1);
    assert(ev(*cc, d) == TVal::Struct(sfs));
    let g1 = get_field(fs, 1, Ghost(d1));
    let g8 = get_field(fs, 8, Ghost(d1));
    let g9 = get_field(fs, 9, Ghost(d1));
    if g1.is_some() || g8.is_some() || g9.is_some() {
        return None;
    }
    let fo = match get_field(fs, 2, Ghost(d1)) {
        Some(EVal::I64(x)) => *x,
        _ => {
            return None;
        },
    };
    let m = match get_field(fs, 3, Ghost(d1)) {
        Some(EVal::Struct(m)) => m,
        _ => {
            return None;
        },
    };
    let ghost d2 = (d - 2) as nat;
    let ghost sm = efs(m@, d2);
    let r5 = get_field(m, 5, Ghost(d2));
    let r6 = get_field(m, 6, Ghost(d2));
    let r7 = get_field(m, 7, Ghost(d2));
    let r9 = get_field(m, 9, Ghost(d2));
    let r11 = get_field(m, 11, Ghost(d2));
    let r14 = get_field(m, 14, Ghost(d2));
    let r15 = get_field(m, 15, Ghost(d2));
    let c4 = get_field(fs, 4, Ghost(d1));
    let c5 = get_field(fs, 5, Ghost(d1));
    let c6 = get_field(fs, 6, Ghost(d1));
    let c7 = get_field(fs, 7, Ghost(d1));
    proof {
        lemma_get_field(sm, r5, d2, 5);
        lemma_get_field(sm, r6, d2, 6);
        lemma_get_field(sm, r7, d2, 7);
        lemma_get_field(sm, r9, d2, 9);
        lemma_get_field(sm, r11, d2, 11);
        lemma_get_field(sm, r14, d2, 14);
        lemma_get_field(sm, r15, d2, 15);
        lemma_get_field(sfs, c4, d1, 4);
        lemma_get_field(sfs, c5, d1, 5);
        lemma_get_field(sfs, c6, d1, 6);
        lemma_get_field(sfs, c7, d1, 7);
    }
    let nv = x_nn64(r5, Ghost(d2));
    let unc = x_nn64(r6, Ghost(d2));
    let tcs = x_nn64(r7, Ghost(d2));
    let dpo = x_nn64(r9, Ghost(d2));
    let dict = x_opt64(r11, Ghost(d2));
    let ci = x_pair(x_opt64(c6, Ghost(d1)), x_opt32(c7, Ghost(d1)));
    let oi = x_pair(x_opt64(c4, Ghost(d1)), x_opt32(c5, Ghost(d1)));
    let bo = x_opt64(r14, Ghost(d2));
    let bl = x_opt32(r15, Ghost(d2));
    let bloom = match (bo, bl) {
        (Some(None), Some(None)) => None,
        (Some(Some(a)), Some(b)) => Some((a, b)),
        _ => {
            return None;
        },
    };
    let (nv, unc, tcs, dpo, dict, ci, oi) = match (nv, unc, tcs, dpo, dict, ci, oi) {
        (Some(a), Some(b), Some(c), Some(e), Some(f), Some(g), Some(h)) => (a, b, c, e, f, g, h),
        _ => {
            return None;
        },
    };
    let start = match dict {
        Some(x) => x,
        None => dpo,
    };
    let copy = tcs > 0 && fo >= 0 && fo as u64 == start + tcs;
    let e = EDesc {
        start,
        size: tcs,
        num_values: nv,
        uncompressed: unc,
        dpo,
        dict: dict.is_some(),
        copy,
        fo,
        col_index: ci,
        off_index: oi,
        bloom,
    };
    assert(desc_of(ev(*cc, d)) == Some(dview(e)));
    Some(e)
}

pub open spec fn take_ok(rgs: Seq<TVal>, n: int) -> bool {
    rgs_ok(rgs.take(n))
}

/// The number of rows of a row group, if it passes [`rg_ok`].
#[verifier::loop_isolation(false)]
fn x_rg_rows(rg: &EVal, Ghost(d): Ghost<nat>) -> (r: Option<u64>)
    requires
        d >= 4,
    ensures
        match r {
            Some(n) => rg_ok(ev(*rg, d)) && rg_rows(ev(*rg, d)) == n as int && n
                < 0x8000_0000_0000_0000,
            None => !rg_ok(ev(*rg, d)),
        },
{
    let rfs = match rg {
        EVal::Struct(fs) => fs,
        _ => {
            return None;
        },
    };
    let ghost d1 = (d - 1) as nat;
    let ghost srfs = efs(rfs@, d1);
    let g3 = get_field(rfs, 3, Ghost(d1));
    let g1 = get_field(rfs, 1, Ghost(d1));
    proof {
        lemma_get_field(srfs, g3, d1, 3);
        lemma_get_field(srfs, g1, d1, 1);
    }
    let rows = match x_nn64(g3, Ghost(d1)) {
        Some(n) => n,
        None => {
            return None;
        },
    };
    let ccs = match x_list(g1, Ghost(d1)) {
        Some(xs) => xs,
        None => {
            return None;
        },
    };
    let ghost d2 = (d - 2) as nat;
    let ghost sccs = evs(ccs@, d2);
    assert(rg_ccs(ev(*rg, d)) == sccs);
    let mut c: usize = 0;
    assert(sccs.take(0) =~= Seq::<TVal>::empty());
    while c < ccs.len()
        invariant
            d >= 4,
            d2 == (d - 2) as nat,
            sccs == evs(ccs@, d2),
            c <= ccs@.len(),
            ccs_ok(sccs.take(c as int)),
        decreases ccs@.len() - c,
    {
        let e = x_desc_of(&ccs[c], Ghost(d2));
        assert(sccs.take(c + 1).drop_last() =~= sccs.take(c as int));
        assert(sccs.take(c + 1).last() == ev(ccs@[c as int], d2));
        if e.is_none() {
            proof {
                lemma_ccs_ok_prefix(sccs, c + 1, sccs.len() as int);
                assert(sccs.take(sccs.len() as int) =~= sccs);
            }
            return None;
        }
        c += 1;
    }
    assert(sccs.take(c as int) =~= sccs);
    Some(rows)
}

/// A prefix of a failing list of column chunks fails.
proof fn lemma_ccs_ok_prefix(ccs: Seq<TVal>, a: int, b: int)
    requires
        0 <= a <= b <= ccs.len(),
        !ccs_ok(ccs.take(a)),
    ensures
        !ccs_ok(ccs.take(b)),
    decreases b - a,
{
    if a < b {
        lemma_ccs_ok_prefix(ccs, a, b - 1);
        assert(ccs.take(b).drop_last() =~= ccs.take(b - 1));
    }
}

proof fn lemma_rgs_ok_prefix(rgs: Seq<TVal>, a: int, b: int)
    requires
        0 <= a <= b <= rgs.len(),
        !rgs_ok(rgs.take(a)),
    ensures
        !rgs_ok(rgs.take(b)),
    decreases b - a,
{
    if a < b {
        lemma_rgs_ok_prefix(rgs, a, b - 1);
        assert(rgs.take(b).drop_last() =~= rgs.take(b - 1));
    }
}

#[verifier::loop_isolation(false)]
fn x_meta_ok(v: &EVal) -> (b: bool)
    ensures
        b == meta_ok(ev(*v, maxd())),
{
    let fs = match v {
        EVal::Struct(fs) => fs,
        _ => {
            return false;
        },
    };
    let ghost d1 = (maxd() - 1) as nat;
    let ghost sfs = efs(fs@, d1);
    let g8 = get_field(fs, 8, Ghost(d1));
    let g4 = get_field(fs, 4, Ghost(d1));
    let g3 = get_field(fs, 3, Ghost(d1));
    proof {
        lemma_get_field(sfs, g8, d1, 8);
        lemma_get_field(sfs, g4, d1, 4);
        lemma_get_field(sfs, g3, d1, 3);
    }
    if g8.is_some() {
        return false;
    }
    let rgs = match x_list(g4, Ghost(d1)) {
        Some(xs) => xs,
        None => {
            return false;
        },
    };
    let ghost d2 = (maxd() - 2) as nat;
    let ghost srgs = evs(rgs@, d2);
    assert(meta_rgs(ev(*v, maxd())) == srgs);
    let mut total: u128 = 0;
    let mut r: usize = 0;
    assert(srgs.take(0) =~= Seq::<TVal>::empty());
    while r < rgs.len()
        invariant
            d2 == (maxd() - 2) as nat,
            srgs == evs(rgs@, d2),
            r <= rgs@.len(),
            rgs_ok(srgs.take(r as int)),
            total as int == sum_rows(srgs.take(r as int)),
            total <= r * 0x7fff_ffff_ffff_ffff,
        decreases rgs@.len() - r,
    {
        let n = x_rg_rows(&rgs[r], Ghost(d2));
        assert(srgs.take(r + 1).drop_last() =~= srgs.take(r as int));
        assert(srgs.take(r + 1).last() == ev(rgs@[r as int], d2));
        match n {
            Some(n) => {
                total = total + n as u128;
            },
            None => {
                proof {
                    lemma_rgs_ok_prefix(srgs, r + 1, srgs.len() as int);
                    assert(srgs.take(srgs.len() as int) =~= srgs);
                }
                return false;
            },
        }
        r += 1;
    }
    assert(srgs.take(r as int) =~= srgs);
    match x_nn64(g3, Ghost(d1)) {
        Some(n) => n as u128 == total,
        None => false,
    }
}

// ============================================================
// Executable jumps
// ============================================================

pub open spec fn jview(js: Seq<EJump<PKind>>) -> Seq<SJump<PKind>> {
    js.map_values(|j: EJump<PKind>| j@)
}

fn push_desc_jumps(js: &mut Vec<EJump<PKind>>, e: &EDesc, rg: u64, col: u64, len: usize)
    requires
        desc_bounds(*e),
    ensures
        jview(final(js)@) == jview(old(js)@) + desc_jumps(dview(*e), rg as int, col as int, len as int),
{
    let ghost j0 = jview(js@);
    let ghost d = dview(*e);
    let ghost pa = seq![
        SJump {
            kind: PKind::Chunk {
                start: d.start as u64,
                dict: d.dict,
                num_values: d.num_values as u64,
                uncompressed: d.uncompressed as u64,
                dpo: d.dpo as u64,
            },
            l: d.start,
            r: d.start + d.size,
            exact: true,
        },
    ];
    let ghost pb = if d.copy {
        seq![
            SJump {
                kind: PKind::InlineCopy { rg: rg as int as u64, col: col as int as u64 },
                l: d.fo,
                r: len as int,
                exact: false,
            },
        ]
    } else {
        seq![]
    };
    let ghost pc = match d.col_index {
        Some((o, l)) => seq![SJump { kind: PKind::ColumnIndex, l: o, r: o + l, exact: true }],
        None => seq![],
    };
    let ghost pd = match d.off_index {
        Some((o, l)) => seq![SJump { kind: PKind::OffsetIndex, l: o, r: o + l, exact: true }],
        None => seq![],
    };
    let ghost pe = match d.bloom {
        Some((o, Some(l))) => seq![SJump { kind: PKind::Bloom, l: o, r: o + l, exact: true }],
        Some((o, None)) => seq![SJump { kind: PKind::Bloom, l: o, r: len as int, exact: false }],
        None => seq![],
    };
    assert(desc_jumps(d, rg as int, col as int, len as int) == pa + pb + pc + pd + pe);
    js.push(
        EJump {
            kind: PKind::Chunk {
                start: e.start,
                dict: e.dict,
                num_values: e.num_values,
                uncompressed: e.uncompressed,
                dpo: e.dpo,
            },
            l: e.start as usize,
            r: (e.start + e.size) as usize,
            exact: true,
        },
    );
    assert(jview(js@) =~= j0 + pa);
    if e.copy {
        js.push(
            EJump { kind: PKind::InlineCopy { rg, col }, l: e.fo as usize, r: len, exact: false },
        );
    }
    assert(jview(js@) =~= j0 + pa + pb);
    if let Some((o, l)) = e.col_index {
        js.push(EJump { kind: PKind::ColumnIndex, l: o as usize, r: (o + l) as usize, exact: true });
    }
    assert(jview(js@) =~= j0 + pa + pb + pc);
    if let Some((o, l)) = e.off_index {
        js.push(EJump { kind: PKind::OffsetIndex, l: o as usize, r: (o + l) as usize, exact: true });
    }
    assert(jview(js@) =~= j0 + pa + pb + pc + pd);
    match e.bloom {
        Some((o, Some(l))) => {
            js.push(EJump { kind: PKind::Bloom, l: o as usize, r: (o + l) as usize, exact: true });
        },
        Some((o, None)) => {
            js.push(EJump { kind: PKind::Bloom, l: o as usize, r: len, exact: false });
        },
        None => {},
    }
    assert(jview(js@) =~= j0 + pa + pb + pc + pd + pe);
    assert(j0 + pa + pb + pc + pd + pe =~= j0 + (pa + pb + pc + pd + pe));
}

fn x_meta_jumps(v: &EVal, len: usize) -> (js: Vec<EJump<PKind>>)
    ensures
        jview(js@) == meta_jumps(ev(*v, maxd()), len as int),
{
    let mut js: Vec<EJump<PKind>> = Vec::new();
    js.push(EJump { kind: PKind::Head, l: 0, r: 4, exact: true });
    let ghost d1 = (maxd() - 1) as nat;
    let ghost d2 = (maxd() - 2) as nat;
    let ghost d3 = (maxd() - 3) as nat;
    let ghost d4 = (maxd() - 4) as nat;
    let empty: Vec<EVal> = Vec::new();
    let rgs: &Vec<EVal> = match v {
        EVal::Struct(fs) => {
            let g4 = get_field(fs, 4, Ghost(d1));
            proof {
                lemma_get_field(efs(fs@, d1), g4, d1, 4);
            }
            match x_list(g4, Ghost(d1)) {
                Some(xs) => xs,
                None => &empty,
            }
        },
        _ => &empty,
    };
    let ghost srgs = evs(rgs@, d2);
    assert(meta_rgs(ev(*v, maxd())) =~= srgs);
    assert(jview(js@) =~= seq![head_jump()] + rgs_jumps(srgs.take(0), len as int));
    let mut r: usize = 0;
    while r < rgs.len()
        invariant
            d2 == (maxd() - 2) as nat,
            d3 == (maxd() - 3) as nat,
            d4 == (maxd() - 4) as nat,
            srgs == evs(rgs@, d2),
            r <= rgs@.len(),
            jview(js@) == seq![head_jump()] + rgs_jumps(srgs.take(r as int), len as int),
        decreases rgs@.len() - r,
    {
        let rg = &rgs[r];
        let empty2: Vec<EVal> = Vec::new();
        let ccs: &Vec<EVal> = match rg {
            EVal::Struct(rfs) => {
                let g1 = get_field(rfs, 1, Ghost(d3));
                proof {
                    lemma_get_field(efs(rfs@, d3), g1, d3, 1);
                }
                match x_list(g1, Ghost(d3)) {
                    Some(xs) => xs,
                    None => &empty2,
                }
            },
            _ => &empty2,
        };
        let ghost sccs = evs(ccs@, d4);
        assert(rg_ccs(ev(*rg, d2)) =~= sccs);
        let ghost before = jview(js@);
        assert(before =~= seq![head_jump()] + rgs_jumps(srgs.take(r as int), len as int) + ccs_jumps(
            sccs.take(0),
            r as int,
            len as int,
        ));
        let mut c: usize = 0;
        while c < ccs.len()
            invariant
                d4 == (maxd() - 4) as nat,
                sccs == evs(ccs@, d4),
                c <= ccs@.len(),
                r < rgs@.len(),
                jview(js@) == seq![head_jump()] + rgs_jumps(srgs.take(r as int), len as int)
                    + ccs_jumps(sccs.take(c as int), r as int, len as int),
            decreases ccs@.len() - c,
        {
            let ghost j0 = jview(js@);
            assert(sccs.take(c + 1).drop_last() =~= sccs.take(c as int));
            assert(sccs.take(c + 1).last() == ev(ccs@[c as int], d4));
            if let Some(e) = x_desc_of(&ccs[c], Ghost(d4)) {
                push_desc_jumps(&mut js, &e, r as u64, c as u64, len);
            }
            assert(jview(js@) =~= seq![head_jump()] + rgs_jumps(srgs.take(r as int), len as int)
                + ccs_jumps(sccs.take(c + 1), r as int, len as int));
            c += 1;
        }
        assert(sccs.take(c as int) =~= sccs);
        assert(srgs.take(r + 1).drop_last() =~= srgs.take(r as int));
        assert(srgs.take(r + 1).last() == ev(rgs@[r as int], d2));
        assert(jview(js@) =~= seq![head_jump()] + rgs_jumps(srgs.take(r + 1), len as int));
        r += 1;
    }
    assert(srgs.take(r as int) =~= srgs);
    js
}

// ============================================================
// Executable inline copy check
// ============================================================

fn x_copy_ok(v: &EVal, rg: u64, col: u64, c: &EVal) -> (b: bool)
    requires
        stable(*v, maxd()),
        stable(*c, maxd()),
    ensures
        b == copy_ok(ev(*v, maxd()), rg as int, col as int, ev(*c, maxd())),
{
    let ghost d1 = (maxd() - 1) as nat;
    let ghost d2 = (maxd() - 2) as nat;
    let ghost d3 = (maxd() - 3) as nat;
    let ghost d4 = (maxd() - 4) as nat;
    let ghost d5 = (maxd() - 5) as nat;
    let fs = match v {
        EVal::Struct(fs) => fs,
        _ => {
            return false;
        },
    };
    let g4 = get_field(fs, 4, Ghost(d1));
    proof {
        lemma_get_field(efs(fs@, d1), g4, d1, 4);
    }
    let rgs = match x_list(g4, Ghost(d1)) {
        Some(xs) => xs,
        None => {
            return false;
        },
    };
    proof {
        let k = choose|k: int| 0 <= k < fs@.len() && *g4->0 == fs@[k].val;
        lemma_stable_field(*fs, d1, k);
    }
    if rg as usize >= rgs.len() {
        return false;
    }
    let rgv = &rgs[rg as usize];
    let et4 = match g4 {
        Some(EVal::List(et, _)) => *et,
        _ => {
            return false;
        },
    };
    proof {
        lemma_stable_elem(et4, *rgs, d2, rg as int);
    }
    let rfs = match rgv {
        EVal::Struct(rfs) => rfs,
        _ => {
            assert(rg_ccs(ev(*rgv, d2)) =~= Seq::<TVal>::empty());
            return false;
        },
    };
    let g1 = get_field(rfs, 1, Ghost(d3));
    proof {
        lemma_get_field(efs(rfs@, d3), g1, d3, 1);
    }
    let ccs = match x_list(g1, Ghost(d3)) {
        Some(xs) => xs,
        None => {
            return false;
        },
    };
    proof {
        let k = choose|k: int| 0 <= k < rfs@.len() && *g1->0 == rfs@[k].val;
        lemma_stable_field(*rfs, d3, k);
    }
    if col as usize >= ccs.len() {
        return false;
    }
    let cc = &ccs[col as usize];
    let et1 = match g1 {
        Some(EVal::List(et, _)) => *et,
        _ => {
            return false;
        },
    };
    proof {
        lemma_stable_elem(et1, *ccs, d4, col as int);
    }
    let cfs = match cc {
        EVal::Struct(cfs) => cfs,
        _ => {
            assert(fields_of(ev(*cc, d4)) =~= Seq::<TField>::empty());
            return false;
        },
    };
    let g2 = get_field(cfs, 2, Ghost(d5));
    let g3 = get_field(cfs, 3, Ghost(d5));
    proof {
        lemma_get_field(efs(cfs@, d5), g2, d5, 2);
        lemma_get_field(efs(cfs@, d5), g3, d5, 3);
    }
    let (fo, md) = match (g2, g3) {
        (Some(a), Some(b)) => (a, b),
        _ => {
            return false;
        },
    };
    proof {
        let k2 = choose|k: int| 0 <= k < cfs@.len() && *fo == cfs@[k].val;
        let k3 = choose|k: int| 0 <= k < cfs@.len() && *md == cfs@[k].val;
        lemma_stable_field(*cfs, d5, k2);
        lemma_stable_field(*cfs, d5, k3);
        assert(ev(*fo, d1) == ev(*fo, d5));
        assert(ev(*md, d1) == ev(*md, d5));
        assert(ev(*md, maxd()) == ev(*md, d5));
    }
    let ghost expected = TVal::Struct(
        seq![TField { id: 2, val: ev(*fo, d5) }, TField { id: 3, val: ev(*md, d5) }],
    );
    let as_chunk = match c {
        EVal::Struct(xf) => {
            if xf.len() == 2 && xf[0].id == 2 && xf[1].id == 3 {
                let a = tval_eq(&xf[0].val, fo, (MAX_DEPTH - 1));
                let b = tval_eq(&xf[1].val, md, (MAX_DEPTH - 1));
                proof {
                    if a && b {
                        assert(efs(xf@, d1) =~= seq![
                            TField { id: 2, val: ev(*fo, d5) },
                            TField { id: 3, val: ev(*md, d5) },
                        ]);
                    } else if !a {
                        assert(efs(xf@, d1)[0].val != ev(*fo, d5));
                    } else {
                        assert(efs(xf@, d1)[1].val != ev(*md, d5));
                    }
                }
                a && b
            } else {
                proof {
                    if xf.len() == 2 {
                        assert(efs(xf@, d1)[0].id == xf@[0].id as int);
                        assert(efs(xf@, d1)[1].id == xf@[1].id as int);
                    }
                    assert(efs(xf@, d1).len() == xf@.len());
                }
                false
            }
        },
        _ => false,
    };
    assert(as_chunk == (ev(*c, maxd()) == expected));
    as_chunk || tval_eq(c, md, MAX_DEPTH)
}

// ============================================================
// Executable column chunks
// ============================================================

pub open spec fn opt_prepend_seq<T>(acc: Seq<T>, r: Option<Seq<T>>) -> Option<Seq<T>> {
    match r {
        Some(xs) => Some(acc + xs),
        None => None,
    }
}

fn x_pages<'a>(s: &'a [u8]) -> (r: Option<Vec<EPage<'a>>>)
    ensures
        match r {
            Some(ps) => spec_pages(s@, 0) == Some(epages(ps@)) && forall|k: int|
                0 <= k < ps@.len() ==> stable(#[trigger] ps@[k].header, maxd()),
            None => spec_pages(s@, 0) is None,
        },
{
    let mut ps: Vec<EPage<'a>> = Vec::new();
    let mut i: usize = 0;
    proof {
        if let Some(xs) = spec_pages(s@, 0) {
            assert(Seq::<Page>::empty() + xs =~= xs);
        }
        assert(epages(ps@) =~= Seq::<Page>::empty());
    }
    while i < s.len()
        invariant
            i <= s@.len(),
            spec_pages(s@, 0) == opt_prepend_seq(epages(ps@), spec_pages(s@, i as int)),
            forall|k: int| 0 <= k < ps@.len() ==> stable(#[trigger] ps@[k].header, maxd()),
        decreases s@.len() - i,
    {
        let (j, h) = match parse_value(s, i, 12, MAX_DEPTH) {
            Some(x) => x,
            None => {
                return None;
            },
        };
        let c = match &h {
            EVal::Struct(fs) => {
                let g = get_field(fs, 3, Ghost((maxd() - 1) as nat));
                proof {
                    lemma_get_field(efs(fs@, (maxd() - 1) as nat), g, (maxd() - 1) as nat, 3);
                }
                x_nn32(g, Ghost((maxd() - 1) as nat))
            },
            _ => None,
        };
        assert(page_csize(ev(h, maxd())) == match c {
            Some(x) => Some(x as int),
            None => None::<int>,
        });
        let c = match c {
            Some(c) => c as usize,
            None => {
                return None;
            },
        };
        if j <= i || c > s.len() - j {
            return None;
        }
        let p = EPage { header: h, header_len: j - i, body: slice_subrange(s, j, j + c) };
        let ghost old_ps = ps@;
        proof {
            if let Some(rest) = spec_pages(s@, (j + c) as int) {
                assert(epages(old_ps) + (seq![
                    Page { header: ev(p.header, maxd()), header_len: (j - i) as int, body: s@.subrange(j as int, (j + c) as int) },
                ] + rest) =~= epages(old_ps.push(p)) + rest);
            }
        }
        ps.push(p);
        i = j + c;
    }
    proof {
        assert(epages(ps@) + Seq::<Page>::empty() =~= epages(ps@));
    }
    Some(ps)
}

fn x_page_type(h: &EVal) -> (r: Option<i32>)
    ensures
        match r {
            Some(t) => get(fields_of(ev(*h, maxd())), 1) == Some(TVal::I32(t as int)),
            None => !(get(fields_of(ev(*h, maxd())), 1) matches Some(TVal::I32(_))),
        },
{
    match h {
        EVal::Struct(fs) => {
            let g = get_field(fs, 1, Ghost((maxd() - 1) as nat));
            match g {
                Some(EVal::I32(t)) => Some(*t),
                _ => None,
            }
        },
        _ => {
            assert(fields_of(ev(*h, maxd())) =~= Seq::<TField>::empty());
            None
        },
    }
}

/// `num_values` in the struct at field `id` of a page header.
fn x_sub_nvals(h: &EVal, id: i16) -> (r: Option<u64>)
    ensures
        match r {
            Some(x) => (match get(fields_of(ev(*h, maxd())), id as int) {
                Some(TVal::Struct(g)) => nn32(get(g, 1)),
                _ => None,
            }) == Some(x as int) && x < 0x8000_0000,
            None => (match get(fields_of(ev(*h, maxd())), id as int) {
                Some(TVal::Struct(g)) => nn32(get(g, 1)),
                _ => None,
            }) is None,
        },
{
    let ghost d1 = (maxd() - 1) as nat;
    let ghost d2 = (maxd() - 2) as nat;
    match h {
        EVal::Struct(fs) => {
            let g = get_field(fs, id, Ghost(d1));
            match g {
                Some(EVal::Struct(gs)) => {
                    let n = get_field(gs, 1, Ghost(d2));
                    proof {
                        lemma_get_field(efs(gs@, d2), n, d2, 1);
                    }
                    x_nn32(n, Ghost(d2))
                },
                _ => None,
            }
        },
        _ => {
            assert(fields_of(ev(*h, maxd())) =~= Seq::<TField>::empty());
            None
        },
    }
}

fn x_page_usize(h: &EVal) -> (r: Option<u64>)
    ensures
        match r {
            Some(x) => nn32(get(fields_of(ev(*h, maxd())), 2)) == Some(x as int) && x < 0x8000_0000,
            None => nn32(get(fields_of(ev(*h, maxd())), 2)) is None,
        },
{
    match h {
        EVal::Struct(fs) => {
            let g = get_field(fs, 2, Ghost((maxd() - 1) as nat));
            proof {
                lemma_get_field(efs(fs@, (maxd() - 1) as nat), g, (maxd() - 1) as nat, 2);
            }
            x_nn32(g, Ghost((maxd() - 1) as nat))
        },
        _ => {
            assert(fields_of(ev(*h, maxd())) =~= Seq::<TField>::empty());
            None
        },
    }
}

fn x_page_nvals(p: &EPage) -> (r: Option<u64>)
    ensures
        match r {
            Some(x) => page_nvals(epages(seq![*p])[0]) == Some(x as int) && x < 0x8000_0000,
            None => page_nvals(epages(seq![*p])[0]) is None,
        },
{
    let ghost sp = epages(seq![*p])[0];
    assert(sp.header == ev(p.header, maxd()));
    match x_page_type(&p.header) {
        Some(0) => x_sub_nvals(&p.header, 5),
        Some(3) => x_sub_nvals(&p.header, 8),
        Some(2) => Some(0),
        _ => None,
    }
}

proof fn lemma_epages_take(ps: Seq<EPage>, k: int)
    requires
        0 <= k < ps.len(),
    ensures
        epages(ps).take(k + 1).drop_last() == epages(ps).take(k),
        epages(ps).take(k + 1).last() == epages(seq![ps[k]])[0],
{
    assert(epages(ps).take(k + 1).drop_last() =~= epages(ps).take(k));
}

proof fn lemma_sums_mono(ps: Seq<Page>, a: int, b: int)
    requires
        0 <= a <= b <= ps.len(),
        forall|k: int| 0 <= k < ps.len() ==> ps[k].header_len >= 0,
    ensures
        sum_nvals(ps.take(a)) <= sum_nvals(ps.take(b)),
        sum_unc(ps.take(a)) <= sum_unc(ps.take(b)),
    decreases b - a,
{
    if a < b {
        lemma_sums_mono(ps, a, b - 1);
        assert(ps.take(b).drop_last() =~= ps.take(b - 1));
        assert(ps.take(b).last() == ps[b - 1]);
    }
}

fn x_chunk_ok(start: u64, dict: bool, nv: u64, unc: u64, dpo: u64, ps: &Vec<EPage>) -> (b: bool)
    ensures
        b == chunk_ok(start as int, dict, nv as int, unc as int, dpo as int, epages(ps@)),
{
    let ghost sps = epages(ps@);
    let n = ps.len();
    if dict && n == 0 {
        return false;
    }
    let mut sum_v: u128 = 0;
    let mut sum_u: u128 = 0;
    let mut k: usize = 0;
    assert(sps.take(0) =~= Seq::<Page>::empty());
    while k < n
        invariant
            n == ps@.len(),
            sps == epages(ps@),
            k <= n,
            forall|q: int| 0 <= q < k ==> #[trigger] page_ok(sps[q], q, dict),
            sum_v as int == sum_nvals(sps.take(k as int)),
            sum_u as int == sum_unc(sps.take(k as int)),
            sum_v <= nv,
            sum_u <= unc,
        decreases n - k,
    {
        let p = &ps[k];
        proof {
            lemma_epages_take(ps@, k as int);
            assert(sps[k as int] == epages(seq![*p])[0]);
        }
        let t = x_page_type(&p.header);
        let ok_type = if k == 0 && dict {
            match t {
                Some(2) => true,
                _ => false,
            }
        } else {
            match t {
                Some(0) | Some(3) => true,
                _ => false,
            }
        };
        let nvals = x_page_nvals(p);
        let us = x_page_usize(&p.header);
        assert(page_type(sps[k as int]) == match t {
            Some(x) => Some(x as int),
            None => None::<int>,
        });
        match (nvals, us) {
            (Some(a), Some(b)) => {
                if !ok_type {
                    assert(!page_ok(sps[k as int], k as int, dict));
                    return false;
                }
                sum_v = sum_v + a as u128;
                sum_u = sum_u + p.header_len as u128 + b as u128;
                if sum_v > nv as u128 || sum_u > unc as u128 {
                    proof {
                        lemma_sums_mono(sps, k + 1, n as int);
                        assert(sps.take(n as int) =~= sps);
                    }
                    return false;
                }
            },
            _ => {
                assert(!page_ok(sps[k as int], k as int, dict));
                return false;
            },
        }
        k += 1;
    }
    assert(sps.take(n as int) =~= sps);
    if sum_v != nv as u128 || sum_u != unc as u128 {
        return false;
    }
    if dict && n >= 2 {
        let l0 = ps[0].header_len as u128 + ps[0].body.len() as u128;
        assert(page_len(sps[0]) == l0 as int);
        dpo as u128 == start as u128 + l0
    } else if !dict && n >= 1 {
        dpo == start
    } else {
        dpo == 0 || dpo == start
    }
}

fn x_bloom<'a>(s: &'a [u8]) -> (r: Option<(usize, PNode<'a>)>)
    ensures
        opt_matches(r, spec_bloom(s@)),
        r matches Some((_, PNode::Bloom(h, _))) ==> stable(h, maxd()),
        r matches Some((_, v)) ==> v is Bloom,
{
    let (j, h) = match parse_value(s, 0, 12, MAX_DEPTH) {
        Some(x) => x,
        None => {
            return None;
        },
    };
    let nb = match &h {
        EVal::Struct(fs) => {
            let g = get_field(fs, 1, Ghost((maxd() - 1) as nat));
            proof {
                lemma_get_field(efs(fs@, (maxd() - 1) as nat), g, (maxd() - 1) as nat, 1);
            }
            x_nn32(g, Ghost((maxd() - 1) as nat))
        },
        _ => None,
    };
    assert(nn32(get(fields_of(ev(h, maxd())), 1)) == match nb {
        Some(x) => Some(x as int),
        None => None::<int>,
    });
    let nb = match nb {
        Some(x) => x as usize,
        None => {
            return None;
        },
    };
    if nb > s.len() - j {
        return None;
    }
    Some((j + nb, PNode::Bloom(h, slice_subrange(s, j, j + nb))))
}

// ============================================================
// The executable ranked format
// ============================================================

impl<'a> RankedFormatExec<'a> for Parquet {
    type EKind = PKind;

    type EVal = PNode<'a>;

    open spec fn val_inv(&self, v: PNode<'a>) -> bool {
        match v {
            PNode::Struct(t) => stable(t, maxd()),
            PNode::Pages(ps) => forall|k: int|
                0 <= k < ps@.len() ==> stable(#[trigger] ps@[k].header, maxd()),
            PNode::Bloom(h, _) => stable(h, maxd()),
            _ => true,
        }
    }

    fn rank_of(&self, k: PKind) -> (r: u64) {
        match k {
            PKind::Tail => 2,
            PKind::Meta => 1,
            _ => 0,
        }
    }

    fn parse_node(&self, k: PKind, s: &'a [u8]) -> (res: Option<(usize, PNode<'a>)>) {
        match k {
            PKind::Tail => match TailFmt.parse(&s) {
                Ok((n, v)) => Some((n, PNode::Tail(v))),
                Err(_) => None,
            },
            PKind::Head => match HeadFmt.parse(&s) {
                Ok((n, v)) => Some((n, PNode::Head(v))),
                Err(_) => None,
            },
            PKind::Chunk { .. } => match x_pages(s) {
                Some(ps) => Some((s.len(), PNode::Pages(ps))),
                None => None,
            },
            PKind::Bloom => x_bloom(s),
            _ => match parse_value(s, 0, 12, MAX_DEPTH) {
                Some((n, v)) => Some((n, PNode::Struct(v))),
                None => None,
            },
        }
    }

    fn node_ok(&self, k: PKind, v: &PNode<'a>) -> (b: bool) {
        match k {
            PKind::Meta => match v {
                PNode::Struct(t) => x_meta_ok(t),
                _ => false,
            },
            PKind::Chunk { start, dict, num_values, uncompressed, dpo } => match v {
                PNode::Pages(ps) => x_chunk_ok(start, dict, num_values, uncompressed, dpo, ps),
                _ => false,
            },
            _ => true,
        }
    }

    fn jumps(&self, k: PKind, v: &PNode<'a>, root_len: usize) -> (js: Vec<EJump<PKind>>) {
        let ghost sj = self.spec_jumps(k@, v.deep_view(), root_len as int);
        let js = match v {
            PNode::Tail(t) => {
                proof {
                    t.lemma_deep_view_fields();
                }
                let m = t.metadata_len as usize;
                assert(v.deep_view() == PVal::Tail(t.deep_view()));
                assert(t.deep_view().metadata_len == t.metadata_len);
                let mut js = Vec::new();
                if root_len >= 8 && m <= root_len - 8 {
                    js.push(EJump { kind: PKind::Meta, l: root_len - 8 - m, r: root_len - 8, exact: true });
                } else {
                    js.push(EJump { kind: PKind::Meta, l: 0, r: 0, exact: true });
                }
                assert(jview(js@) =~= sj);
                js
            },
            PNode::Struct(t) => match k {
                PKind::Meta => x_meta_jumps(t, root_len),
                _ => {
                    let js = Vec::new();
                    assert(jview(js@) =~= sj);
                    js
                },
            },
            _ => {
                let js = Vec::new();
                assert(jview(js@) =~= sj);
                js
            },
        };
        assert(jview(js@) == sj);
        assert forall|i: int| 0 <= i < js@.len() implies #[trigger] js@[i]@ == sj[i] by {
            assert(jview(js@)[i] == js@[i]@);
        }
        js
    }

    fn link_ok(
        &self,
        k: PKind,
        v: &PNode<'a>,
        i: usize,
        child_kind: PKind,
        child_val: &PNode<'a>,
    ) -> (b: bool) {
        match child_kind {
            PKind::InlineCopy { rg, col } => match (v, child_val) {
                (PNode::Struct(t), PNode::Struct(c)) => x_copy_ok(t, rg, col, c),
                _ => false,
            },
            _ => true,
        }
    }

    fn root(&self, root: &'a [u8]) -> (j: Option<EJump<PKind>>) {
        if root.len() < 8 {
            None
        } else {
            Some(EJump { kind: PKind::Tail, l: root.len() - 8, r: root.len(), exact: true })
        }
    }

    fn layout(&self, rs: &Vec<ERegion<PKind, PNode<'a>>>) -> (o: Vec<usize>) {
        sorted_layout(rs)
    }
}

/// Parses `b` under the strict Parquet profile; see [`spec_parse_file`].
pub fn parse_parquet<'a>(b: &'a [u8]) -> (res: Option<Vec<ERegion<PKind, PNode<'a>>>>)
    ensures
        match res {
            Some(rs) => spec_parse_file(Parquet, b@) == Some(
                SFile { len: b@.len() as int, regions: dv_regions(rs@) },
            ),
            None => spec_parse_file(Parquet, b@) is None,
        },
{
    parse_file(&Parquet, b)
}

} // verus!
