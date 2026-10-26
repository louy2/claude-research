//! Root-relative intervals and ranked jumps.
//!
//! A format is described by a set of node kinds. Each kind has
//!
//! - a rank (a natural number),
//! - a local parser for the bytes of its region (normally a Vest format),
//! - a list of outgoing jumps computed from its parsed value; a jump names a
//!   kind and an interval `[l, r)` of the *whole* input,
//! - a link check between a node and each node it jumps to.
//!
//! [`spec_resolve`] parses a node at its interval and then follows its jumps.
//! A jump is followed only if the target kind has a strictly lower rank, so
//! the recursion is well founded on `(rank, ...)`. Within a region, ordinary
//! Vest combinators do the work, so variable-length tables (which IPG cannot
//! index) are a plain `Vec<entry>` there.
//!
//! [`spec_parse_file`] locates the root node, resolves it, and accepts the
//! file only if the regions tile it: in the order given by the format's
//! layout, every region starts where the previous one ended, the first starts
//! at 0, the last ends at the end of the file, and each region appears once.
//!
//! Proven here, for every format:
//!
//! - [`theorem_non_malleable`]: two accepted inputs with the same parse are
//!   equal;
//! - [`theorem_regions_disjoint`]: the regions of an accepted input are in
//!   bounds, pairwise disjoint and cover it;
//! - [`parse_file`] (executable) returns what [`spec_parse_file`] specifies.
use vest_lib::core::exec::parser::Parser;
use vest_lib::core::spec::SpecParser;
use vstd::prelude::*;
use vstd::slice::slice_subrange;

verus! {

// ============================================================
// At and backward scan
// ============================================================

/// `At(l, r, c)`: `c` parses a prefix of `root[l..r)`; the footprint is
/// `[l, l + n)` where `n` is the number of bytes consumed.
pub open spec fn spec_at<C: SpecParser>(c: C, root: Seq<u8>, l: int, r: int) -> Option<(int, C::PVal)> {
    if 0 <= l <= r <= root.len() {
        c.spec_parse(root.subrange(l, r))
    } else {
        None
    }
}

/// Relates an executable `Option` result to a spec result.
pub open spec fn opt_matches<O: DeepView>(r: Option<(usize, O)>, s: Option<(int, O::V)>) -> bool {
    match r {
        Some((n, v)) => s == Some((n as int, v.deep_view())),
        None => s is None,
    }
}

pub fn at<'a, C: Parser<&'a [u8]>>(c: &C, root: &'a [u8], l: usize, r: usize) -> (res: Option<
    (usize, C::PT),
>)
    requires
        c.exec_inv(),
    ensures
        opt_matches(res, spec_at(*c, root@, l as int, r as int)),
{
    if l <= r && r <= root.len() {
        let s = slice_subrange(root, l, r);
        match c.parse(&s) {
            Ok((n, v)) => Some((n, v)),
            Err(_) => None,
        }
    } else {
        None
    }
}

/// The largest `p` in `[lo, hi]` such that `c` parses `root[p..)` and
/// consumes all of it: a record anchored at the end of the input, found by
/// scanning backwards (the ZIP end of central directory record, a PDF
/// trailer).
pub open spec fn spec_scan_back<C: SpecParser>(c: C, root: Seq<u8>, lo: int, hi: int) -> Option<
    (int, C::PVal),
>
    decreases hi - lo + 1,
{
    if hi < lo {
        None
    } else {
        let r = spec_at(c, root, hi, root.len() as int);
        if r is Some && (r->0).0 == root.len() - hi {
            Some((hi, (r->0).1))
        } else {
            spec_scan_back(c, root, lo, hi - 1)
        }
    }
}

pub fn scan_back<'a, C: Parser<&'a [u8]>>(c: &C, root: &'a [u8], lo: usize, hi: usize) -> (res:
    Option<(usize, C::PT)>)
    requires
        c.exec_inv(),
    ensures
        opt_matches(res, spec_scan_back(*c, root@, lo as int, hi as int)),
{
    let mut p = hi;
    loop
        invariant
            c.exec_inv(),
            spec_scan_back(*c, root@, lo as int, hi as int) == spec_scan_back(
                *c,
                root@,
                lo as int,
                p as int,
            ),
        decreases p,
    {
        if p < lo {
            return None;
        }
        if let Some((n, v)) = at(c, root, p, root.len()) {
            if n == root.len() - p {
                return Some((p, v));
            }
        }
        if p == 0 {
            assert(spec_scan_back(*c, root@, lo as int, -1) is None);
            return None;
        }
        p = p - 1;
    }
}

// ============================================================
// Ranked formats
// ============================================================

/// A jump to a node of kind `kind` at the interval `[l, r)` of the whole
/// input. With `exact`, the node must consume the whole interval; otherwise
/// it may stop earlier (a ZIP local file header knows its own length).
pub struct SJump<K> {
    pub kind: K,
    pub l: int,
    pub r: int,
    pub exact: bool,
}

/// A parsed region: a node of kind `kind` occupying `[start, start + len)`.
pub struct SRegion<K, V> {
    pub kind: K,
    pub start: int,
    pub len: int,
    pub val: V,
}

/// The result of parsing a whole input.
pub struct SFile<K, V> {
    pub len: int,
    pub regions: Seq<SRegion<K, V>>,
}

pub trait RankedFormat {
    type Kind;

    type Val;

    /// Jumps go from a kind to kinds of strictly lower rank.
    spec fn rank(&self, k: Self::Kind) -> nat;

    /// The local parser of a kind, run on the bytes of its interval.
    spec fn spec_parse_node(&self, k: Self::Kind, s: Seq<u8>) -> Option<(int, Self::Val)>;

    /// The serializer matching `spec_parse_node`.
    spec fn spec_serialize_node(&self, k: Self::Kind, v: Self::Val) -> Seq<u8>;

    /// Constraints on a single value that the local parser does not express.
    spec fn spec_node_ok(&self, k: Self::Kind, v: Self::Val) -> bool;

    /// The jumps out of a node, given the length of the whole input.
    spec fn spec_jumps(&self, k: Self::Kind, v: Self::Val, root_len: int) -> Seq<SJump<Self::Kind>>;

    /// Agreement between a node and the node its `i`-th jump reached.
    spec fn spec_link_ok(
        &self,
        k: Self::Kind,
        v: Self::Val,
        i: int,
        child_kind: Self::Kind,
        child_val: Self::Val,
    ) -> bool;

    /// Where the root node is.
    spec fn spec_root(&self, root: Seq<u8>) -> Option<SJump<Self::Kind>>;

    /// The order in which the regions must tile the input.
    spec fn spec_layout(&self, rs: Seq<SRegion<Self::Kind, Self::Val>>) -> Seq<int>;

    /// The local parser is safe and non-malleable: what it consumed is the
    /// serialization of what it returned (Vest's `PSRoundTrip`).
    proof fn lemma_parse_node_serialize(&self, k: Self::Kind, s: Seq<u8>)
        ensures
            self.spec_parse_node(k, s) matches Some((n, v)) ==> {
                &&& 0 <= n <= s.len()
                &&& self.spec_serialize_node(k, v) == s.take(n)
            },
    ;
}

pub open spec fn spec_node_at<F: RankedFormat>(
    f: F,
    root: Seq<u8>,
    k: F::Kind,
    l: int,
    r: int,
) -> Option<(int, F::Val)> {
    if 0 <= l <= r <= root.len() {
        f.spec_parse_node(k, root.subrange(l, r))
    } else {
        None
    }
}

/// Parses the node `j` points at and everything reachable from it. The
/// first region is the node itself, followed by the regions of its jumps in
/// order.
pub open spec fn spec_resolve<F: RankedFormat>(f: F, root: Seq<u8>, j: SJump<F::Kind>) -> Option<
    Seq<SRegion<F::Kind, F::Val>>,
>
    decreases f.rank(j.kind), 1nat, 0nat,
{
    match spec_node_at(f, root, j.kind, j.l, j.r) {
        None => None,
        Some((n, v)) => {
            if (j.exact && n != j.r - j.l) || !f.spec_node_ok(j.kind, v) {
                None
            } else {
                let js = f.spec_jumps(j.kind, v, root.len() as int);
                match spec_resolve_children(f, root, j.kind, v, js, js.len()) {
                    None => None,
                    Some(rs) => Some(
                        seq![SRegion { kind: j.kind, start: j.l, len: n, val: v }] + rs,
                    ),
                }
            }
        }
    }
}

/// The regions reached through the first `i` jumps `js` of node `(k, v)`.
/// A jump to a kind whose rank is not strictly lower than `k`'s fails.
pub open spec fn spec_resolve_children<F: RankedFormat>(
    f: F,
    root: Seq<u8>,
    k: F::Kind,
    v: F::Val,
    js: Seq<SJump<F::Kind>>,
    i: nat,
) -> Option<Seq<SRegion<F::Kind, F::Val>>>
    decreases f.rank(k), 0nat, i,
{
    if i == 0 {
        Some(seq![])
    } else if i > js.len() {
        None
    } else {
        match spec_resolve_children(f, root, k, v, js, (i - 1) as nat) {
            None => None,
            Some(prev) => {
                let c = js[i - 1];
                if f.rank(c.kind) < f.rank(k) {
                    match spec_resolve(f, root, c) {
                        None => None,
                        Some(rs) => {
                            if rs.len() > 0 && f.spec_link_ok(k, v, i - 1, rs[0].kind, rs[0].val) {
                                Some(prev + rs)
                            } else {
                                None
                            }
                        },
                    }
                } else {
                    None
                }
            },
        }
    }
}

/// End of the region at layout position `i`.
pub open spec fn layout_end<K, V>(rs: Seq<SRegion<K, V>>, order: Seq<int>, i: int) -> int {
    rs[order[i]].start + rs[order[i]].len
}

pub open spec fn in_order(order: Seq<int>, x: int) -> bool {
    exists|i: int| 0 <= i < order.len() && order[i] == x
}

/// `order` lists indices below `n`, without repetition, and every index
/// below `n` appears in it.
pub open spec fn is_permutation(n: int, order: Seq<int>) -> bool {
    &&& order.len() == n
    &&& forall|i: int| 0 <= i < order.len() ==> 0 <= #[trigger] order[i] < n
    &&& forall|i: int, j: int| 0 <= i < j < order.len() ==> order[i] != order[j]
    &&& forall|x: int| 0 <= x < n ==> #[trigger] in_order(order, x)
}

/// In layout order, each region starts where the previous one ended, the
/// first at 0 and the last ending at `total`.
pub open spec fn is_chain<K, V>(rs: Seq<SRegion<K, V>>, order: Seq<int>, total: int) -> bool {
    &&& order.len() == 0 ==> total == 0
    &&& order.len() > 0 ==> rs[order[0]].start == 0 && layout_end(rs, order, order.len() - 1)
        == total
    &&& forall|i: int|
        0 < i < order.len() ==> #[trigger] rs[order[i]].start == layout_end(rs, order, i - 1)
}

/// The regions, in layout order, tile `[0, total)`, and the layout is a
/// permutation of the regions.
pub open spec fn spec_tiled<K, V>(rs: Seq<SRegion<K, V>>, order: Seq<int>, total: int) -> bool {
    is_permutation(rs.len() as int, order) && is_chain(rs, order, total)
}

pub open spec fn spec_parse_file<F: RankedFormat>(f: F, root: Seq<u8>) -> Option<
    SFile<F::Kind, F::Val>,
> {
    match f.spec_root(root) {
        None => None,
        Some(j) => match spec_resolve(f, root, j) {
            None => None,
            Some(rs) => {
                if spec_tiled(rs, f.spec_layout(rs), root.len() as int) {
                    Some(SFile { len: root.len() as int, regions: rs })
                } else {
                    None
                }
            },
        },
    }
}

// ============================================================
// Proofs
// ============================================================

/// A region is in bounds and holds the serialization of its value.
pub open spec fn region_ok<F: RankedFormat>(
    f: F,
    root: Seq<u8>,
    r: SRegion<F::Kind, F::Val>,
) -> bool {
    &&& 0 <= r.start
    &&& 0 <= r.len
    &&& r.start + r.len <= root.len()
    &&& root.subrange(r.start, r.start + r.len) == f.spec_serialize_node(r.kind, r.val)
}

pub open spec fn regions_ok<F: RankedFormat>(
    f: F,
    root: Seq<u8>,
    rs: Seq<SRegion<F::Kind, F::Val>>,
) -> bool {
    forall|i: int| 0 <= i < rs.len() ==> #[trigger] region_ok(f, root, rs[i])
}

pub proof fn lemma_resolve_ok<F: RankedFormat>(f: F, root: Seq<u8>, j: SJump<F::Kind>)
    ensures
        spec_resolve(f, root, j) matches Some(rs) ==> regions_ok(f, root, rs) && rs.len() > 0
            && rs[0].start == j.l,
    decreases f.rank(j.kind), 1nat, 0nat,
{
    if let Some((n, v)) = spec_node_at(f, root, j.kind, j.l, j.r) {
        if !((j.exact && n != j.r - j.l) || !f.spec_node_ok(j.kind, v)) {
            let js = f.spec_jumps(j.kind, v, root.len() as int);
            lemma_resolve_children_ok(f, root, j.kind, v, js, js.len());
            if let Some(rs) = spec_resolve_children(f, root, j.kind, v, js, js.len()) {
                let s = root.subrange(j.l, j.r);
                f.lemma_parse_node_serialize(j.kind, s);
                assert(s.take(n) =~= root.subrange(j.l, j.l + n));
                let me = SRegion { kind: j.kind, start: j.l, len: n, val: v };
                assert(region_ok(f, root, me));
                let all = seq![me] + rs;
                assert forall|i: int| 0 <= i < all.len() implies #[trigger] region_ok(
                    f,
                    root,
                    all[i],
                ) by {
                    if i > 0 {
                        assert(all[i] == rs[i - 1]);
                    }
                }
            }
        }
    }
}

pub proof fn lemma_resolve_children_ok<F: RankedFormat>(
    f: F,
    root: Seq<u8>,
    k: F::Kind,
    v: F::Val,
    js: Seq<SJump<F::Kind>>,
    i: nat,
)
    ensures
        spec_resolve_children(f, root, k, v, js, i) matches Some(rs) ==> regions_ok(f, root, rs),
    decreases f.rank(k), 0nat, i,
{
    if i > 0 && i <= js.len() {
        lemma_resolve_children_ok(f, root, k, v, js, (i - 1) as nat);
        if let Some(prev) = spec_resolve_children(f, root, k, v, js, (i - 1) as nat) {
            let c = js[i - 1];
            if f.rank(c.kind) < f.rank(k) {
                lemma_resolve_ok(f, root, c);
                if let Some(rs) = spec_resolve(f, root, c) {
                    let all = prev + rs;
                    assert forall|q: int| 0 <= q < all.len() implies #[trigger] region_ok(
                        f,
                        root,
                        all[q],
                    ) by {
                        if q >= prev.len() {
                            assert(all[q] == rs[q - prev.len()]);
                        } else {
                            assert(all[q] == prev[q]);
                        }
                    }
                }
            }
        }
    }
}

/// Once a prefix of the jumps fails, every longer prefix fails.
pub proof fn lemma_children_none_mono<F: RankedFormat>(
    f: F,
    root: Seq<u8>,
    k: F::Kind,
    v: F::Val,
    js: Seq<SJump<F::Kind>>,
    i: nat,
    j: nat,
)
    requires
        i <= j,
        spec_resolve_children(f, root, k, v, js, i) is None,
    ensures
        spec_resolve_children(f, root, k, v, js, j) is None,
    decreases j - i,
{
    if i < j {
        lemma_children_none_mono(f, root, k, v, js, i, (j - 1) as nat);
    }
}

/// In a tiling, a region at a later layout position starts after every
/// region at an earlier position ends.
pub proof fn lemma_tiled_monotone<F: RankedFormat>(
    f: F,
    root: Seq<u8>,
    rs: Seq<SRegion<F::Kind, F::Val>>,
    order: Seq<int>,
    total: int,
    p: int,
    q: int,
)
    requires
        spec_tiled(rs, order, total),
        regions_ok(f, root, rs),
        0 <= p < q < order.len(),
    ensures
        layout_end(rs, order, p) <= rs[order[q]].start,
    decreases q - p,
{
    assert(rs[order[q]].start == layout_end(rs, order, q - 1));
    if p < q - 1 {
        lemma_tiled_monotone(f, root, rs, order, total, p, q - 1);
        assert(0 <= order[q - 1] < rs.len());
        assert(region_ok(f, root, rs[order[q - 1]]));
    }
}

/// In a tiling, every position of `[0, layout_end(m - 1))` lies in one of
/// the first `m` regions in layout order.
pub proof fn lemma_tiled_cover<K, V>(
    rs: Seq<SRegion<K, V>>,
    order: Seq<int>,
    total: int,
    m: int,
    x: int,
) -> (i: int)
    requires
        spec_tiled(rs, order, total),
        0 < m <= order.len(),
        0 <= x < layout_end(rs, order, m - 1),
    ensures
        0 <= i < m,
        rs[order[i]].start <= x < layout_end(rs, order, i),
    decreases m,
{
    if m == 1 || x >= rs[order[m - 1]].start {
        if m == 1 {
            assert(rs[order[0]].start == 0);
        }
        m - 1
    } else {
        assert(rs[order[m - 1]].start == layout_end(rs, order, m - 2));
        lemma_tiled_cover(rs, order, total, m - 1, x)
    }
}

pub open spec fn covers<K, V>(r: SRegion<K, V>, x: int) -> bool {
    r.start <= x < r.start + r.len
}

pub open spec fn covered<K, V>(rs: Seq<SRegion<K, V>>, x: int) -> bool {
    exists|a: int| 0 <= a < rs.len() && covers(#[trigger] rs[a], x)
}

/// The regions of an accepted input are in bounds, pairwise disjoint, and
/// cover the input.
pub proof fn theorem_regions_disjoint<F: RankedFormat>(f: F, root: Seq<u8>)
    requires
        spec_parse_file(f, root) is Some,
    ensures
        ({
            let rs = spec_parse_file(f, root)->0.regions;
            &&& regions_ok(f, root, rs)
            &&& forall|a: int, b: int|
                0 <= a < rs.len() && 0 <= b < rs.len() && a != b ==> rs[a].start + rs[a].len
                    <= rs[b].start || rs[b].start + rs[b].len <= rs[a].start
            &&& forall|x: int| 0 <= x < root.len() ==> #[trigger] covered(rs, x)
        }),
{
    let j = f.spec_root(root)->0;
    let rs = spec_resolve(f, root, j)->0;
    let order = f.spec_layout(rs);
    lemma_resolve_ok(f, root, j);
    assert forall|a: int, b: int|
        0 <= a < rs.len() && 0 <= b < rs.len() && a != b implies rs[a].start + rs[a].len
        <= rs[b].start || rs[b].start + rs[b].len <= rs[a].start by {
        assert(in_order(order, a));
        assert(in_order(order, b));
        let p = choose|p: int| 0 <= p < order.len() && order[p] == a;
        let q = choose|q: int| 0 <= q < order.len() && order[q] == b;
        assert(p != q);
        if p < q {
            lemma_tiled_monotone(f, root, rs, order, root.len() as int, p, q);
        } else {
            lemma_tiled_monotone(f, root, rs, order, root.len() as int, q, p);
        }
    }
    assert forall|x: int| 0 <= x < root.len() implies #[trigger] covered(rs, x) by {
        let i = lemma_tiled_cover(rs, order, root.len() as int, order.len() as int, x);
        assert(0 <= order[i] < rs.len());
        assert(covers(rs[order[i]], x));
    }
}

/// Two inputs whose regions both hold the serializations of the same values,
/// with the regions tiling, agree at every position.
pub proof fn lemma_same_byte<F: RankedFormat>(
    f: F,
    b1: Seq<u8>,
    b2: Seq<u8>,
    rs: Seq<SRegion<F::Kind, F::Val>>,
    order: Seq<int>,
    x: int,
)
    requires
        spec_tiled(rs, order, b1.len() as int),
        b1.len() == b2.len(),
        regions_ok(f, b1, rs),
        regions_ok(f, b2, rs),
        0 <= x < b1.len(),
    ensures
        b1[x] == b2[x],
{
    let i = lemma_tiled_cover(rs, order, b1.len() as int, order.len() as int, x);
    let a = order[i];
    assert(0 <= a < rs.len());
    let r = rs[a];
    assert(region_ok(f, b1, r));
    assert(region_ok(f, b2, r));
    assert(b1[x] == b1.subrange(r.start, r.start + r.len)[x - r.start]);
    assert(b2[x] == b2.subrange(r.start, r.start + r.len)[x - r.start]);
}

/// Non-malleability: two accepted inputs with the same regions and values
/// are the same bytes.
pub proof fn theorem_non_malleable<F: RankedFormat>(f: F, b1: Seq<u8>, b2: Seq<u8>)
    requires
        spec_parse_file(f, b1) is Some,
        spec_parse_file(f, b1) == spec_parse_file(f, b2),
    ensures
        b1 == b2,
{
    let j1 = f.spec_root(b1)->0;
    let j2 = f.spec_root(b2)->0;
    let rs = spec_resolve(f, b1, j1)->0;
    assert(spec_resolve(f, b2, j2) == Some(rs));
    let order = f.spec_layout(rs);
    lemma_resolve_ok(f, b1, j1);
    lemma_resolve_ok(f, b2, j2);
    assert(b1.len() == b2.len());
    assert forall|x: int| 0 <= x < b1.len() implies b1[x] == b2[x] by {
        lemma_same_byte(f, b1, b2, rs, order, x);
    }
    assert(b1 =~= b2);
}

// ============================================================
// Executable driver
// ============================================================

pub struct EJump<K> {
    pub kind: K,
    pub l: usize,
    pub r: usize,
    pub exact: bool,
}

impl<K: Copy> Clone for EJump<K> {
    fn clone(&self) -> (c: Self)
        ensures
            c == *self,
    {
        EJump { kind: self.kind, l: self.l, r: self.r, exact: self.exact }
    }
}

impl<K: Copy> Copy for EJump<K> {

}

impl<K: View> View for EJump<K> {
    type V = SJump<K::V>;

    open spec fn view(&self) -> SJump<K::V> {
        SJump { kind: self.kind@, l: self.l as int, r: self.r as int, exact: self.exact }
    }
}

pub struct ERegion<K, V> {
    pub kind: K,
    pub start: usize,
    pub len: usize,
    pub val: V,
}

impl<K: View, V: DeepView> DeepView for ERegion<K, V> {
    type V = SRegion<K::V, V::V>;

    open spec fn deep_view(&self) -> SRegion<K::V, V::V> {
        SRegion {
            kind: self.kind@,
            start: self.start as int,
            len: self.len as int,
            val: self.val.deep_view(),
        }
    }
}

pub open spec fn dv_regions<K: View, V: DeepView>(rs: Seq<ERegion<K, V>>) -> Seq<
    SRegion<K::V, V::V>,
> {
    rs.map_values(|r: ERegion<K, V>| r.deep_view())
}

pub open spec fn dv_order(o: Seq<usize>) -> Seq<int> {
    o.map_values(|x: usize| x as int)
}

pub trait RankedFormatExec<'a>: RankedFormat {
    type EKind: View<V = Self::Kind> + Copy;

    type EVal: DeepView<V = Self::Val>;

    fn rank_of(&self, k: Self::EKind) -> (r: u64)
        ensures
            r as nat == self.rank(k@),
    ;

    fn parse_node(&self, k: Self::EKind, s: &'a [u8]) -> (res: Option<(usize, Self::EVal)>)
        ensures
            opt_matches(res, self.spec_parse_node(k@, s@)),
    ;

    fn node_ok(&self, k: Self::EKind, v: &Self::EVal) -> (b: bool)
        ensures
            b == self.spec_node_ok(k@, v.deep_view()),
    ;

    fn jumps(&self, k: Self::EKind, v: &Self::EVal, root_len: usize) -> (js: Vec<EJump<Self::EKind>>)
        ensures
            js@.len() == self.spec_jumps(k@, v.deep_view(), root_len as int).len(),
            forall|i: int|
                0 <= i < js@.len() ==> #[trigger] js@[i]@ == self.spec_jumps(
                    k@,
                    v.deep_view(),
                    root_len as int,
                )[i],
    ;

    fn link_ok(
        &self,
        k: Self::EKind,
        v: &Self::EVal,
        i: usize,
        child_kind: Self::EKind,
        child_val: &Self::EVal,
    ) -> (b: bool)
        ensures
            b == self.spec_link_ok(k@, v.deep_view(), i as int, child_kind@, child_val.deep_view()),
    ;

    fn root(&self, root: &'a [u8]) -> (j: Option<EJump<Self::EKind>>)
        ensures
            match j {
                Some(j) => self.spec_root(root@) == Some(j@),
                None => self.spec_root(root@) is None,
            },
    ;

    fn layout(&self, rs: &Vec<ERegion<Self::EKind, Self::EVal>>) -> (o: Vec<usize>)
        ensures
            dv_order(o@) == self.spec_layout(dv_regions(rs@)),
    ;
}

#[verifier::loop_isolation(false)]
pub fn resolve<'a, F: RankedFormatExec<'a>>(f: &F, root: &'a [u8], j: EJump<F::EKind>) -> (res:
    Option<Vec<ERegion<F::EKind, F::EVal>>>)
    ensures
        match res {
            Some(rs) => spec_resolve(*f, root@, j@) == Some(dv_regions(rs@)),
            None => spec_resolve(*f, root@, j@) is None,
        },
    decreases f.rank(j.kind@),
{
    if !(j.l <= j.r && j.r <= root.len()) {
        return None;
    }
    let s = slice_subrange(root, j.l, j.r);
    let (n, v) = match f.parse_node(j.kind, s) {
        Some(x) => x,
        None => {
            return None;
        },
    };
    if (j.exact && n != j.r - j.l) || !f.node_ok(j.kind, &v) {
        return None;
    }
    assert(spec_node_at(*f, root@, j.kind@, j.l as int, j.r as int) == Some((n as int, v.deep_view())));
    let js = f.jumps(j.kind, &v, root.len());
    let ghost sjs = f.spec_jumps(j.kind@, v.deep_view(), root@.len() as int);
    let k_rank = f.rank_of(j.kind);
    let mut acc: Vec<ERegion<F::EKind, F::EVal>> = Vec::new();
    let mut i: usize = 0;
    while i < js.len()
        invariant
            0 <= i <= js@.len(),
            js@.len() == sjs.len(),
            forall|q: int| 0 <= q < js@.len() ==> #[trigger] js@[q]@ == sjs[q],
            k_rank as nat == f.rank(j.kind@),
            spec_resolve_children(*f, root@, j.kind@, v.deep_view(), sjs, i as nat) == Some(
                dv_regions(acc@),
            ),
        decreases js@.len() - i,
    {
        let c = js[i];
        assert(c@ == sjs[i as int]);
        let ghost spec_next = spec_resolve_children(
            *f,
            root@,
            j.kind@,
            v.deep_view(),
            sjs,
            (i + 1) as nat,
        );
        if f.rank_of(c.kind) >= k_rank {
            proof {
                assert(spec_next is None);
                lemma_children_none_mono(
                    *f,
                    root@,
                    j.kind@,
                    v.deep_view(),
                    sjs,
                    (i + 1) as nat,
                    sjs.len(),
                );
                assert(spec_resolve(*f, root@, j@) is None);
            }
            return None;
        }
        let mut child = match resolve(f, root, c) {
            Some(x) => x,
            None => {
                proof {
                    assert(spec_next is None);
                    lemma_children_none_mono(
                        *f,
                        root@,
                        j.kind@,
                        v.deep_view(),
                        sjs,
                        (i + 1) as nat,
                        sjs.len(),
                    );
                    assert(spec_resolve(*f, root@, j@) is None);
                }
                return None;
            },
        };
        let ok = child.len() > 0 && f.link_ok(j.kind, &v, i, child[0].kind, &child[0].val);
        if !ok {
            proof {
                assert(spec_next is None);
                lemma_children_none_mono(
                    *f,
                    root@,
                    j.kind@,
                    v.deep_view(),
                    sjs,
                    (i + 1) as nat,
                    sjs.len(),
                );
                assert(spec_resolve(*f, root@, j@) is None);
            }
            return None;
        }
        let ghost old_acc = acc@;
        let ghost child_v = child@;
        acc.append(&mut child);
        proof {
            assert(dv_regions(acc@) =~= dv_regions(old_acc) + dv_regions(child_v));
        }
        i += 1;
    }
    let me = ERegion { kind: j.kind, start: j.l, len: n, val: v };
    let mut out = Vec::new();
    out.push(me);
    let ghost acc_v = acc@;
    out.append(&mut acc);
    proof {
        assert(dv_regions(out@) =~= seq![me.deep_view()] + dv_regions(acc_v));
    }
    Some(out)
}

pub fn check_permutation(n: usize, order: &Vec<usize>) -> (b: bool)
    ensures
        b == is_permutation(n as int, dv_order(order@)),
{
    let ghost so = dv_order(order@);
    if order.len() != n {
        return false;
    }
    let mut seen: Vec<bool> = Vec::new();
    let mut x: usize = 0;
    while x < n
        invariant
            x <= n,
            seen@.len() == x,
            forall|y: int| 0 <= y < x ==> !(#[trigger] seen@[y]),
        decreases n - x,
    {
        seen.push(false);
        x += 1;
    }
    let mut i: usize = 0;
    while i < n
        invariant
            n == order@.len(),
            so == dv_order(order@),
            0 <= i <= n,
            seen@.len() == n,
            forall|p: int| 0 <= p < i ==> 0 <= #[trigger] so[p] < n,
            forall|p: int, q: int| 0 <= p < q < i ==> so[p] != so[q],
            forall|y: int|
                0 <= y < n ==> (#[trigger] seen@[y] <==> exists|p: int|
                    0 <= p < i && #[trigger] so[p] == y),
        decreases n - i,
    {
        let x = order[i];
        assert(so[i as int] == x as int);
        if x >= n {
            return false;
        }
        if seen[x] {
            proof {
                let p = choose|p: int| 0 <= p < i && so[p] == x as int;
                assert(so[p] == so[i as int]);
            }
            return false;
        }
        let ghost old_seen = seen@;
        seen.set(x, true);
        proof {
            assert forall|y: int| 0 <= y < n implies (#[trigger] seen@[y] <==> exists|p: int|
                0 <= p < i + 1 && #[trigger] so[p] == y) by {
                if y == x as int {
                    assert(so[i as int] == y);
                } else {
                    assert(seen@[y] == old_seen[y]);
                    if exists|p: int| 0 <= p < i + 1 && #[trigger] so[p] == y {
                        let p = choose|p: int| 0 <= p < i + 1 && #[trigger] so[p] == y;
                        assert(p != i as int);
                    }
                }
            }
        }
        i += 1;
    }
    let mut y: usize = 0;
    while y < n
        invariant
            n == order@.len(),
            so == dv_order(order@),
            seen@.len() == n,
            0 <= y <= n,
            forall|z: int|
                0 <= z < n ==> (#[trigger] seen@[z] <==> exists|p: int|
                    0 <= p < n && #[trigger] so[p] == z),
            forall|z: int| 0 <= z < y ==> #[trigger] seen@[z],
        decreases n - y,
    {
        if !seen[y] {
            assert(!in_order(so, y as int));
            return false;
        }
        y += 1;
    }
    assert forall|z: int| 0 <= z < n implies #[trigger] in_order(so, z) by {
        assert(seen@[z]);
    }
    true
}

pub fn check_chain<K: View, V: DeepView>(
    rs: &Vec<ERegion<K, V>>,
    order: &Vec<usize>,
    total: usize,
) -> (b: bool)
    requires
        forall|i: int| 0 <= i < order@.len() ==> 0 <= #[trigger] order@[i] < rs@.len(),
    ensures
        b == is_chain(dv_regions(rs@), dv_order(order@), total as int),
{
    let ghost srs = dv_regions(rs@);
    let ghost so = dv_order(order@);
    let n = order.len();
    if n == 0 {
        return total == 0;
    }
    let mut end: usize = 0;
    let mut i: usize = 0;
    while i < n
        invariant
            n == order@.len(),
            n > 0,
            srs == dv_regions(rs@),
            so == dv_order(order@),
            forall|p: int| 0 <= p < order@.len() ==> 0 <= #[trigger] order@[p] < rs@.len(),
            0 <= i <= n,
            i > 0 ==> srs[so[0]].start == 0,
            forall|p: int| 0 < p < i ==> #[trigger] srs[so[p]].start == layout_end(srs, so, p - 1),
            i == 0 ==> end == 0,
            i > 0 ==> end as int == layout_end(srs, so, i - 1),
        decreases n - i,
    {
        let x = order[i];
        assert(so[i as int] == x as int);
        let r = &rs[x];
        assert(srs[x as int] == r.deep_view());
        assert(srs[so[i as int]].start == r.start as int);
        if r.start != end {
            return false;
        }
        if r.len > usize::MAX - r.start {
            proof {
                assert(layout_end(srs, so, i as int) > usize::MAX);
                if i + 1 < n {
                    let nx = so[i + 1];
                    assert(srs[nx] == rs@[nx].deep_view());
                    assert(srs[so[i + 1]].start <= usize::MAX);
                }
            }
            return false;
        }
        end = r.start + r.len;
        i += 1;
    }
    end == total
}

pub fn check_tiled<K: View, V: DeepView>(
    rs: &Vec<ERegion<K, V>>,
    order: &Vec<usize>,
    total: usize,
) -> (b: bool)
    ensures
        b == spec_tiled(dv_regions(rs@), dv_order(order@), total as int),
{
    if !check_permutation(rs.len(), order) {
        return false;
    }
    proof {
        let so = dv_order(order@);
        assert forall|i: int| 0 <= i < order@.len() implies 0 <= #[trigger] order@[i] < rs@.len() by {
            assert(so[i] == order@[i] as int);
        }
    }
    check_chain(rs, order, total)
}

pub fn parse_file<'a, F: RankedFormatExec<'a>>(f: &F, root: &'a [u8]) -> (res: Option<
    Vec<ERegion<F::EKind, F::EVal>>,
>)
    ensures
        match res {
            Some(rs) => spec_parse_file(*f, root@) == Some(
                SFile { len: root@.len() as int, regions: dv_regions(rs@) },
            ),
            None => spec_parse_file(*f, root@) is None,
        },
{
    let j = match f.root(root) {
        Some(j) => j,
        None => {
            return None;
        },
    };
    let rs = match resolve(f, root, j) {
        Some(rs) => rs,
        None => {
            return None;
        },
    };
    let order = f.layout(&rs);
    if check_tiled(&rs, &order, root.len()) {
        Some(rs)
    } else {
        None
    }
}

} // verus!
